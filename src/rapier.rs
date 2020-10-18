use crate::{controller::*, events::*};
use bevy::prelude::*;
use bevy_rapier3d::{
    na,
    physics::RigidBodyHandleComponent,
    rapier::{
        dynamics::RigidBodySet,
        math::{Isometry, Vector},
    },
};

pub struct RapierDynamicImpulseCharacterControllerPlugin;

impl Plugin for RapierDynamicImpulseCharacterControllerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(CharacterControllerPlugin)
            .add_system(create_mass.system())
            .add_system(constrain_rotation.system())
            .add_system_to_stage_front(bevy::app::stage::PRE_UPDATE, body_to_velocity.system())
            .add_system_to_stage_front(
                bevy::app::stage::UPDATE,
                controller_to_rapier_dynamic_impulse.system(),
            )
            .add_system_to_stage_front(bevy::app::stage::UPDATE, controller_to_rapier_yaw.system())
            .add_system_to_stage_front(bevy::app::stage::UPDATE, controller_to_pitch.system());
    }
}

pub struct RapierDynamicForceCharacterControllerPlugin;

impl Plugin for RapierDynamicForceCharacterControllerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(CharacterControllerPlugin)
            .add_system(create_mass.system())
            .add_system(constrain_rotation.system())
            .add_system_to_stage_front(bevy::app::stage::PRE_UPDATE, body_to_velocity.system())
            .add_system_to_stage_front(
                bevy::app::stage::UPDATE,
                controller_to_rapier_dynamic_force.system(),
            )
            .add_system_to_stage_front(bevy::app::stage::UPDATE, controller_to_rapier_yaw.system())
            .add_system_to_stage_front(bevy::app::stage::UPDATE, controller_to_pitch.system());
    }
}

pub struct ConstrainedTag;

pub fn constrain_rotation(
    mut commands: Commands,
    mut bodies: ResMut<RigidBodySet>,
    mut query: Query<Without<ConstrainedTag, With<BodyTag, (Entity, &RigidBodyHandleComponent)>>>,
) {
    for (entity, body_handle) in &mut query.iter() {
        let mut body = bodies
            .get_mut(body_handle.handle())
            .expect("Failed to get RigidBody");
        body.mass_properties.inv_principal_inertia_sqrt.x = 0.0;
        body.mass_properties.inv_principal_inertia_sqrt.y = 0.0;
        body.mass_properties.inv_principal_inertia_sqrt.z = 0.0;
        commands.insert_one(entity, ConstrainedTag);
    }
}

pub fn create_mass(
    mut commands: Commands,
    bodies: Res<RigidBodySet>,
    mut query: Query<Without<Mass, (Entity, &RigidBodyHandleComponent)>>,
) {
    for (entity, body_handle) in &mut query.iter() {
        let body = bodies
            .get(body_handle.handle())
            .expect("Failed to get RigidBody");
        let mass = 1.0 / body.mass_properties.inv_mass;
        commands.insert_one(entity, Mass::new(mass));
    }
}

pub fn body_to_velocity(
    bodies: Res<RigidBodySet>,
    _body: &BodyTag,
    body_handle: &RigidBodyHandleComponent,
    mut controller: Mut<CharacterController>,
) {
    let body = bodies
        .get(body_handle.handle())
        .expect("Failed to get RigidBody");
    let velocity = body.linvel;
    controller.velocity = Vec3::new(velocity[0], velocity[1], velocity[2]);
}

pub fn controller_to_rapier_dynamic_impulse(
    impulses: Res<Events<ImpulseEvent>>,
    mut reader: ResMut<ControllerEvents>,
    mut bodies: ResMut<RigidBodySet>,
    _body: &BodyTag,
    body_handle: &RigidBodyHandleComponent,
) {
    let mut impulse = Vec3::zero();
    for event in reader.impulses.iter(&impulses) {
        impulse += **event;
    }

    if impulse.length_squared() > 1E-6 {
        let mut body = bodies
            .get_mut(body_handle.handle())
            .expect("Failed to get character body");
        body.wake_up(true);
        body.apply_impulse(Vector::new(impulse.x(), impulse.y(), impulse.z()));
    }
}

pub fn controller_to_rapier_dynamic_force(
    forces: Res<Events<ForceEvent>>,
    mut reader: ResMut<ControllerEvents>,
    mut bodies: ResMut<RigidBodySet>,
    _body: &BodyTag,
    body_handle: &RigidBodyHandleComponent,
) {
    let mut force = Vec3::zero();
    for event in reader.forces.iter(&forces) {
        force += **event;
    }

    if force.length_squared() > 1E-6 {
        let mut body = bodies
            .get_mut(body_handle.handle())
            .expect("Failed to get character body");
        body.wake_up(true);
        body.apply_force(Vector::new(force.x(), force.y(), force.z()));
    }
}

pub fn controller_to_rapier_yaw(
    mut reader: ResMut<ControllerEvents>,
    yaws: Res<Events<YawEvent>>,
    _yaw: &YawTag,
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
