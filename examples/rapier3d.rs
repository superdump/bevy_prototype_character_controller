use bevy::{input::system::exit_on_esc_system, prelude::*};
use bevy_prototype_character_controller::{
    controller::{BodyTag, CameraTag, CharacterController, HeadTag, YawTag},
    look::{LookDirection, LookEntity},
    rapier::*,
};
use bevy_rapier3d::{
    physics::TimestepMode,
    prelude::{
        ColliderBundle, ColliderMassProps, ColliderShape, NoUserData, RapierConfiguration,
        RapierPhysicsPlugin, RigidBodyActivation, RigidBodyBundle, RigidBodyMassPropsFlags,
        RigidBodyPosition, RigidBodyPositionSync, RigidBodyType,
    },
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
    app.insert_resource(ClearColor(Color::hex("101010").unwrap()))
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_system(exit_on_esc_system.system())
        // Rapier
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(RapierConfiguration {
            timestep_mode: TimestepMode::InterpolatedTimestep,
            ..Default::default()
        });

    // IMPORTANT: The impulse/force systems MUST run before the physics simulation step, so they
    // either need to be added to the end of PRE_UPDATE or the beginning of UPDATE
    println!("Using {:?} method", controller_type);
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
    let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

    // Light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_translation(Vec3::new(-15.0, 10.0, -15.0)),
        ..Default::default()
    });

    // Ground cuboid
    let grey = materials.add(Color::hex("808080").unwrap().into());
    let box_xz = 200.0;
    let box_y = 1.0;
    commands
        .spawn_bundle(PbrBundle {
            material: grey,
            mesh: cube.clone(),
            transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(
                Vec3::new(box_xz, box_y, box_xz),
                Quat::IDENTITY,
                Vec3::ZERO,
            )),
            ..Default::default()
        })
        .insert_bundle(RigidBodyBundle {
            body_type: RigidBodyType::Static,
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(0.5 * box_xz, 0.5 * box_y, 0.5 * box_xz),
            ..Default::default()
        });

    // Cubes for some kind of reference in the scene to make it easy to see
    // what is happening
    let teal = materials.add(Color::hex("008080").unwrap().into());
    let cube_scale = 1.0;
    let mut rng = rand::thread_rng();
    for _ in 0..20 {
        let x = rng.gen_range(-10.0..10.0);
        let z = rng.gen_range(-10.0..10.0);
        let translation = Vec3::new(x, 0.5 * (cube_scale - box_y), z);
        commands
            .spawn_bundle(PbrBundle {
                material: teal.clone(),
                mesh: cube.clone(),
                transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(
                    Vec3::splat(cube_scale),
                    Quat::IDENTITY,
                    translation,
                )),
                ..Default::default()
            })
            .insert_bundle(RigidBodyBundle {
                activation: RigidBodyActivation {
                    sleeping: false,
                    ..Default::default()
                },
                body_type: RigidBodyType::Dynamic,
                position: RigidBodyPosition {
                    position: (translation + Vec3::new(x, 0.5 * (cube_scale - box_y), z)).into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert_bundle(ColliderBundle {
                shape: ColliderShape::cuboid(0.5 * cube_scale, 0.5 * cube_scale, 0.5 * cube_scale),
                ..Default::default()
            })
            .insert(RigidBodyPositionSync::Interpolated { prev_pos: None });
    }
}

pub fn spawn_character(
    mut commands: Commands,
    character_settings: Res<CharacterSettings>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let box_y = 1.0;
    let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let red = materials.add(Color::hex("800000").unwrap().into());
    let body = commands
        .spawn_bundle((
            GlobalTransform::identity(),
            Transform::identity(),
            CharacterController::default(),
            BodyTag,
        ))
        .insert_bundle(RigidBodyBundle {
            activation: RigidBodyActivation {
                sleeping: false,
                ..Default::default()
            },
            body_type: RigidBodyType::Dynamic,
            mass_properties: RigidBodyMassPropsFlags::ROTATION_LOCKED.into(),
            position: RigidBodyPosition {
                position: (0.5 * (box_y + character_settings.scale.y) * Vec3::Y).into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            mass_properties: ColliderMassProps::Density(200.0),
            shape: ColliderShape::capsule(
                (-0.5 * character_settings.scale.y * Vec3::Y).into(),
                (0.5 * character_settings.scale.y * Vec3::Y).into(),
                0.5 * character_settings.scale.x.max(character_settings.scale.z),
            ),
            ..Default::default()
        })
        .insert(RigidBodyPositionSync::Interpolated { prev_pos: None })
        .id();
    let yaw = commands
        .spawn_bundle((GlobalTransform::identity(), Transform::identity(), YawTag))
        .id();
    let body_model = commands
        .spawn_bundle(PbrBundle {
            material: red.clone(),
            mesh: cube.clone(),
            transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(
                character_settings.scale - character_settings.head_scale * Vec3::Y,
                Quat::IDENTITY,
                Vec3::new(
                    0.0,
                    0.5 * (box_y + character_settings.scale.y - character_settings.head_scale)
                        - 1.695,
                    0.0,
                ),
            )),
            ..Default::default()
        })
        .id();
    let head = commands
        .spawn_bundle((
            GlobalTransform::identity(),
            Transform::from_matrix(Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::from_rotation_y(character_settings.head_yaw),
                Vec3::new(
                    0.0,
                    0.5 * (box_y - character_settings.head_scale) + character_settings.scale.y
                        - 1.695,
                    0.0,
                ),
            )),
            HeadTag,
        ))
        .id();
    let head_model = commands
        .spawn_bundle(PbrBundle {
            material: red,
            mesh: cube,
            transform: Transform::from_scale(Vec3::splat(character_settings.head_scale)),
            ..Default::default()
        })
        .id();
    let camera = commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_matrix(Mat4::face_toward(
                character_settings.follow_offset,
                character_settings.focal_point,
                Vec3::Y,
            )),
            ..Default::default()
        })
        .insert_bundle((LookDirection::default(), CameraTag))
        .id();
    commands
        .entity(body)
        .insert(LookEntity(camera))
        .push_children(&[yaw]);
    commands.entity(yaw).push_children(&[body_model, head]);
    commands.entity(head).push_children(&[head_model, camera]);
}
