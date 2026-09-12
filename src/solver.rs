use crate::construction;
use crate::construction::{
    Circle, CircleId, Construct, Construction, Line, LineId, ObjectId, PointId,
    Shape,
};
use crate::field::Field;
use std::collections::HashMap;

/// Concrete 2D point over the field `F`.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Point<F: Field>(F, F);

/// Concrete objects over the field `F`. Objects can be a shape (line or circle,
/// see `crate::construction::Shape`) or a point.
#[derive(Clone, Debug)]
enum Object<F: Field> {
    Shape(Shape<Point<F>>),
    Point(Point<F>),
}

impl<F: Field> Object<F> {
    // TODO: figure out the idiomatic way to do this in Rust
    /// Convert an object to a point, panic otherwise.
    pub fn to_point(self) -> Point<F> {
        match self {
            Object::Shape(_) => panic!("Object is not a point"),
            Object::Point(point) => point,
        }
    }

    // TODO: figure out the idiomatic way to do this in Rust
    /// Same as the above, but for references.
    pub fn to_point_ref<'a>(&'a self) -> &'a Point<F> {
        match self {
            Object::Shape(_) => panic!("Object is not a point"),
            Object::Point(point) => point,
        }
    }

    // TODO: figure out the idiomatic way to do this in Rust
    /// Convert an object to a shape, panic otherwise.
    pub fn to_shape(self) -> Shape<Point<F>> {
        match self {
            Object::Shape(shape) => shape,
            Object::Point(_) => panic!("Object is not a shape"),
        }
    }

    // TODO: figure out the idiomatic way to do this in Rust
    /// Same as the above, but for references.
    pub fn to_shape_ref<'a>(&'a self) -> &'a Shape<Point<F>> {
        match self {
            Object::Shape(shape) => shape,
            Object::Point(_) => panic!("Object is not a shape"),
        }
    }
}

/// A mapping from `ObjectId`s of a construction to their concrete object
/// values. The values are over field `F`.
pub struct Assignment<F: Field>(HashMap<ObjectId, Object<F>>);

/// A `Resolver` decides what to do when we encounter a free choice during a
/// a construction solve (see `solve`).
pub trait Resolver<F: Field> {
    /// Choose the value of the free point referenced by `point_id`.
    fn choose_free_point(point_id: ObjectId) -> Point<F>;
    /// Choose the value of the point referenced by `point_id` and bound to
    /// `shape`.
    fn choose_bound_point(point: ObjectId, shape: &Shape<Point<F>>)
    -> Point<F>;
}

impl<F: Field> Assignment<F> {
    /// Create an empty assignment.
    fn new() -> Self {
        Self(HashMap::new())
    }

    /// Retrieve the value of the object referenced by `object_id`. Panics if
    /// the `object_id` is not assigned in the assignment.
    fn get(&self, object_id: ObjectId) -> &Object<F> {
        self.0
            .get(&object_id)
            .expect("Dependency should already be resolved")
    }

    /// Retrieve the value of the point referenced by `point_id`. Panics if
    /// no assignment for `point_id` exists.
    pub fn get_point(&self, point_id: PointId) -> &Point<F> {
        self.get(point_id.0).to_point_ref()
    }

    /// Retrieve the value of the line referenced by `line_id`. Panics if
    /// no assignment for `line_id` exists.
    pub fn get_line(&self, line_id: LineId) -> &Line<Point<F>> {
        self.get(line_id.0).to_shape_ref().to_line_ref()
    }

    /// Retrieve the value of the circle referenced by `circle_id`. Panics if
    /// no assignment for `circle_id` exists.
    pub fn get_circle(&self, circle_id: CircleId) -> &Circle<Point<F>> {
        self.get(circle_id.0).to_shape_ref().to_circle_ref()
    }

    /// Given a `shape` that depends on some objects in the construction,
    /// resolve the value of those dependencies and use them to construct a
    /// shape `assigned_shape` with concrete values, then add the mapping
    /// `object_ids[0]` -> `assigned_shape` to the assignment. Requires
    /// `object_ids` to have exactly 1 element.
    ///
    /// Returns true if we successfully added the assignment (always returns
    /// true for this function).
    ///
    /// Example: if we have the following assignment:
    ///   7 -> Point(3, 4),
    ///   8 -> Point(5, 6),
    /// and we are given the shape
    ///     shape = Circle{ center: 7, point: 8 },
    ///     object_ids[0] = 9
    /// then we compute
    ///     assigned_shape = Circle{ center: Point(3, 4), point: Point(5, 6) }
    /// and add the mapping
    ///     9 -> Circle{ center: Point(3, 4), point: (5, 6) }.
    fn assign_shape(
        &mut self,
        shape: &Shape<ObjectId>,
        object_ids: &Vec<ObjectId>,
    ) -> bool {
        let assigned_shape = match shape {
            Shape::Line(line) => {
                let point1 = self.get(line.point1).clone().to_point();
                let point2 = self.get(line.point2).clone().to_point();
                Object::Shape(Shape::Line(Line { point1, point2 }))
            }
            Shape::Circle(circle) => {
                let center = self.get(circle.center).clone().to_point();
                let point = self.get(circle.point).clone().to_point();
                Object::Shape(Shape::Circle(Circle { center, point }))
            }
        };
        assert_eq!(object_ids.len(), 1);
        self.0.insert(object_ids[0], assigned_shape);
        true
    }

    /// Same as the above but for points.
    ///
    /// Returns true if we successfully added the assignment. Note that since
    /// `point` can be an intersection (see `crate::construction::Point`), it's
    /// possible the current assignment of objects do not yield any intersection
    /// for `point`, then this function returns false.
    ///
    /// We require that `object_ids.len()` matches the expected number of points
    /// generated from `point`.
    ///
    /// If `point` is a free or bound point, use `R` to choose the point's
    /// value.
    fn assign_point<R: Resolver<F>>(
        &mut self,
        point: &construction::Point,
        object_ids: &Vec<ObjectId>,
    ) -> bool {
        let assigned_points = match point {
            construction::Point::Free => {
                assert_eq!(object_ids.len(), 1);
                vec![R::choose_free_point(object_ids[0])]
            }
            construction::Point::Bound(shape) => {
                assert_eq!(object_ids.len(), 1);
                let shape = self.get(*shape).clone().to_shape();
                vec![R::choose_bound_point(object_ids[0], &shape)]
            }
            construction::Point::Intersect(shape1, shape2) => {
                let shape1 = self.get(*shape1).clone().to_shape();
                let shape2 = self.get(*shape2).clone().to_shape();
                let points = compute_intersection(&shape1, &shape2);
                if points.is_empty() {
                    return false;
                }
                points
            }
        };
        assert_eq!(object_ids.len(), assigned_points.len());
        for (object_id, assigned_point) in
            object_ids.iter().zip(assigned_points)
        {
            self.0.insert(*object_id, Object::Point(assigned_point));
        }
        true
    }
}

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

/// Compute the intersection between `line1` and `line2`. Returns an empty
/// vector if the two lines coincide or if they are parallel to each other,
/// otherwise returns a singleton vector of the intersection point.
fn compute_line_line_intersection<F: Field>(
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
        // The intersection is computed using Cramer's rule.
        vec![(Point((b1 * c2 - b2 * c1) * d_inv, (c1 * a2 - c2 * a1) * d_inv))]
    }
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

/// Compute the intersection points of `line` and `circle`. Return an empty
/// vector if there is no intersection.
///
/// See docs/line_circle_intersections.pdf for derivation of the intersections.
fn compute_line_circle_intersection<F: Field>(
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

/// Compute the intersection points of `circle1` and `circle2`. Return an empty
/// vector if there is no intersection.
///
/// See docs/line_circle_intersections.pdf for derivation of the intersections.
fn compute_circle_circle_intersection<F: Field>(
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

/// Compute the intersection of two arbitrary shapes (line or circle) `shape1`
/// and `shape2`. Return an empty vector if there is no intersection.
fn compute_intersection<F: Field>(
    shape1: &Shape<Point<F>>,
    shape2: &Shape<Point<F>>,
) -> Vec<Point<F>> {
    match shape1 {
        Shape::Line(line1) => match shape2 {
            Shape::Line(line2) => compute_line_line_intersection(line1, line2),
            Shape::Circle(circle2) => {
                compute_line_circle_intersection(line1, circle2)
            }
        },
        Shape::Circle(circle1) => match shape2 {
            Shape::Line(line2) => {
                compute_line_circle_intersection(line2, circle1)
            }
            Shape::Circle(circle2) => {
                compute_circle_circle_intersection(circle1, circle2)
            }
        },
    }
}

/// Compute an assignment for `construction`, using resolver `R` to pick the
/// free choices and derive the objects from the steps deterministically.
/// Returns `None` if any of the following happens:
/// * The construction is invalid (the dependencies do not form a DAG).
/// * Some step cannot successfully compute an intersection.
pub fn solve<F: Field, R: Resolver<F>>(
    construction: &Construction,
) -> Option<Assignment<F>> {
    if !construction.is_valid() {
        return None;
    }
    let mut assignments = Assignment::new();
    for step in construction.into_iter() {
        let success = match &step.construct {
            Construct::Shape(shape) => {
                assignments.assign_shape(shape, &step.objects)
            }
            Construct::Point(point) => {
                assignments.assign_point::<R>(point, &step.objects)
            }
        };
        if !success {
            return None;
        }
    }
    Some(assignments)
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
        let intersections = compute_line_line_intersection(&line1, &line2);
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
        let intersections = compute_line_line_intersection(&line1, &line2);
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
                compute_line_circle_intersection(&line, &circle);
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
                compute_line_circle_intersection(&line, &circle);
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
                compute_circle_circle_intersection(&circle1, &circle2);
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
                compute_circle_circle_intersection(&circle1, &circle2);
            assert!(intersections.is_empty());
        }
    }

    struct SameDistPointResolver;
    impl<F: Field> Resolver<F> for SameDistPointResolver {
        /// Assign the free points such that dist(p0, p1) == dist(p2, p3) == 5.
        fn choose_free_point(object_id: ObjectId) -> Point<F> {
            let zero = F::zero();
            let one = F::one();
            let two = one + one;
            let three = two + one;
            let four = three + one;
            let five = four + one;
            let six = five + one;
            match object_id.0 {
                0 => Point(zero, zero),
                1 => Point(five, zero),
                2 => Point(six, zero),
                3 => Point(one, zero),
                _ => panic!("Expected only 4 free points"),
            }
        }

        fn choose_bound_point(_: ObjectId, _: &Shape<Point<F>>) -> Point<F> {
            unimplemented!("Bound point resolver should not be called")
        }
    }

    struct DiffDistPointResolver;
    impl<F: Field> Resolver<F> for DiffDistPointResolver {
        // Assign the free points such that dist(p0, p1) != dist(p2, p3).
        fn choose_free_point(object_id: ObjectId) -> Point<F> {
            let zero = F::zero();
            let one = F::one();
            let neg_one = one.add_inv();
            let two = one + one;
            let three = two + one;
            let four = three + one;
            let five = four + one;
            match object_id.0 {
                0 => Point(zero, zero),
                1 => Point(five, zero),
                2 => Point(three, zero),
                3 => Point(neg_one, zero),
                _ => panic!("Expected only 4 free points"),
            }
        }

        fn choose_bound_point(_: ObjectId, _: &Shape<Point<F>>) -> Point<F> {
            unimplemented!("Bound point resolver should not be called")
        }
    }

    #[test]
    fn test_solver() {
        type F = FiniteField<67>;
        let mut construction = Construction::new();
        let center1 = construction.add_free_point();
        let point1 = construction.add_free_point();
        let center2 = construction.add_free_point();
        let point2 = construction.add_free_point();
        let circle1 = construction.add_circle(center1, point1);
        let circle2 = construction.add_circle(center2, point2);
        let (intersect1, intersect2) =
            construction.add_circle_circle_intersection(circle1, circle2);
        let line1 = construction.add_line(center1, center2);
        let line2 = construction.add_line(intersect1, intersect2);
        let center = construction.add_line_line_intersection(line1, line2);

        // If `dist(center1, point1) == dist(point2, center2)`, then `center`
        // will be the center of `center1` and `center2`.
        let assignments = solve::<F, SameDistPointResolver>(&construction);
        assert!(assignments.is_some());
        let assignments = assignments.unwrap();
        let Point(x1, y1) = assignments.get_point(center1);
        let Point(x2, y2) = assignments.get_point(center2);
        let Point(x, y) = assignments.get_point(center);
        assert_eq!(*x1 + *x2, *x + *x);
        assert_eq!(*y1 + *y2, *y + *y);

        // The opposite case of the above.
        let assignments = solve::<F, DiffDistPointResolver>(&construction);
        assert!(assignments.is_some());
        let assignments = assignments.unwrap();
        let Point(x1, _) = assignments.get_point(center1);
        let Point(x2, _) = assignments.get_point(center2);
        let Point(x, _) = assignments.get_point(center);
        assert_ne!(*x1 + *x2, *x + *x);
    }
}
