use bevy::{input::system::exit_on_esc_system, prelude::*};
use bevy_prototype_character_controller::{
    controller::{BodyTag, CameraTag, CharacterController, HeadTag},
    look::LookDirection,
    rapier::*,
};
use bevy_rapier3d::{
    physics::RapierPhysicsPlugin,
    rapier::{dynamics::RigidBodyBuilder, geometry::ColliderBuilder},
};
use clap::{arg_enum, value_t};
use rand::Rng;

// Take a look at example_utils/utils.rs for details!
#[path = "../example_utils/utils.rs"]
mod utils;
use utils::*;

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum ControllerType {
        DynamicImpulse,
        DynamicForce,
    }
}

fn main() {
    let matches = clap::App::new("Bevy Rapier 3D Character Controller")
        .arg(
            clap::Arg::from_usage("<type> Controller type. ")
                .possible_values(&ControllerType::variants())
                .case_insensitive(true)
                .default_value("DynamicForce"),
        )
        .get_matches();
    let controller_type =
        value_t!(matches.value_of("type"), ControllerType).unwrap_or(ControllerType::DynamicForce);

    let mut app = App::build();

    // Generic
    app.add_resource(ClearColor(Color::hex("101010").unwrap()))
        .add_resource(Msaa { samples: 4 })
        .add_default_plugins()
        .add_system(exit_on_esc_system.system())
        // Rapier
        .add_plugin(RapierPhysicsPlugin);

    // IMPORTANT: The impulse/force systems MUST run before the physics simulation step, so they
    // either need to be added to the end of PRE_UPDATE or the beginning of UPDATE
    if controller_type == ControllerType::DynamicImpulse {
        // Option A. Apply impulses (changes in momentum)
        app.add_plugin(RapierDynamicImpulseCharacterControllerPlugin);
    } else {
        // Option B. Apply forces (rate of change of momentum)
        app.add_plugin(RapierDynamicForceCharacterControllerPlugin);
    }

    // Specific to this demo
    app.init_resource::<CharacterSettings>()
        .add_startup_system(spawn_world.system())
        .add_startup_system(spawn_character.system())
        .run();
}

pub fn spawn_world(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let cube = meshes.add(Mesh::from(shape::Cube { size: 0.5 }));

    // Light
    commands.spawn(LightComponents {
        transform: Transform::from_translation(Vec3::new(-15.0, 10.0, -15.0)),
        ..Default::default()
    });

    // Ground cuboid
    let grey = materials.add(Color::hex("808080").unwrap().into());
    let box_xz = 200.0;
    let box_y = 1.0;
    commands
        .spawn(PbrComponents {
            material: grey,
            mesh: cube,
            transform: Transform::new(Mat4::from_scale_rotation_translation(
                Vec3::new(box_xz, box_y, box_xz),
                Quat::identity(),
                Vec3::zero(),
            )),
            ..Default::default()
        })
        .with_bundle((
            RigidBodyBuilder::new_static(),
            ColliderBuilder::cuboid(0.5 * box_xz, 0.5 * box_y, 0.5 * box_xz),
        ));

    // Cubes for some kind of reference in the scene to make it easy to see
    // what is happening
    let teal = materials.add(Color::hex("008080").unwrap().into());
    let cube_scale = 1.0;
    let mut rng = rand::thread_rng();
    for _ in 0..20 {
        let x = rng.gen_range(-10.0, 10.0);
        let z = rng.gen_range(-10.0, 10.0);
        commands
            .spawn(PbrComponents {
                material: teal,
                mesh: cube,
                transform: Transform::from_translation_rotation_scale(
                    Vec3::new(x, 0.5 * (cube_scale - box_y), z),
                    Quat::identity(),
                    cube_scale,
                ),
                ..Default::default()
            })
            .with_bundle((
                RigidBodyBuilder::new_dynamic().translation(x, 0.5 * (cube_scale - box_y), z),
                ColliderBuilder::cuboid(0.5 * cube_scale, 0.5 * cube_scale, 0.5 * cube_scale),
            ));
    }
}

pub fn spawn_character(
    mut commands: Commands,
    character_settings: Res<CharacterSettings>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let box_y = 1.0;
    let cube = meshes.add(Mesh::from(shape::Cube { size: 0.5 }));
    // Character
    let red = materials.add(Color::hex("800000").unwrap().into());
    commands
        .spawn((
            GlobalTransform::identity(),
            Transform::identity(),
            CharacterController::default(),
            RigidBodyBuilder::new_dynamic().translation(
                0.0,
                0.5 * (box_y + character_settings.scale.y()),
                0.0,
            ),
            ColliderBuilder::capsule_y(
                0.5 * character_settings.scale.y(),
                0.5 * character_settings
                    .scale
                    .x()
                    .max(character_settings.scale.z()),
            )
            .density(200.0),
            BodyTag,
        ))
        .with_children(|body| {
            body.spawn(PbrComponents {
                material: red,
                mesh: cube,
                transform: Transform::new(Mat4::from_scale_rotation_translation(
                    character_settings.scale - character_settings.head_scale * Vec3::unit_y(),
                    Quat::identity(),
                    Vec3::new(0.0, -0.5 * character_settings.head_scale, 0.0),
                )),
                ..Default::default()
            })
            .spawn((
                GlobalTransform::identity(),
                Transform::from_translation_rotation(
                    Vec3::new(
                        0.0,
                        0.5 * (character_settings.scale.y() - character_settings.head_scale),
                        0.0,
                    ),
                    Quat::from_rotation_y(character_settings.head_yaw),
                ),
                HeadTag,
            ))
            .with_children(|head| {
                head.spawn(PbrComponents {
                    material: red,
                    mesh: cube,
                    transform: Transform::from_scale(character_settings.head_scale),
                    ..Default::default()
                })
                .spawn(Camera3dComponents {
                    transform: Transform::new(Mat4::face_toward(
                        character_settings.follow_offset,
                        character_settings.focal_point,
                        Vec3::unit_y(),
                    )),
                    ..Default::default()
                })
                .with(LookDirection::default())
                .with(CameraTag);
            });
        });
}
