use bevy::prelude::*;

// Take a look at example_utils/utils.rs for details!
#[path = "../example_utils/utils.rs"]
mod utils;
use utils::{
    build_app, controller_to_kinematic, controller_to_pitch, controller_to_yaw, CharacterSettings,
};

fn main() {
    let mut app = App::build();
    build_app(&mut app);
    app.init_resource::<CharacterSettings>()
        .add_system(controller_to_kinematic.system())
        .add_system(controller_to_yaw.system())
        .add_system(controller_to_pitch.system())
        .run();
}
