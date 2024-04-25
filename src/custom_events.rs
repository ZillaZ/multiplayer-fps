use crate::*;

pub fn handle_collision(
    manager: &mut GameManager,
    collision_recv: &Receiver<CollisionEvent>,
    contact_recv: &Receiver<ContactForceEvent>,
) {
    while let Ok(collision_event) = collision_recv.try_recv() {
        println!("inside!");
        if !collision_event.sensor() {
            continue;
        }
        let collider1 = collision_event.collider1();
        for tuple in manager.objects.iter() {
            if tuple.1 != collider1 {
                continue;
            }
            let access = manager.bodies.get_mut(tuple.2).unwrap();
            access.apply_impulse(vector![0.0, 99999.0, 0.0], false);
        }
    }
    while let Ok(_contact_event) = contact_recv.try_recv() {}
}
