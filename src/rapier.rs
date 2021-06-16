use crate::{controller::*, events::*};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct RapierDynamicImpulseCharacterControllerPlugin;

pub const BODY_TO_VELOCITY_SYSTEM: &str = "body_to_velocity";
pub const CONTROLLER_TO_RAPIER_DYNAMIC_IMPULSE_SYSTEM: &str =
    "controller_to_rapier_dynamic_impulse";
pub const CONTROLLER_TO_RAPIER_DYNAMIC_FORCE_SYSTEM: &str = "controller_to_rapier_dynamic_force";
pub const CREATE_MASS_FROM_RAPIER_SYSTEM: &str = "create_mass_from_rapier";
pub const TOGGLE_FLY_MODE_SYSTEM: &str = "toggle_fly_mode";

impl Plugin for RapierDynamicImpulseCharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(CharacterControllerPlugin)
            .add_system_to_stage(
                CoreStage::PreUpdate,
                body_to_velocity
                    .system()
                    .label(BODY_TO_VELOCITY_SYSTEM)
                    .before(INPUT_TO_EVENTS_SYSTEM),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                toggle_fly_mode
                    .system()
                    .label(TOGGLE_FLY_MODE_SYSTEM)
                    .after(INPUT_TO_EVENTS_SYSTEM),
            )
            // NOTE: This must come after the bevy_rapier3d finalize_collider_attach_to_bodies system
            .add_system_to_stage(
                CoreStage::PreUpdate,
                create_mass_from_rapier
                    .system()
                    .label(CREATE_MASS_FROM_RAPIER_SYSTEM)
                    .after(bevy_rapier3d::physics::PhysicsSystems::FinalizeColliderAttachToBodies),
            )
            .add_system(
                controller_to_rapier_dynamic_impulse
                    .system()
                    .label(CONTROLLER_TO_RAPIER_DYNAMIC_IMPULSE_SYSTEM)
                    .before(bevy_rapier3d::physics::PhysicsSystems::StepWorld),
            )
            .add_system(controller_to_yaw.system())
            .add_system(controller_to_pitch.system());
    }
}

pub struct RapierDynamicForceCharacterControllerPlugin;

impl Plugin for RapierDynamicForceCharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(CharacterControllerPlugin)
            .add_system_to_stage(
                CoreStage::PreUpdate,
                body_to_velocity
                    .system()
                    .label(BODY_TO_VELOCITY_SYSTEM)
                    .before(INPUT_TO_EVENTS_SYSTEM),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                toggle_fly_mode
                    .system()
                    .label(TOGGLE_FLY_MODE_SYSTEM)
                    .after(INPUT_TO_EVENTS_SYSTEM),
            )
            // NOTE: This must come after the bevy_rapier3d finalize_collider_attach_to_bodies system
            .add_system_to_stage(
                CoreStage::PreUpdate,
                create_mass_from_rapier
                    .system()
                    .label(CREATE_MASS_FROM_RAPIER_SYSTEM)
                    .after(bevy_rapier3d::physics::PhysicsSystems::FinalizeColliderAttachToBodies),
            )
            .add_system(
                controller_to_rapier_dynamic_force
                    .system()
                    .label(CONTROLLER_TO_RAPIER_DYNAMIC_FORCE_SYSTEM)
                    .before(bevy_rapier3d::physics::PhysicsSystems::StepWorld),
            )
            .add_system(controller_to_yaw.system())
            .add_system(controller_to_pitch.system());
    }
}

pub fn create_mass_from_rapier(
    mut commands: Commands,
    query: Query<(Entity, &RigidBodyMassProps), Without<Mass>>,
) {
    for (entity, mass_props) in query.iter() {
        let mass = 1.0 / mass_props.effective_inv_mass;
        commands.entity(entity).insert(Mass::new(mass));
    }
}

pub fn body_to_velocity(
    mut query: Query<(&RigidBodyVelocity, &mut CharacterController), With<BodyTag>>,
) {
    for (velocity, mut controller) in query.iter_mut() {
        controller.velocity = velocity.linvel.into();
    }
}

pub fn controller_to_rapier_dynamic_impulse(
    mut impulses: EventReader<ImpulseEvent>,
    mut query: Query<
        (
            &mut RigidBodyVelocity,
            &mut RigidBodyActivation,
            &RigidBodyMassProps,
        ),
        With<BodyTag>,
    >,
) {
    let mut impulse = Vec3::ZERO;
    for event in impulses.iter() {
        impulse += **event;
    }
    if impulse.length_squared() > 1E-6 {
        for (mut velocity, mut activation, mass_props) in query.iter_mut() {
            velocity.apply_impulse(mass_props, impulse.into());
            activation.wake_up(true);
        }
    }
}

pub fn controller_to_rapier_dynamic_force(
    mut forces: EventReader<ForceEvent>,
    mut query: Query<(&mut RigidBodyForces, &mut RigidBodyActivation), With<BodyTag>>,
) {
    let mut force = Vec3::ZERO;
    for event in forces.iter() {
        force += **event;
    }

    if force.length_squared() > 1E-6 {
        for (mut forces, mut activation) in query.iter_mut() {
            forces.force = force.into();
            activation.wake_up(true);
        }
    }
}

const NO_GRAVITY: [f32; 3] = [0.0, 0.0, 0.0];
const GRAVITY: [f32; 3] = [0.0, -9.81, 0.0];

fn toggle_fly_mode(
    keyboard_input: Res<Input<KeyCode>>,
    mut rapier_config: ResMut<RapierConfiguration>,
    mut query: Query<(&CharacterController, &mut ColliderFlags)>,
) {
    for (controller, mut collider_flags) in query.iter_mut() {
        if keyboard_input.just_pressed(controller.input_map.key_fly) {
            rapier_config.gravity = if controller.fly {
                collider_flags.collision_groups = InteractionGroups::none();
                NO_GRAVITY.into()
            } else {
                collider_flags.collision_groups = InteractionGroups::default();
                GRAVITY.into()
            };
        }
    }
}
