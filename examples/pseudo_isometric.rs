use bevy::prelude::*;

// Take a look at example_utils/utils.rs for details!
#[path = "../example_utils/utils.rs"]
mod utils;
use utils::{build_app, controller_to_kinematic, CharacterSettings};

fn main() {
    let mut app = App::build();
    build_app(&mut app);
    app.add_resource(CharacterSettings {
        focal_point: Vec3::ZERO,
        follow_offset: Vec3::new(-10.0, 10.0, -10.0),
        head_yaw: 0.5 * std::f32::consts::TAU,
        ..Default::default()
    })
    .add_system(controller_to_kinematic.system())
    .run();
}
