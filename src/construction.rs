use std::slice::Iter;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ObjectId(usize);

impl ObjectId {
    pub fn new(object_id: usize) -> ObjectId {
        ObjectId(object_id)
    }
}

pub enum Object {
    Line(ObjectId, ObjectId),
    Circle(ObjectId, ObjectId),
    FreePoint,
    BoundPoint(ObjectId),
    Intersection(ObjectId, ObjectId),
}

impl Object {
    fn get_deps(&self) -> Vec<ObjectId> {
        match self {
            Object::Line(point1, point2) => vec![*point1, *point2],
            Object::Circle(center, point) => vec![*center, *point],
            Object::FreePoint => vec![],
            Object::BoundPoint(object) => vec![*object],
            Object::Intersection(object1, object2) => vec![*object1, *object2],
        }
    }
}

pub struct Construction {
    objects: Vec<Object>
}

impl Construction {
    pub fn is_valid(&self) -> bool {
        // Check that the object dependencies form a DAG. For this
        // implementation, we require a stronger constraint: every object must
        // only reference an object of smaller ObjectId.
        for (object_id, object) in self.objects.iter().enumerate() {
            for dep in object.get_deps() {
                if dep.0 >= object_id {
                    return false;
                }
            }
        }
        true
    }

    pub fn get_object<'a>(&'a self, object_id: ObjectId) -> &'a Object {
        &self.objects[object_id.0]
    }
}

impl<'a> IntoIterator for &'a Construction {
    type Item = &'a Object;
    type IntoIter = Iter<'a, Object>;

    fn into_iter(self) -> Self::IntoIter {
        self.objects.iter()
    }
}