/*
 * First-order character controller
 *
 * Directly manipulate the position of the character through translations.
 */

use crate::{
    events::{
        ControllerEvents, ForceEvent, ImpulseEvent, LookDeltaEvent, LookEvent, PitchEvent,
        TranslationEvent, YawEvent,
    },
    input_map::InputMap,
    look::{forward_up, input_to_look, LookDirection, LookEntity, MouseMotionState, MouseSettings},
};
use bevy::prelude::*;

pub struct BodyTag;
pub struct YawTag;
pub struct HeadTag;
pub struct CameraTag;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<PitchEvent>()
            .add_event::<YawEvent>()
            .add_event::<LookEvent>()
            .add_event::<LookDeltaEvent>()
            .add_event::<TranslationEvent>()
            .add_event::<ImpulseEvent>()
            .add_event::<ForceEvent>()
            .init_resource::<ControllerEvents>()
            .init_resource::<MouseMotionState>()
            .init_resource::<MouseSettings>()
            .add_system_to_stage_front(bevy::app::stage::PRE_UPDATE, input_to_events.system())
            .add_system_to_stage_front(bevy::app::stage::PRE_UPDATE, input_to_look.system())
            .add_system_to_stage_front(bevy::app::stage::PRE_UPDATE, forward_up.system());
    }
}

#[derive(Default)]
pub struct InputState {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub run: bool,
    pub jump: bool,
}

pub struct CharacterController {
    pub input_map: InputMap,
    pub fly: bool,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub jump_speed: f32,
    pub velocity: Vec3,
    pub jumping: bool,
    pub dt: f32,
    pub sim_to_render: f32,
    pub input_state: InputState,
}

impl Default for CharacterController {
    fn default() -> Self {
        Self {
            input_map: InputMap::default(),
            fly: false,
            walk_speed: 5.0,
            run_speed: 8.0,
            jump_speed: 6.0,
            velocity: Vec3::zero(),
            jumping: false,
            dt: 1.0 / 60.0,
            sim_to_render: 0.0,
            input_state: InputState::default(),
        }
    }
}

pub struct Mass {
    pub mass: f32,
}

impl Mass {
    pub fn new(mass: f32) -> Self {
        Self { mass }
    }
}

pub fn input_to_events(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut translation_events: ResMut<Events<TranslationEvent>>,
    mut impulse_events: ResMut<Events<ImpulseEvent>>,
    mut force_events: ResMut<Events<ForceEvent>>,
    mut controller_query: Query<(&Mass, &LookEntity, &mut CharacterController)>,
    look_direction_query: Query<&LookDirection>,
) {
    let xz = Vec3::new(1.0, 0.0, 1.0);
    for (mass, look_entity, mut controller) in &mut controller_query.iter() {
        controller.sim_to_render += time.delta_seconds;

        if keyboard_input.pressed(controller.input_map.key_forward) {
            controller.input_state.forward = true;
        }
        if keyboard_input.pressed(controller.input_map.key_backward) {
            controller.input_state.backward = true;
        }
        if keyboard_input.pressed(controller.input_map.key_right) {
            controller.input_state.right = true;
        }
        if keyboard_input.pressed(controller.input_map.key_left) {
            controller.input_state.left = true;
        }
        if keyboard_input.pressed(controller.input_map.key_run) {
            controller.input_state.run = true;
        }
        if keyboard_input.just_pressed(controller.input_map.key_jump) {
            controller.input_state.jump = true;
        }

        if controller.sim_to_render < controller.dt {
            continue;
        }
        // Calculate the remaining simulation to render time after all
        // simulation steps were taken
        controller.sim_to_render %= controller.dt;

        let look = look_direction_query
            .get::<LookDirection>(look_entity.0)
            .expect("Failed to get LookDirection from Entity");

        // Calculate forward / right / up vectors
        let (forward, right, _up) = if controller.fly {
            (look.forward, look.right, look.up)
        } else {
            (
                (look.forward * xz).normalize(),
                (look.right * xz).normalize(),
                Vec3::unit_y(),
            )
        };

        // Calculate the desired velocity based on input
        let mut desired_velocity = Vec3::zero();
        if controller.input_state.forward {
            desired_velocity += forward;
        }
        if controller.input_state.backward {
            desired_velocity -= forward;
        }
        if controller.input_state.right {
            desired_velocity += right;
        }
        if controller.input_state.left {
            desired_velocity -= right;
        }

        // Limit x/z velocity to walk/run speed
        let speed = if controller.input_state.run {
            controller.run_speed
        } else {
            controller.walk_speed
        };
        desired_velocity = if desired_velocity.length_squared() > 1E-6 {
            desired_velocity.normalize() * speed
        } else {
            // No input - apply damping to the x/z of the current velocity
            controller.velocity * 0.5 * xz
        };

        // Handle jumping
        let was_jumping = controller.jumping;
        *desired_velocity.y_mut() = if controller.input_state.jump {
            controller.jumping = true;
            controller.jump_speed
        } else {
            0.0
        };

        // Calculate impulse - the desired momentum change for the time period
        let delta_velocity = desired_velocity - controller.velocity * xz;
        let impulse = delta_velocity * mass.mass;
        if impulse.length_squared() > 1E-6 {
            impulse_events.send(ImpulseEvent::new(&impulse));
        }

        // Calculate force - the desired rate of change of momentum for the time period
        let force = impulse / controller.dt;
        if force.length_squared() > 1E-6 {
            force_events.send(ForceEvent::new(&force));
        }

        *controller.velocity.x_mut() = desired_velocity.x();
        *controller.velocity.z_mut() = desired_velocity.z();
        *controller.velocity.y_mut() = if was_jumping {
            // Apply gravity for kinematic simulation
            (-9.81f32).mul_add(controller.dt, controller.velocity.y())
        } else {
            desired_velocity.y()
        };

        let translation = controller.velocity * controller.dt;
        if translation.length_squared() > 1E-6 {
            translation_events.send(TranslationEvent::new(&translation));
        }

        controller.input_state = InputState::default();
    }
}

pub fn controller_to_yaw(
    mut reader: ResMut<ControllerEvents>,
    yaws: Res<Events<YawEvent>>,
    _yaw: &YawTag,
    mut transform: Mut<Transform>,
) {
    for yaw in reader.yaws.iter(&yaws) {
        transform.set_rotation(Quat::from_rotation_ypr(**yaw, 0.0, 0.0));
    }
}

pub fn controller_to_pitch(
    mut reader: ResMut<ControllerEvents>,
    pitches: Res<Events<PitchEvent>>,
    _head: &HeadTag,
    mut transform: Mut<Transform>,
) {
    for pitch in reader.pitches.iter(&pitches) {
        transform.set_rotation(Quat::from_rotation_ypr(0.0, **pitch, 0.0));
    }
}
