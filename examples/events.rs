use bevy::prelude::*;
use bevy_prototype_character_controller::events::{
    ForceEvent, ImpulseEvent, LookDeltaEvent, LookEvent, PitchEvent, TranslationEvent, YawEvent,
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
    mut translations: EventReader<TranslationEvent>,
    mut impulses: EventReader<ImpulseEvent>,
    mut forces: EventReader<ForceEvent>,
    mut pitches: EventReader<PitchEvent>,
    mut yaws: EventReader<YawEvent>,
    mut looks: EventReader<LookEvent>,
    mut look_deltas: EventReader<LookDeltaEvent>,
) {
    for event in translations.iter() {
        println!("{:?}", event);
    }
    for event in impulses.iter() {
        println!("{:?}", event);
    }
    for event in forces.iter() {
        println!("{:?}", event);
    }
    for event in pitches.iter() {
        println!("{:?}", event);
    }
    for event in yaws.iter() {
        println!("{:?}", event);
    }
    for event in looks.iter() {
        println!("{:?}", event);
    }
    for event in look_deltas.iter() {
        println!("{:?}", event);
    }
}
