use bevy::{input::system::exit_on_esc_system, prelude::*};
use bevy_prototype_character_controller::{
    events::{LookDeltaEvent, LookEvent, PitchEvent, TranslationEvent, YawEvent},
    look::LookDirection,
    translation::{TranslationController, TranslationControllerPlugin},
};

fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_default_plugins()
        .add_plugin(TranslationControllerPlugin)
        .init_resource::<ControllerEvents>()
        .add_system(exit_on_esc_system.system())
        .add_startup_system(spawn_world.system())
        .add_system(print_controller_events.system())
        .run();
}

fn spawn_world(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let grey = materials.add(Color::hex("808080").unwrap().into());
    let red = materials.add(Color::hex("800000").unwrap().into());
    let cube = meshes.add(Mesh::from(shape::Cube::default()));
    commands
        .spawn(LightComponents {
            transform: Transform::from_translation(Vec3::new(-15.0, 10.0, -15.0)),
            ..Default::default()
        })
        .spawn(Camera3dComponents {
            transform: Transform::new(Mat4::face_toward(
                Vec3::new(-15.0, 10.0, -15.0),
                Vec3::new(5.0, 0.0, 5.0),
                Vec3::unit_y(),
            )),
            ..Default::default()
        })
        .spawn(PbrComponents {
            material: grey,
            mesh: cube,
            transform: Transform::new(Mat4::from_scale_rotation_translation(
                Vec3::new(10.0, 1.0, 10.0),
                Quat::identity(),
                Vec3::new(0.0, -0.5, 0.0),
            )),
            ..Default::default()
        })
        .spawn(PbrComponents {
            material: red,
            mesh: cube,
            transform: Transform::new(Mat4::from_scale_rotation_translation(
                Vec3::new(1.0, 2.0, 1.0),
                Quat::identity(),
                Vec3::new(0.0, 0.5, 0.0),
            )),
            ..Default::default()
        })
        .with(TranslationController::default())
        .with(LookDirection::default());
}

#[derive(Default)]
struct ControllerEvents {
    translations: EventReader<TranslationEvent>,
    pitches: EventReader<PitchEvent>,
    yaws: EventReader<YawEvent>,
    looks: EventReader<LookEvent>,
    look_deltas: EventReader<LookDeltaEvent>,
}

fn print_controller_events(
    mut reader: ResMut<ControllerEvents>,
    translations: Res<Events<TranslationEvent>>,
    pitches: Res<Events<PitchEvent>>,
    yaws: Res<Events<YawEvent>>,
    looks: Res<Events<LookEvent>>,
    look_deltas: Res<Events<LookDeltaEvent>>,
) {
    for event in reader.translations.iter(&translations) {
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
