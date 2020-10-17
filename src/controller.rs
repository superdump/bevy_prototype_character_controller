/*
 * First-order character controller
 *
 * Directly manipulate the position of the character through translations.
 */

use crate::{
    events::{
        ForceEvent, ImpulseEvent, LookDeltaEvent, LookEvent, PitchEvent, TranslationEvent, YawEvent,
    },
    input_map::InputMap,
    look::{forward_up, input_to_look, LookDirection, MouseMotionState, MouseSettings},
};
use bevy::prelude::*;
use std::{collections::HashMap, ops};

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
            .init_resource::<ControllerToLook>()
            .init_resource::<MouseMotionState>()
            .init_resource::<MouseSettings>()
            .add_system_to_stage_front(bevy::app::stage::PRE_UPDATE, input_to_events.system())
            .add_system_to_stage_front(bevy::app::stage::PRE_UPDATE, input_to_look.system())
            .add_system_to_stage_front(bevy::app::stage::PRE_UPDATE, forward_up.system())
            .add_system_to_stage_front(
                bevy::app::stage::PRE_UPDATE,
                controller_to_look_direction.thread_local_system(),
            );
    }
}

pub struct CharacterController {
    pub input_map: InputMap,
    pub fly: bool,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub jump_speed: f32,
    pub velocity: Vec3,
    pub jumping: bool,
}

impl Default for CharacterController {
    fn default() -> Self {
        Self {
            input_map: InputMap::default(),
            fly: false,
            walk_speed: 3.0,
            run_speed: 6.0,
            jump_speed: 6.0,
            velocity: Vec3::zero(),
            jumping: false,
        }
    }
}

fn get_look_entity_for_entity(world: &World, entity: Entity) -> Option<Entity> {
    if world.get::<LookDirection>(entity).is_ok() {
        return Some(entity);
    }
    if let Ok(children) = world.get::<Children>(entity) {
        for child in children.iter() {
            let look = get_look_entity_for_entity(world, *child);
            if look.is_some() {
                return look;
            }
        }
    }
    None
}

#[derive(Default)]
pub struct ControllerToLook {
    map: HashMap<Entity, Entity>,
}

impl ops::Deref for ControllerToLook {
    type Target = HashMap<Entity, Entity>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl ops::DerefMut for ControllerToLook {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
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

pub fn controller_to_look_direction(world: &mut World, resources: &mut Resources) {
    let mut controller_to_look = resources
        .get_mut::<ControllerToLook>()
        .expect("Could not get ControllerToLook resource!");
    let mut query = world.query::<(Entity, &CharacterController)>();

    for (entity, _controller) in &mut query.iter() {
        if controller_to_look.contains_key(&entity) {
            continue;
        }
        let look_entity =
            get_look_entity_for_entity(world, entity).expect("Failed to get LookDirection");
        controller_to_look.insert(entity, look_entity);
    }
}

pub fn input_to_events(
    controller_to_look: Res<ControllerToLook>,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut events: ResMut<Events<TranslationEvent>>,
    mut controller_query: Query<(Entity, &mut CharacterController)>,
    look_direction_query: Query<&LookDirection>,
) {
    for (entity, mut controller) in &mut controller_query.iter() {
        let look_entity = controller_to_look
            .get(&entity)
            .expect("Failed to look up LookDirection");
        let look = look_direction_query
            .get::<LookDirection>(*look_entity)
            .expect("Failed to get LookDirection from Entity");
        // Calculate forward / right / up vectors
        let (forward, right, _up) = if controller.fly {
            (look.forward, look.right, look.up)
        } else {
            let xz = Vec3::new(1.0, 0.0, 1.0);
            (
                (look.forward * xz).normalize(),
                (look.right * xz).normalize(),
                Vec3::unit_y(),
            )
        };

        // Calculate velocity based on input
        let mut velocity = Vec3::zero();
        if keyboard_input.pressed(controller.input_map.key_forward) {
            velocity += forward;
        }
        if keyboard_input.pressed(controller.input_map.key_backward) {
            velocity -= forward;
        }
        if keyboard_input.pressed(controller.input_map.key_right) {
            velocity += right;
        }
        if keyboard_input.pressed(controller.input_map.key_left) {
            velocity -= right;
        }

        if velocity.length_squared() > 1E-6 {
            controller.velocity = velocity.normalize() * Vec3::new(1.0, 0.0, 1.0)
                + controller.velocity * Vec3::unit_y();
        } else {
            controller.velocity *= Vec3::new(0.5, 1.0, 0.5);
        }

        let speed = if keyboard_input.pressed(controller.input_map.key_run) {
            controller.run_speed
        } else {
            controller.walk_speed
        };

        *controller.velocity.y_mut() = if controller.jumping {
            (-9.81f32).mul_add(time.delta_seconds, controller.velocity.y())
        } else if keyboard_input.just_pressed(controller.input_map.key_jump) {
            controller.jumping = true;
            controller.jump_speed
        } else {
            0.0
        };

        let translation = (controller.velocity * Vec3::new(1.0, 0.0, 1.0) * speed
            + controller.velocity * Vec3::unit_y())
            * time.delta_seconds;
        if translation.length_squared() > 1E-6 {
            events.send(TranslationEvent::new(&translation));
        }
    }
}
