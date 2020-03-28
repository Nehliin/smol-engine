use crate::components::{PhysicsBody, Transform};

use legion::prelude::*;
use nalgebra::Vector3;
use ncollide3d::shape::{Cuboid, ShapeHandle};
use nphysics3d::force_generator::DefaultForceGeneratorSet;
use nphysics3d::joint::DefaultJointConstraintSet;
use nphysics3d::object::{
    BodyPartHandle, BodyStatus, ColliderDesc, DefaultBodySet, DefaultColliderSet, RigidBodyDesc,
};
use nphysics3d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};

// absolute shite this is
pub struct Physics {
    pub system: Box<dyn Schedulable>,
}

impl Physics {
    pub fn new(resources: &mut Resources) -> Self {
        let mechanical_world: DefaultMechanicalWorld<f32> =
            DefaultMechanicalWorld::new(Vector3::y() * -9.81);
        let geometrical_world: DefaultGeometricalWorld<f32> = DefaultGeometricalWorld::new();
        let body_set: DefaultBodySet<f32> = DefaultBodySet::new();
        let force_gen_set: DefaultForceGeneratorSet<f32> = DefaultForceGeneratorSet::new();
        let joint_set: DefaultJointConstraintSet<f32> = DefaultJointConstraintSet::new();
        let collider_set: DefaultColliderSet<f32> = DefaultColliderSet::new();
        resources.insert(mechanical_world);
        resources.insert(geometrical_world);
        resources.insert(body_set);
        resources.insert(force_gen_set);
        resources.insert(joint_set);
        resources.insert(collider_set);
        let system = SystemBuilder::new("physics-system")
            .write_resource::<DefaultMechanicalWorld<f32>>()
            .write_resource::<DefaultGeometricalWorld<f32>>()
            .write_resource::<DefaultJointConstraintSet<f32>>()
            .write_resource::<DefaultForceGeneratorSet<f32>>()
            .write_resource::<DefaultBodySet<f32>>()
            .write_resource::<DefaultColliderSet<f32>>()
            .with_query(<(Read<PhysicsBody>, Write<Transform>)>::query())
            .build(
                |_,
                 world,
                 (mech_world, geo_world, joint_set, force_gen, body_set, collider_set),
                 query| {
                    mech_world.step(
                        geo_world,
                        body_set as &mut DefaultBodySet<f32>,
                        collider_set as &mut DefaultColliderSet<f32>,
                        joint_set as &mut DefaultJointConstraintSet<f32>,
                        force_gen as &mut DefaultForceGeneratorSet<f32>,
                    );
                    for (physics_body, mut transform) in query.iter_mut(world) {
                        if let Some(collider) = collider_set.get(physics_body.collider_handle) {
                            transform.position = collider.position().translation.vector;
                            //   dbg!(collider.position().rotation);
                            transform.rotation = collider.position().rotation.as_vector().xyz();
                        }
                    }
                },
            );
        Physics { system }
    }

    pub fn create_cube(
        resources: &mut Resources,
        transform: &Transform,
        body_status: BodyStatus,
    ) -> PhysicsBody {
        let cube_body = RigidBodyDesc::new()
            .translation(Vector3::new(
                transform.position.x,
                transform.position.y,
                transform.position.z,
            ))
            .rotation(Vector3::new(
                transform.rotation.x,
                transform.rotation.y,
                transform.rotation.z,
            ))
            .status(body_status)
            .mass(5.0)
            .build();

        let mut body_set = resources
            .get_mut::<DefaultBodySet<f32>>()
            .expect("Default body set not added as a resource");
        let body_handle = body_set.insert(cube_body);

        let shape = ShapeHandle::new(Cuboid::new(Vector3::new(
            transform.scale.x,
            transform.scale.y,
            transform.scale.z,
        )));

        let collider = ColliderDesc::new(shape)
            .density(1.0)
            .build(BodyPartHandle(body_handle, 0));

        let mut collider_set = resources
            .get_mut::<DefaultColliderSet<f32>>()
            .expect("Collider set not added to resources yet");

        let collider_handle = collider_set.insert(collider);

        PhysicsBody {
            body_handle,
            collider_handle,
        }
    }
}
