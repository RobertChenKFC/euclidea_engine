use std::slice::Iter;

/// A `Line` is represented by a pair of points that the line passes through. The
/// point can be of any type. This allows us to represent:
/// * Abstract lines: see `ObjectId`, which is a reference to an object in a
///   construction.
/// * Concrete lines: see `crate::solver::Point`, which is a pair of concrete
///   elements of a field.
#[derive(Clone, Debug)]
pub struct Line<Point: Clone> {
    pub point1: Point,
    pub point2: Point,
}

impl<Point: Clone> Line<Point> {
    /// Get the dependencies of the line.
    fn get_deps<'a>(&'a self) -> Vec<&'a Point> {
        vec![&self.point1, &self.point2]
    }
}

/// A `Circle` is represented by a pair of points. Similar to `Line`, the point
/// can be of any type.
#[derive(Clone, Debug)]
pub struct Circle<Point: Clone> {
    /// The center of the circle.
    pub center: Point,
    /// A point on the circle's perimeter.
    pub point: Point,
}

impl<Point: Clone> Circle<Point> {
    /// Get the dependencies of the circle.
    fn get_deps<'a>(&'a self) -> Vec<&'a Point> {
        vec![&self.center, &self.point]
    }
}

/// A shape can be a line or a circle.
#[derive(Clone, Debug)]
pub enum Shape<Point: Clone> {
    Line(Line<Point>),
    Circle(Circle<Point>),
}

impl<Point: Clone> Shape<Point> {
    /// Get the dependencies of the shape.
    fn get_deps<'a>(&'a self) -> Vec<&'a Point> {
        match self {
            Shape::Line(line) => line.get_deps(),
            Shape::Circle(circle) => circle.get_deps(),
        }
    }

    // TODO: figure out the idomatic way to do this in Rust.
    /// Convert the shape into a `Line`, panic otherwise.
    pub fn to_line_ref<'a>(&'a self) -> &'a Line<Point> {
        match self {
            Shape::Line(line) => line,
            Shape::Circle(_) => panic!("Shape is not a line"),
        }
    }

    // TODO: figure out the idomatic way to do this in Rust.
    /// Convert the shape into a `Circle`, panic otherwise.
    pub fn to_circle_ref<'a>(&'a self) -> &'a Circle<Point> {
        match self {
            Shape::Line(_) => panic!("Shape is not a circle"),
            Shape::Circle(circle) => circle,
        }
    }
}

/// A reference to an object created in a `Construction`. See `Step` on how a
/// construction can create an object. The underlying `usize` of the `ObjectId`
/// is an index that is guaranteed to start from 0 (for the first object
/// created) and strictly increase by 1 for each new object created.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct ObjectId(pub usize);

impl ObjectId {
    /// Create a new `ObjectId` from the underlying index.
    fn new(object_id: usize) -> ObjectId {
        ObjectId(object_id)
    }
}

/// `ObjectId` wrapper for points. This exists to use type safety to guarantee
/// correct construction (eg. a line can be constructed from two `PointId`s but
/// not two `LineId`s).
#[derive(Copy, Clone)]
pub struct PointId(pub ObjectId);

/// `ObjectId` wrapper for lines. See `PointId` for purpose.
#[derive(Copy, Clone)]
pub struct LineId(pub ObjectId);

/// `CircleId` wrapper for circles. See `PointId` for purpose.
#[derive(Copy, Clone)]
pub struct CircleId(pub ObjectId);

/// Represents the construction of a point from the given dependencies.
#[derive(Clone)]
pub enum Point {
    /// Create a free point anywhere on the 2D plane.
    Free,
    /// Create a point bound to the object `ObjectId` references. `ObjectId`
    /// must refer to a shape.
    /// - If `ObjectId` refers to a line, the point lies anywhere on that line.
    /// - If `ObjectId` refers to a circle, the point lies anywhere on the
    ///   circumference of the circle.
    Bound(ObjectId),
    /// Create point(s) that are the intersection of the objects the `ObjectId`s
    /// reference. Both `ObjectId` must refer to a shape.
    Intersect(ObjectId, ObjectId),
}

impl Point {
    // Get the dependencies of a point construct.
    fn get_deps<'a>(&'a self) -> Vec<&'a ObjectId> {
        match self {
            Point::Free => vec![],
            Point::Bound(shape) => vec![shape],
            Point::Intersect(shape1, shape2) => vec![shape1, shape2],
        }
    }
}

/// For each step of the construction, we can either construct a shape (a line
/// or circle) or construct points.
#[derive(Clone)]
pub enum Construct {
    Shape(Shape<ObjectId>),
    Point(Point),
}

impl Construct {
    /// Get the dependencies of a construct.
    fn get_deps<'a>(&'a self) -> Vec<&'a ObjectId> {
        match self {
            Construct::Shape(shape) => shape.get_deps(),
            Construct::Point(point) => point.get_deps(),
        }
    }
}

/// Each step of the construction is represented by the construct we are
/// building and the objects this construct creates.
pub struct Step {
    /// The construct we are building in this step. This represents what kind
    /// of construct we are building and which previous objects we build on
    /// (see `Construct`).
    pub construct: Construct,
    /// The objects `construct` creates.
    pub objects: Vec<ObjectId>,
}

/// A construction is a collection of steps, each step creating objects that are
/// built on objects created in the previous steps.
pub struct Construction {
    steps: Vec<Step>,
    num_objects: usize,
}

impl Construction {
    /// Create an empty construction.
    pub fn new() -> Construction {
        Construction {
            steps: vec![],
            num_objects: 0,
        }
    }

    /// Get object id for new object. The objects are guaranteed to be numbered
    /// sequentially starting from 0 (see `ObjectId`).
    fn get_new_object_id(&mut self) -> ObjectId {
        let object_id = ObjectId::new(self.num_objects);
        self.num_objects += 1;
        object_id
    }

    /// Add a step to the construction that creates a free point (see
    /// `Point::Free`).
    pub fn add_free_point(&mut self) -> PointId {
        let point = self.get_new_object_id();
        let step = Step {
            construct: Construct::Point(Point::Free),
            objects: vec![point],
        };
        self.steps.push(step);
        PointId(point)
    }

    /// Add a step to the construction that creates a point bound to
    /// `bound_object` (see `Point::Bound`).
    fn add_bound_point(&mut self, bound_object: ObjectId) -> PointId {
        let point = self.get_new_object_id();
        let step = Step {
            construct: Construct::Point(Point::Bound(bound_object)),
            objects: vec![point],
        };
        self.steps.push(step);
        PointId(point)
    }

    /// Add a step to the construction that creates a point bound to `line (see
    /// `Point::Bound`).
    pub fn add_point_on_line(&mut self, line: LineId) -> PointId {
        self.add_bound_point(line.0)
    }

    /// Add a step to the construction that creates a point bound to `circle`
    /// (see `Point::Bound`).
    pub fn add_point_on_circle(&mut self, circle: LineId) -> PointId {
        self.add_bound_point(circle.0)
    }

    /// Add a step to the construction that computes the intersection `points`
    /// of `object1` and `object2` (see `Point::Intersection`).
    fn add_intersection(
        &mut self,
        object1: ObjectId,
        object2: ObjectId,
        points: Vec<ObjectId>,
    ) {
        let step = Step {
            construct: Construct::Point(Point::Intersect(object1, object2)),
            objects: points,
        };
        self.steps.push(step);
    }

    /// Add a step to the construction that computes the interesection point of
    /// `line1` and `line2` (see `Point::Intersection`).
    pub fn add_line_line_intersection(
        &mut self,
        line1: LineId,
        line2: LineId,
    ) -> PointId {
        let point = self.get_new_object_id();
        self.add_intersection(line1.0, line2.0, vec![point]);
        PointId(point)
    }

    /// Add a step to the construction that computes the interesection points of
    /// `line` and `circle` (see `Point::Intersection`).
    pub fn add_line_circle_intersection(
        &mut self,
        line: LineId,
        circle: CircleId,
    ) -> (PointId, PointId) {
        let point1 = self.get_new_object_id();
        let point2 = self.get_new_object_id();
        self.add_intersection(line.0, circle.0, vec![point1, point2]);
        (PointId(point1), PointId(point2))
    }

    /// Add a step to the construction that computes the interesection points of
    /// `circle1` and `circle2` (see `Point::Intersection`).
    pub fn add_circle_circle_intersection(
        &mut self,
        circle1: CircleId,
        circle2: CircleId,
    ) -> (PointId, PointId) {
        let point1 = self.get_new_object_id();
        let point2 = self.get_new_object_id();
        self.add_intersection(circle1.0, circle2.0, vec![point1, point2]);
        (PointId(point1), PointId(point2))
    }

    /// Add a step to the construction that creates a line that passes through
    /// `point1` and `point2` (see `Line`).
    pub fn add_line(&mut self, point1: PointId, point2: PointId) -> LineId {
        let line = self.get_new_object_id();
        self.steps.push(Step {
            construct: Construct::Shape(Shape::Line(Line {
                point1: point1.0,
                point2: point2.0,
            })),
            objects: vec![line],
        });
        LineId(line)
    }

    /// Add a step to the construction that creates a circle that's centered
    /// at `center` and passes through `point` (see `Circle`).
    pub fn add_circle(&mut self, center: PointId, point: PointId) -> CircleId {
        let circle = self.get_new_object_id();
        self.steps.push(Step {
            construct: Construct::Shape(Shape::Circle(Circle {
                center: center.0,
                point: point.0,
            })),
            objects: vec![circle],
        });
        CircleId(circle)
    }

    /// Check that the object dependencies of each step of the construction
    /// form a DAG.
    ///
    /// For this specific implementation, we require a stronger constraint:
    /// every object must only reference an object of smaller `ObjectId`. For
    /// typical usage of `Construction` where each step only uses objects
    /// created in the previous step, this constraint trivially holds.
    pub fn is_valid(&self) -> bool {
        for step in &self.steps {
            for dep_object_id in step.construct.get_deps() {
                for object_id in &step.objects {
                    if dep_object_id.0 >= object_id.0 {
                        return false;
                    }
                }
            }
        }
        true
    }
}

/// Given a `Construction`, you can iterate over all its steps.
impl<'a> IntoIterator for &'a Construction {
    type Item = &'a Step;
    type IntoIter = Iter<'a, Step>;

    fn into_iter(self) -> Self::IntoIter {
        self.steps.iter()
    }
}
