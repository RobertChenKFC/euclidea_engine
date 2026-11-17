use crate::construction;
use crate::construction::{
    Circle, CircleId, Construct, Construction, Line, LineId, ObjectId, PointId,
    Shape,
};
use crate::field::Field;
use crate::intersector::Intersector;
use std::collections::HashMap;

/// Concrete 2D point over the field `F`.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Point<F: Field>(pub F, pub F);

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
    fn assign_point<R: Resolver<F>, I: Intersector<F>>(
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
                let points = I::compute_intersection(&shape1, &shape2);
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

/// Compute an assignment for `construction`, using resolver `R` to pick the
/// free choices and derive the objects from the steps deterministically.
/// Returns `None` if any of the following happens:
/// * The construction is invalid (the dependencies do not form a DAG).
/// * Some step cannot successfully compute an intersection.
pub fn solve<F: Field, R: Resolver<F>, I: Intersector<F>>(
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
                assignments.assign_point::<R, I>(point, &step.objects)
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
    use crate::intersector::default_intersector::DefaultIntersector;

    struct SameDistPointResolver;
    impl<F: Field> Resolver<F> for SameDistPointResolver {
        /// Assign the free points such that dist(p0, p1) == dist(p2, p3) == 5.
        fn choose_free_point(object_id: ObjectId) -> Point<F> {
            let zero = F::zero();
            let one = F::one();
            let two = &one + &one;
            let three = &two + &one;
            let four = &three + &one;
            let five = &four + &one;
            let six = &five + &one;
            match object_id.0 {
                0 => Point(zero.clone(), zero.clone()),
                1 => Point(five, zero.clone()),
                2 => Point(six, zero.clone()),
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
            let two = &one + &one;
            let three = &two + &one;
            let four = &three + &one;
            let five = four + one;
            match object_id.0 {
                0 => Point(zero.clone(), zero.clone()),
                1 => Point(five, zero.clone()),
                2 => Point(three, zero.clone()),
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
        let assignments = solve::<F, SameDistPointResolver, DefaultIntersector>(
            &construction,
        );
        assert!(assignments.is_some());
        let assignments = assignments.unwrap();
        let Point(x1, y1) = assignments.get_point(center1);
        let Point(x2, y2) = assignments.get_point(center2);
        let Point(x, y) = assignments.get_point(center);
        assert_eq!(*x1 + *x2, *x + *x);
        assert_eq!(*y1 + *y2, *y + *y);

        // The opposite case of the above.
        let assignments = solve::<F, DiffDistPointResolver, DefaultIntersector>(
            &construction,
        );
        assert!(assignments.is_some());
        let assignments = assignments.unwrap();
        let Point(x1, _) = assignments.get_point(center1);
        let Point(x2, _) = assignments.get_point(center2);
        let Point(x, _) = assignments.get_point(center);
        assert_ne!(*x1 + *x2, *x + *x);
    }
}
