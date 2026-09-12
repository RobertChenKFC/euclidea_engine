use std::collections::HashMap;
use crate::construction::{Construction, ObjectId, Object};
use crate::field::Field;

struct Assignments<F: Field> {
    assignments: HashMap<ObjectId, (F, F)>,
}

trait Resolver<F: Field> {
    fn choose_free_point(object_id: ObjectId) -> (F, F);
}

fn compute_intersection(object1: &Object, object2: &Object) -> ((F, F), (F, F)){

}

fn solve<F: Field, R: Resolver<F>>(construction: &Construction) -> Assignments<F> {
    let mut assignments = HashMap::new();
    for (object_id, object) in construction.into_iter().enumerate() {
        let object_id = ObjectId::new(object_id);
        match object {
            Object::Line(_, _) => {},
            Object::Circle(_, _) => {},
            Object::FreePoint => {
                assignments.insert(object_id, R::choose_free_point(object_id));
            },
            Object::BoundPoint(on_object) => {
                todo!("Finish me")
            },
            Object::Intersection(object1_id, object2_id) => {
                let object1 = construction.get_object(*object1_id);
                let object2 = construction.get_object(*object2_id);
                if Some((point1, point2)) = compute_intersection(object1, object2) {
                    assignments.insert(*object1_id, object1);
                    assignments.insert(*object2_id, object2);
                }
            },
        }
    }
    Assignments { assignments }
}