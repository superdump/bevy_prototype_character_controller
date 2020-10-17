use crate::{controller::*, events::*};
use bevy::prelude::*;
use bevy_prototype_physx::*;

pub struct KinematicYawTag;

pub struct PhysXKinematicTranslationCharacterControllerPlugin;

impl Plugin for PhysXKinematicTranslationCharacterControllerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(create_mass.system())
            .add_system(constrain_rotation.system())
            .add_system_to_stage_front(bevy::app::stage::PRE_UPDATE, body_to_velocity.system())
            .add_system_to_stage_front(
                bevy::app::stage::UPDATE,
                controller_to_physx_kinematic_yaw.system(),
            )
            .add_system_to_stage_front(bevy::app::stage::UPDATE, controller_to_pitch.system());
    }
}

pub struct PhysXDynamicImpulseCharacterControllerPlugin;

impl Plugin for PhysXDynamicImpulseCharacterControllerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(create_mass.system())
            .add_system(constrain_rotation.system())
            .add_system_to_stage_front(bevy::app::stage::PRE_UPDATE, body_to_velocity.system())
            // IMPORTANT: The impulse/force systems MUST run before the physics simulation step, so they
            // either need to be added to the end of PRE_UPDATE or the beginning of UPDATE
            .add_system_to_stage_front(
                bevy::app::stage::UPDATE,
                controller_to_physx_dynamic_impulse.system(),
            )
            .add_system_to_stage_front(
                bevy::app::stage::UPDATE,
                controller_to_physx_dynamic_yaw.system(),
            )
            .add_system_to_stage_front(bevy::app::stage::UPDATE, controller_to_pitch.system());
    }
}
pub struct PhysXDynamicForceCharacterControllerPlugin;

impl Plugin for PhysXDynamicForceCharacterControllerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(create_mass.system())
            .add_system(constrain_rotation.system())
            .add_system_to_stage_front(bevy::app::stage::PRE_UPDATE, body_to_velocity.system())
            // IMPORTANT: The impulse/force systems MUST run before the physics simulation step, so they
            // either need to be added to the end of PRE_UPDATE or the beginning of UPDATE
            .add_system_to_stage_front(
                bevy::app::stage::UPDATE,
                controller_to_physx_dynamic_force.system(),
            )
            .add_system_to_stage_front(
                bevy::app::stage::UPDATE,
                controller_to_physx_dynamic_yaw.system(),
            )
            .add_system_to_stage_front(bevy::app::stage::UPDATE, controller_to_pitch.system());
    }
}

pub struct ConstrainedTag;

pub fn constrain_rotation(
    mut commands: Commands,
    mut physx: ResMut<PhysX>,
    mut query: Query<
        Without<ConstrainedTag, With<BodyTag, (Entity, &PhysXDynamicRigidBodyHandle)>>,
    >,
) {
    for (entity, body_handle) in &mut query.iter() {
        let mut body = physx
            .scene
            .get_dynamic_mut(body_handle.0)
            .expect("Failed to get dynamic rigid body");
        body.set_mass_space_inertia_tensor(Vec3::zero());
        commands.insert_one(entity, ConstrainedTag);
    }
}

pub fn create_mass(
    mut commands: Commands,
    physx: Res<PhysX>,
    mut query: Query<Without<Mass, (Entity, &PhysXDynamicRigidBodyHandle)>>,
) {
    for (entity, body_handle) in &mut query.iter() {
        let body = physx
            .scene
            .get_dynamic(body_handle.0)
            .expect("Failed to get dynamic rigid body");
        commands.insert_one(entity, Mass::new(body.get_mass()));
    }
}

pub fn body_to_velocity(
    physx: Res<PhysX>,
    _body: &BodyTag,
    body_handle: &PhysXDynamicRigidBodyHandle,
    mut controller: Mut<CharacterController>,
) {
    let body = physx
        .scene
        .get_dynamic(body_handle.0)
        .expect("Failed to get dynamic rigid body");
    controller.velocity = body.get_linear_velocity();
}

pub fn controller_to_physx_dynamic_impulse(
    impulses: Res<Events<ImpulseEvent>>,
    mut reader: ResMut<ControllerEvents>,
    mut physx: ResMut<PhysX>,
    _body: &BodyTag,
    body_handle: &PhysXDynamicRigidBodyHandle,
) {
    let mut impulse = Vec3::zero();
    for event in reader.impulses.iter(&impulses) {
        impulse += **event;
    }

    if impulse.length_squared() > 1E-6 {
        let mut body = physx
            .scene
            .get_dynamic_mut(body_handle.0)
            .expect("Failed to get dynamic rigid body");
        body.add_force(impulse, physx::rigid_body::ForceMode::Impulse, true);
    }
}

pub fn controller_to_physx_dynamic_force(
    forces: Res<Events<ForceEvent>>,
    mut reader: ResMut<ControllerEvents>,
    mut physx: ResMut<PhysX>,
    _body: &BodyTag,
    body_handle: &PhysXDynamicRigidBodyHandle,
) {
    let mut force = Vec3::zero();
    for event in reader.forces.iter(&forces) {
        force += **event;
    }

    if force.length_squared() > 1E-6 {
        let mut body = physx
            .scene
            .get_dynamic_mut(body_handle.0)
            .expect("Failed to get dynamic rigid body");
        body.add_force(force, physx::rigid_body::ForceMode::Force, true);
    }
}

pub fn controller_to_physx_kinematic_yaw(
    mut reader: ResMut<ControllerEvents>,
    yaws: Res<Events<YawEvent>>,
    _yaw: &KinematicYawTag,
    mut transform: Mut<Transform>,
) {
    let mut yaw = None;
    for event in reader.yaws.iter(&yaws) {
        yaw = Some(**event);
    }
    if let Some(yaw) = yaw {
        transform.set_rotation(Quat::from_rotation_y(yaw));
    }
}

pub fn controller_to_physx_dynamic_yaw(
    mut reader: ResMut<ControllerEvents>,
    yaws: Res<Events<YawEvent>>,
    mut physx: ResMut<PhysX>,
    _body: &BodyTag,
    body_handle: &PhysXDynamicRigidBodyHandle,
) {
    let mut yaw = None;
    for event in reader.yaws.iter(&yaws) {
        yaw = Some(**event);
    }
    if let Some(yaw) = yaw {
        let mut body = physx
            .scene
            .get_dynamic_mut(body_handle.0)
            .expect("Failed to get dynamic rigid body");
        let translation = body.get_global_pose().w_axis().truncate().into();
        body.set_global_pose(
            Mat4::from_rotation_translation(Quat::from_rotation_y(yaw), translation),
            true,
        );
    }
}
