// system that converts delta axis events into pitch and yaw
use crate::events::{LookDeltaEvent, LookEvent, PitchEvent, YawEvent};
use bevy::{input::mouse::MouseMotion, prelude::*};

#[derive(Clone, Copy)]
pub struct LookDirection {
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

impl Default for LookDirection {
    fn default() -> Self {
        Self {
            forward: Vec3::unit_z(),
            right: -Vec3::unit_x(),
            up: Vec3::unit_y(),
        }
    }
}

pub struct LookEntity(pub Entity);

pub fn forward_up(settings: Res<MouseSettings>, mut look: Mut<LookDirection>) {
    let rotation = Quat::from_rotation_ypr(
        settings.yaw_pitch_roll.x(),
        settings.yaw_pitch_roll.y(),
        settings.yaw_pitch_roll.z(),
    );
    look.forward = rotation * -Vec3::unit_z();
    look.right = rotation * Vec3::unit_x();
    look.up = rotation * Vec3::unit_y();
}

pub struct MouseSettings {
    pub sensitivity: f32,
    pub yaw_pitch_roll: Vec3,
}

impl Default for MouseSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.01,
            yaw_pitch_roll: Vec3::zero(),
        }
    }
}

#[derive(Default)]
pub struct MouseMotionState {
    event_reader: EventReader<MouseMotion>,
}

const PITCH_BOUND: f32 = std::f32::consts::FRAC_PI_2 - 1E-3;

pub fn input_to_look(
    mouse_motion_events: Res<Events<MouseMotion>>,
    mut settings: ResMut<MouseSettings>,
    mut mouse_motion: ResMut<MouseMotionState>,
    mut pitch_events: ResMut<Events<PitchEvent>>,
    mut yaw_events: ResMut<Events<YawEvent>>,
    mut look_events: ResMut<Events<LookEvent>>,
    mut look_delta_events: ResMut<Events<LookDeltaEvent>>,
) {
    let mut delta = Vec2::zero();
    for motion in mouse_motion.event_reader.iter(&mouse_motion_events) {
        // NOTE: -= to invert
        delta -= motion.delta;
    }
    if delta.length_squared() > 1E-6 {
        delta *= settings.sensitivity;
        settings.yaw_pitch_roll += delta.extend(0.0);
        if settings.yaw_pitch_roll.y() > PITCH_BOUND {
            *settings.yaw_pitch_roll.y_mut() = PITCH_BOUND;
        }
        if settings.yaw_pitch_roll.y() < -PITCH_BOUND {
            *settings.yaw_pitch_roll.y_mut() = -PITCH_BOUND;
        }
        look_delta_events.send(LookDeltaEvent::new(&delta.extend(0.0)));
        look_events.send(LookEvent::new(&settings.yaw_pitch_roll));
        pitch_events.send(PitchEvent::new(settings.yaw_pitch_roll.y()));
        yaw_events.send(YawEvent::new(settings.yaw_pitch_roll.x()));
    }
}
