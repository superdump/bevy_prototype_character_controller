use bevy::{app::Events, prelude::*};
use bevy_prototype_character_controller::events::{
    ControllerEvents, ForceEvent, ImpulseEvent, LookDeltaEvent, LookEvent, PitchEvent,
    TranslationEvent, YawEvent,
};

// Take a look at example_utils/utils.rs for details!
#[path = "../example_utils/utils.rs"]
mod utils;
use utils::{build_app, CharacterSettings};

fn main() {
    let mut app = App::build();
    build_app(&mut app);
    app.init_resource::<CharacterSettings>()
        .add_system(print_controller_events.system())
        .run();
}

fn print_controller_events(
    mut reader: ResMut<ControllerEvents>,
    translations: Res<Events<TranslationEvent>>,
    impulses: Res<Events<ImpulseEvent>>,
    forces: Res<Events<ForceEvent>>,
    pitches: Res<Events<PitchEvent>>,
    yaws: Res<Events<YawEvent>>,
    looks: Res<Events<LookEvent>>,
    look_deltas: Res<Events<LookDeltaEvent>>,
) {
    for event in reader.translations.iter(&translations) {
        println!("{:?}", event);
    }
    for event in reader.impulses.iter(&impulses) {
        println!("{:?}", event);
    }
    for event in reader.forces.iter(&forces) {
        println!("{:?}", event);
    }
    for event in reader.pitches.iter(&pitches) {
        println!("{:?}", event);
    }
    for event in reader.yaws.iter(&yaws) {
        println!("{:?}", event);
    }
    for event in reader.looks.iter(&looks) {
        println!("{:?}", event);
    }
    for event in reader.look_deltas.iter(&look_deltas) {
        println!("{:?}", event);
    }
}
