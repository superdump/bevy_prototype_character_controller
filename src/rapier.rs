use crate::{controller::*, events::*};
use bevy::prelude::*;
use bevy_rapier3d::{
    physics::RigidBodyHandleComponent,
    rapier::{dynamics::RigidBodySet, math::Vector},
};

pub struct RapierDynamicImpulseCharacterControllerPlugin;

pub const APPLY_INPUT: &str = "apply_input";
pub const UPDATE_VELOCITY: &str = "update_velocity";

impl Plugin for RapierDynamicImpulseCharacterControllerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(CharacterControllerPlugin)
            .add_system_to_stage(bevy::app::stage::PRE_UPDATE, create_mass.system())
            .add_stage_before(
                PROCESS_INPUT_EVENTS,
                UPDATE_VELOCITY,
                SystemStage::parallel(),
            )
            .add_system_to_stage(UPDATE_VELOCITY, body_to_velocity.system())
            .add_stage_after(PROCESS_INPUT_EVENTS, APPLY_INPUT, SystemStage::parallel())
            .add_system_to_stage(APPLY_INPUT, controller_to_rapier_dynamic_impulse.system())
            .add_system_to_stage(bevy::app::stage::UPDATE, controller_to_yaw.system())
            .add_system_to_stage(bevy::app::stage::UPDATE, controller_to_pitch.system());
    }
}

pub struct RapierDynamicForceCharacterControllerPlugin;

impl Plugin for RapierDynamicForceCharacterControllerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(CharacterControllerPlugin)
            .add_system_to_stage(bevy::app::stage::PRE_UPDATE, create_mass.system())
            .add_stage_before(
                PROCESS_INPUT_EVENTS,
                UPDATE_VELOCITY,
                SystemStage::parallel(),
            )
            .add_system_to_stage(UPDATE_VELOCITY, body_to_velocity.system())
            .add_stage_after(PROCESS_INPUT_EVENTS, APPLY_INPUT, SystemStage::parallel())
            .add_system_to_stage(APPLY_INPUT, controller_to_rapier_dynamic_force.system())
            .add_system_to_stage(bevy::app::stage::UPDATE, controller_to_yaw.system())
            .add_system_to_stage(bevy::app::stage::UPDATE, controller_to_pitch.system());
    }
}

pub fn create_mass(
    commands: &mut Commands,
    bodies: Res<RigidBodySet>,
    query: Query<(Entity, &RigidBodyHandleComponent), Without<Mass>>,
) {
    for (entity, body_handle) in &mut query.iter() {
        let body = bodies
            .get(body_handle.handle())
            .expect("Failed to get RigidBody");
        let mass = 1.0 / body.mass_properties().inv_mass;
        commands.insert_one(entity, Mass::new(mass));
    }
}

pub fn body_to_velocity(
    bodies: Res<RigidBodySet>,
    mut query: Query<(&RigidBodyHandleComponent, &mut CharacterController), With<BodyTag>>,
) {
    for (body_handle, mut controller) in query.iter_mut() {
        let body = bodies
            .get(body_handle.handle())
            .expect("Failed to get RigidBody");
        let velocity = body.linvel();
        controller.velocity = Vec3::new(velocity[0], velocity[1], velocity[2]);
    }
}

pub fn controller_to_rapier_dynamic_impulse(
    impulses: Res<Events<ImpulseEvent>>,
    mut reader: ResMut<ControllerEvents>,
    mut bodies: ResMut<RigidBodySet>,
    query: Query<&RigidBodyHandleComponent, With<BodyTag>>,
) {
    for body_handle in query.iter() {
        let mut impulse = Vec3::zero();
        for event in reader.impulses.iter(&impulses) {
            impulse += **event;
        }
        if impulse.length_squared() > 1E-6 {
            let body = bodies
                .get_mut(body_handle.handle())
                .expect("Failed to get character body");
            body.apply_impulse(Vector::new(impulse.x, impulse.y, impulse.z), true);
        }
    }
}

pub fn controller_to_rapier_dynamic_force(
    forces: Res<Events<ForceEvent>>,
    mut reader: ResMut<ControllerEvents>,
    mut bodies: ResMut<RigidBodySet>,
    query: Query<&RigidBodyHandleComponent, With<BodyTag>>,
) {
    let mut force = Vec3::zero();
    for event in reader.forces.iter(&forces) {
        force += **event;
    }

    if force.length_squared() > 1E-6 {
        for body_handle in query.iter() {
            let body = bodies
                .get_mut(body_handle.handle())
                .expect("Failed to get character body");
            body.apply_force(Vector::new(force.x, force.y, force.z), true);
        }
    }
}
