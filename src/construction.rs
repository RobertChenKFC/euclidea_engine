use std::slice::Iter;

#[derive(Clone, Debug)]
pub struct Line<Point: Clone> {
    pub point1: Point,
    pub point2: Point,
}

impl<Point: Clone> Line<Point> {
    fn get_deps<'a>(&'a self) -> Vec<&'a Point> {
        vec![&self.point1, &self.point2]
    }
}

#[derive(Clone, Debug)]
pub struct Circle<Point: Clone> {
    pub center: Point,
    pub point: Point,
}

impl<Point: Clone> Circle<Point> {
    fn get_deps<'a>(&'a self) -> Vec<&'a Point> {
        vec![&self.center, &self.point]
    }
}

#[derive(Clone, Debug)]
pub enum Shape<Point: Clone> {
    Line(Line<Point>),
    Circle(Circle<Point>),
}

impl<Point: Clone> Shape<Point> {
    fn get_deps<'a>(&'a self) -> Vec<&'a Point> {
        match self {
            Shape::Line(line) => line.get_deps(),
            Shape::Circle(circle) => circle.get_deps(),
        }
    }

    // TODO: figure out the idomatic way to do this in Rust
    pub fn to_line_ref<'a>(&'a self) -> &'a Line<Point> {
        match self {
            Shape::Line(line) => line,
            Shape::Circle(_) => panic!("Shape is not a line"),
        }
    }

    pub fn to_circle_ref<'a>(&'a self) -> &'a Circle<Point> {
        match self {
            Shape::Line(_) => panic!("Shape is not a circle"),
            Shape::Circle(circle) => circle,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct ObjectId(pub usize);

impl ObjectId {
    pub fn new(object_id: usize) -> ObjectId {
        ObjectId(object_id)
    }
}

#[derive(Copy, Clone)]
pub struct PointId(pub ObjectId);
#[derive(Copy, Clone)]
pub struct LineId(pub ObjectId);
#[derive(Copy, Clone)]
pub struct CircleId(pub ObjectId);

#[derive(Clone)]
pub enum Point {
    Free,
    Bound(ObjectId),
    Intersect(ObjectId, ObjectId),
}

impl Point {
    fn get_deps<'a>(&'a self) -> Vec<&'a ObjectId> {
        match self {
            Point::Free => vec![],
            Point::Bound(shape) => vec![shape],
            Point::Intersect(shape1, shape2) => vec![shape1, shape2],
        }
    }
}

#[derive(Clone)]
pub enum Construct {
    Shape(Shape<ObjectId>),
    Point(Point),
}

impl Construct {
    fn get_deps<'a>(&'a self) -> Vec<&'a ObjectId> {
        match self {
            Construct::Shape(shape) => shape.get_deps(),
            Construct::Point(point) => point.get_deps(),
        }
    }
}

pub struct Step {
    pub construct: Construct,
    pub objects: Vec<ObjectId>,
}

pub struct Construction {
    steps: Vec<Step>,
    num_objects: usize,
}

impl Construction {
    pub fn new() -> Construction {
        Construction {
            steps: vec![],
            num_objects: 0,
        }
    }

    fn get_new_object_id(&mut self) -> ObjectId {
        let object_id = ObjectId::new(self.num_objects);
        self.num_objects += 1;
        object_id
    }

    pub fn add_free_point(&mut self) -> PointId {
        let point_id = self.get_new_object_id();
        let step = Step {
            construct: Construct::Point(Point::Free),
            objects: vec![point_id],
        };
        self.steps.push(step);
        PointId(point_id)
    }

    fn add_bound_point(&mut self, bound_object_id: ObjectId) -> PointId {
        let point_id = self.get_new_object_id();
        let step = Step {
            construct: Construct::Point(Point::Bound(bound_object_id)),
            objects: vec![point_id],
        };
        self.steps.push(step);
        PointId(point_id)
    }

    pub fn add_point_on_line(&mut self, line_id: LineId) -> PointId {
        self.add_bound_point(line_id.0)
    }

    pub fn add_point_on_circle(&mut self, circle_id: LineId) -> PointId {
        self.add_bound_point(circle_id.0)
    }

    fn add_intersection(
        &mut self,
        object1_id: ObjectId,
        object2_id: ObjectId,
        points: Vec<ObjectId>,
    ) {
        let step = Step {
            construct: Construct::Point(Point::Intersect(
                object1_id, object2_id,
            )),
            objects: points,
        };
        self.steps.push(step);
    }

    pub fn add_line_line_intersection(
        &mut self,
        line1_id: LineId,
        line2_id: LineId,
    ) -> PointId {
        let point_id = self.get_new_object_id();
        self.add_intersection(line1_id.0, line2_id.0, vec![point_id]);
        PointId(point_id)
    }

    pub fn add_line_circle_intersection(
        &mut self,
        line_id: LineId,
        circle_id: CircleId,
    ) -> (PointId, PointId) {
        let point1_id = self.get_new_object_id();
        let point2_id = self.get_new_object_id();
        self.add_intersection(
            line_id.0,
            circle_id.0,
            vec![point1_id, point2_id],
        );
        (PointId(point1_id), PointId(point2_id))
    }

    pub fn add_circle_circle_intersection(
        &mut self,
        circle1_id: CircleId,
        circle2_id: CircleId,
    ) -> (PointId, PointId) {
        let point1_id = self.get_new_object_id();
        let point2_id = self.get_new_object_id();
        self.add_intersection(
            circle1_id.0,
            circle2_id.0,
            vec![point1_id, point2_id],
        );
        (PointId(point1_id), PointId(point2_id))
    }

    pub fn add_line(&mut self, point1: PointId, point2: PointId) -> LineId {
        let line_id = self.get_new_object_id();
        self.steps.push(Step {
            construct: Construct::Shape(Shape::Line(Line {
                point1: point1.0,
                point2: point2.0,
            })),
            objects: vec![line_id],
        });
        LineId(line_id)
    }

    pub fn add_circle(&mut self, center: PointId, point: PointId) -> CircleId {
        let circle_id = self.get_new_object_id();
        self.steps.push(Step {
            construct: Construct::Shape(Shape::Circle(Circle {
                center: center.0,
                point: point.0,
            })),
            objects: vec![circle_id],
        });
        CircleId(circle_id)
    }

    pub fn is_valid(&self) -> bool {
        // Check that the object dependencies form a DAG. For this
        // implementation, we require a stronger constraint: every object must
        // only reference an object of smaller ObjectId.
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

impl<'a> IntoIterator for &'a Construction {
    type Item = &'a Step;
    type IntoIter = Iter<'a, Step>;

    fn into_iter(self) -> Self::IntoIter {
        self.steps.iter()
    }
}
