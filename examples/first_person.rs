use bevy::prelude::*;

#[path = "../example_utils/utils.rs"]
mod utils;
use utils::{
    build_app, controller_to_kinematic, controller_to_pitch, controller_to_yaw, CharacterSettings,
};

fn main() {
    let mut app = App::build();
    build_app(&mut app);
    app.add_resource(CharacterSettings {
        focal_point: -Vec3::unit_z(), // Relative to head
        follow_offset: Vec3::zero(),  // Relative to head
        ..Default::default()
    })
    .add_system(controller_to_kinematic.system())
    .add_system(controller_to_yaw.system())
    .add_system(controller_to_pitch.system())
    .run();
}
