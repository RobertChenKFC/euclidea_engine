use crate::construction::{Circle, Line};
use crate::field::Field;
use crate::intersector::Intersector;
use crate::solver::Point;

/// Compute the coefficients (a, b, c) for the line equation
///   ax + by + c = 0
/// that `line` represents.
///
/// With the two points (x1, y1), (x2, y2) that `line` passes through, the line
/// equation is
///   (y1 - y2)x + (x2 - x1)y + x1y2 - x2y1 = 0,
/// which we can easily verify by substituing the points in and doing simple
/// algebra.
fn get_line_coeff<F: Field>(line: &Line<Point<F>>) -> (F, F, F) {
    let Point(x1, y1) = line.point1;
    let Point(x2, y2) = line.point2;
    (y1 - y2, x2 - x1, x1 * y2 - x2 * y1)
}

/// Compute the coefficients (a, b, c) for the circle equation
///   (x - a)**2 + (y - b)**2 = c
/// that `circle` represents.
fn get_circle_coeff<F: Field>(circle: &Circle<Point<F>>) -> (F, F, F) {
    let Circle {
        center: Point(x1, y1),
        point: Point(x2, y2),
    } = circle;
    let dx = *x1 - *x2;
    let dy = *y1 - *y2;
    (*x1, *y1, dx * dx + dy * dy)
}

/// `DefaultIntersector` is a naive algorithm for computing line and circle
/// intersections. See docs/line_circle_intersections.pdf for the derivation.
pub struct DefaultIntersector;

impl<F: Field> Intersector<F> for DefaultIntersector {
    fn compute_line_line_intersection(
        line1: &Line<Point<F>>,
        line2: &Line<Point<F>>,
    ) -> Vec<Point<F>> {
        let (a1, b1, c1) = get_line_coeff(line1);
        let (a2, b2, c2) = get_line_coeff(line2);
        let d = a1 * b2 - a2 * b1;
        if d == F::zero() {
            vec![]
        } else {
            let d_inv = d.mul_inv();
            vec![
                (Point(
                    (b1 * c2 - b2 * c1) * d_inv,
                    (c1 * a2 - c2 * a1) * d_inv,
                )),
            ]
        }
    }

    fn compute_line_circle_intersection(
        line: &Line<Point<F>>,
        circle: &Circle<Point<F>>,
    ) -> Vec<Point<F>> {
        let (a, b, c) = get_line_coeff(&line);
        let (u, v, r2) = get_circle_coeff(&circle);
        let line_mag = a * a + b * b;
        let line_mag_inv = line_mag.mul_inv();
        let center_on_line = a * u + b * v + c;
        let t0 = center_on_line * line_mag_inv;
        let xm = u - a * t0;
        let ym = v - b * t0;
        let discriminant = line_mag * r2 - center_on_line * center_on_line;
        match discriminant.sqrt() {
            Some(s) => {
                let scale = s * line_mag_inv;
                vec![
                    Point(xm - b * scale, ym + a * scale),
                    Point(xm + b * scale, ym - a * scale),
                ]
            }
            None => {
                vec![]
            }
        }
    }

    fn compute_circle_circle_intersection(
        circle1: &Circle<Point<F>>,
        circle2: &Circle<Point<F>>,
    ) -> Vec<Point<F>> {
        let (u1, v1, r12) = get_circle_coeff(circle1);
        let (u2, v2, r22) = get_circle_coeff(circle2);
        let dx = u2 - u1;
        let dy = v2 - v1;
        let d2 = dx * dx + dy * dy;
        let two = F::one() + F::one();
        let half = two.mul_inv();
        let k = r12 - r22 + d2;
        let half_k = k * half;
        let discriminant = d2 * r12 - half_k * half_k;
        match discriminant.sqrt() {
            Some(s) => {
                let scale = s / d2;
                let two_d2_inv = (two * d2).mul_inv();
                let xm = u1 + k * dx * two_d2_inv;
                let ym = v1 + k * dy * two_d2_inv;
                vec![
                    Point(xm - dy * scale, ym + dx * scale),
                    Point(xm + dy * scale, ym - dx * scale),
                ]
            }
            None => {
                vec![]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::finite_field::FiniteField;

    /// Returns true iff `point` is on `line`.
    fn point_is_on_line<F: Field>(
        point: &Point<F>,
        line: &Line<Point<F>>,
    ) -> bool {
        let Point(x, y) = point;
        let (a, b, c) = get_line_coeff(&line);
        a * (*x) + b * (*y) + c == F::zero()
    }

    /// Returns true iff `point` is on `circle`.
    fn point_is_on_circle<F: Field>(
        point: &Point<F>,
        circle: &Circle<Point<F>>,
    ) -> bool {
        let Point(x1, y1) = point;
        let (x2, y2, r2) = get_circle_coeff(&circle);
        let dx = *x1 - x2;
        let dy = *y1 - y2;
        dx * dx + dy * dy == r2
    }

    /// Test if `get_line_coeff` returns the correct coefficients for the line
    /// equation by checking if the two points used to form the line actually
    /// satisifies the line equation.
    #[test]
    fn test_line_coeff() {
        type F = FiniteField<23>;
        let point_pairs = [
            ((21, 3), (20, 21)),
            ((14, 6), (21, 22)),
            ((3, 20), (11, 11)),
            ((2, 5), (7, 11)),
            ((3, 3), (21, 14)),
        ];
        for ((x1, y1), (x2, y2)) in point_pairs {
            let x1 = F::new(x1);
            let y1 = F::new(y1);
            let x2 = F::new(x2);
            let y2 = F::new(y2);
            let point1 = Point(x1, y1);
            let point2 = Point(x2, y2);
            let line = Line {
                point1: point1.clone(),
                point2: point2.clone(),
            };
            assert!(point_is_on_line(&point1, &line));
            assert!(point_is_on_line(&point2, &line));
        }
    }

    /// Test if `compute_line_line_intersection` correctly finds the
    /// intersection of two non-parallel lines and fail to find the intersection
    /// for two parallel lines.
    #[test]
    fn test_line_line_intersection() {
        type F = FiniteField<23>;
        let line1 = Line {
            point1: Point(F::new(0), F::new(0)),
            point2: Point(F::new(2), F::new(4)),
        };
        let line2 = Line {
            point1: Point(F::new(3), F::new(1)),
            point2: Point(F::new(5), F::new(0)),
        };
        let intersections =
            DefaultIntersector::compute_line_line_intersection(&line1, &line2);
        assert_eq!(intersections.len(), 1);
        for point in intersections {
            assert!(point_is_on_line(&point, &line1));
            assert!(point_is_on_line(&point, &line2));
        }

        let line1 = Line {
            point1: Point(F::new(0), F::new(0)),
            point2: Point(F::new(1), F::new(1)),
        };
        let line2 = Line {
            point1: Point(F::new(0), F::new(1)),
            point2: Point(F::new(1), F::new(2)),
        };
        let intersections =
            DefaultIntersector::compute_line_line_intersection(&line1, &line2);
        assert!(intersections.is_empty());
    }

    /// Test if `compute_line_circle_intersection` correctly finds the
    /// intersection points of a line and a circle  and fails to find
    /// intersection points of non-intersecting line and circle.
    #[test]
    fn test_line_circle_intersection() {
        type F = FiniteField<31>;

        let half = F::new(2).mul_inv();
        let pairs = [
            (
                Line {
                    point1: Point(F::new(1), half),
                    point2: Point(F::new(2), F::new(1)),
                },
                Circle {
                    center: Point(F::new(5) * half, F::new(0)),
                    point: Point(F::new(5), F::new(0)),
                },
            ),
            (
                Line {
                    point1: Point(F::new(0), F::new(2)),
                    point2: Point(F::new(8), F::new(10)),
                },
                Circle {
                    center: Point(F::new(6), F::new(4)),
                    point: Point(F::new(10), F::new(4)),
                },
            ),
        ];
        for (line, circle) in pairs {
            let intersections =
                DefaultIntersector::compute_line_circle_intersection(
                    &line, &circle,
                );
            assert_eq!(intersections.len(), 2);
            for point in intersections {
                assert!(point_is_on_line(&point, &line));
                assert!(point_is_on_circle(&point, &circle));
            }
        }

        let pairs = [
            (
                Line {
                    point1: Point(F::new(4), F::new(10)),
                    point2: Point(F::new(6), F::new(10)),
                },
                Circle {
                    center: Point(F::new(6), F::new(4)),
                    point: Point(F::new(10), F::new(4)),
                },
            ),
            (
                Line {
                    point1: Point(F::new(16), F::new(0)),
                    point2: Point(F::new(0), F::new(16)),
                },
                Circle {
                    center: Point(F::new(6), F::new(4)),
                    point: Point(F::new(10), F::new(4)),
                },
            ),
        ];
        for (line, circle) in pairs {
            let intersections =
                DefaultIntersector::compute_line_circle_intersection(
                    &line, &circle,
                );
            assert!(intersections.is_empty());
        }
    }

    /// Test if `compute_circle_circle_intersection` correctly finds the
    /// intersection points of two intersecting circles while failing to find
    /// intersection points of two non-intersecting (eg. concentric) circles.
    #[test]
    fn test_circle_circle_intersection() {
        type F = FiniteField<31>;

        let pairs = [
            (
                Circle {
                    center: Point(F::new(10), F::new(8)),
                    point: Point(F::new(16), F::new(0)),
                },
                Circle {
                    center: Point(F::new(0), F::new(28)),
                    point: Point(F::new(20).add_inv(), F::new(28)),
                },
            ),
            (
                Circle {
                    center: Point(F::new(6), F::new(2)),
                    point: Point(F::new(5), F::new(2)),
                },
                Circle {
                    center: Point(F::new(0), F::new(28)),
                    point: Point(F::new(20).add_inv(), F::new(28)),
                },
            ),
        ];
        for (circle1, circle2) in pairs {
            let intersections =
                DefaultIntersector::compute_circle_circle_intersection(
                    &circle1, &circle2,
                );
            assert_eq!(intersections.len(), 2);
            for point in intersections {
                assert!(point_is_on_circle(&point, &circle1));
                assert!(point_is_on_circle(&point, &circle2));
            }
        }

        let pairs = [
            (
                Circle {
                    center: Point(F::new(3), F::new(4)),
                    point: Point(F::new(0), F::new(2)),
                },
                Circle {
                    center: Point(F::new(3), F::new(4)),
                    point: Point(F::new(2), F::new(5)),
                },
            ),
            (
                Circle {
                    center: Point(F::new(1), F::new(2)),
                    point: Point(F::new(0), F::new(3)),
                },
                Circle {
                    center: Point(F::new(4), F::new(2)),
                    point: Point(F::new(5), F::new(2)),
                },
            ),
        ];
        for (circle1, circle2) in pairs {
            let intersections =
                DefaultIntersector::compute_circle_circle_intersection(
                    &circle1, &circle2,
                );
            assert!(intersections.is_empty());
        }
    }
}
