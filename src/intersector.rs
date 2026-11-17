pub mod default_intersector;

use crate::construction::{Circle, Line, Shape};
use crate::field::Field;
use crate::solver::Point;

/// Represents abstract methods for computing intersection points between lines
/// and circles.
pub trait Intersector<F: Field> {
    /// Compute the intersection between `line1` and `line2`. Returns an empty
    /// vector if the two lines coincide or if they are parallel to each other,
    /// otherwise returns a singleton vector of the intersection point.
    fn compute_line_line_intersection(
        line1: &Line<Point<F>>,
        line2: &Line<Point<F>>,
    ) -> Vec<Point<F>>;

    /// Compute the intersection points of `line` and `circle`. Return an empty
    /// vector if there is no intersection.
    fn compute_line_circle_intersection(
        line: &Line<Point<F>>,
        circle: &Circle<Point<F>>,
    ) -> Vec<Point<F>>;

    /// Compute the intersection points of `circle1` and `circle2`. Return an empty
    /// vector if there is no intersection.
    fn compute_circle_circle_intersection(
        circle1: &Circle<Point<F>>,
        circle2: &Circle<Point<F>>,
    ) -> Vec<Point<F>>;

    /// Compute the intersection of two arbitrary shapes (line or circle)
    /// `shape1` and `shape2`. Return an empty vector if there is no
    /// intersection.
    fn compute_intersection(
        shape1: &Shape<Point<F>>,
        shape2: &Shape<Point<F>>,
    ) -> Vec<Point<F>> {
        match shape1 {
            Shape::Line(line1) => match shape2 {
                Shape::Line(line2) => {
                    Self::compute_line_line_intersection(line1, line2)
                }
                Shape::Circle(circle2) => {
                    Self::compute_line_circle_intersection(line1, circle2)
                }
            },
            Shape::Circle(circle1) => match shape2 {
                Shape::Line(line2) => {
                    Self::compute_line_circle_intersection(line2, circle1)
                }
                Shape::Circle(circle2) => {
                    Self::compute_circle_circle_intersection(circle1, circle2)
                }
            },
        }
    }
}
