use bevy::{input::system::exit_on_esc_system, prelude::*};
use bevy_prototype_character_controller::{
    controller::{
        BodyTag, CameraTag, CharacterController, CharacterControllerPlugin, HeadTag, Mass, YawTag,
    },
    events::{ControllerEvents, TranslationEvent},
    look::{LookDirection, LookEntity},
    physx::*,
};
use bevy_prototype_physx::*;
use clap::{arg_enum, value_t};
use rand::Rng;

// Take a look at example_utils/utils.rs for details!
#[path = "../example_utils/utils.rs"]
mod utils;
use utils::*;

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum ControllerType {
        KinematicTranslation,
        DynamicImpulse,
        DynamicForce,
    }
}

impl Default for ControllerType {
    fn default() -> Self {
        ControllerType::DynamicForce
    }
}

fn main() {
    let matches = clap::App::new("Bevy PhysX 3D Character Controller")
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
        // Character Controller
        .add_plugin(CharacterControllerPlugin)
        .init_resource::<ControllerEvents>()
        // PhysX
        .add_plugin(PhysXPlugin);
    // Character controller adaptations for PhysX
    if controller_type == ControllerType::KinematicTranslation {
        // Option A. Apply translations (changes in position)
        app.add_plugin(PhysXKinematicTranslationCharacterControllerPlugin)
            .add_system_to_stage(
                bevy::app::stage::UPDATE,
                controller_to_physx_kinematic.system(),
            );
    } else if controller_type == ControllerType::DynamicImpulse {
        // Option B. Apply impulses (changes in momentum)
        app.add_plugin(PhysXDynamicImpulseCharacterControllerPlugin);
    } else {
        // Option C. Apply forces (rate of change of momentum)
        app.add_plugin(PhysXDynamicForceCharacterControllerPlugin);
    }

    // Specific to this demo
    app.init_resource::<CharacterSettings>()
        .add_resource(controller_type)
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
            PhysXMaterialDesc {
                static_friction: 0.5,
                dynamic_friction: 0.5,
                restitution: 0.6,
            },
            PhysXColliderDesc::Box(0.5 * box_xz, 0.5 * box_y, 0.5 * box_xz),
            PhysXRigidBodyDesc::Static,
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
                    Vec3::new(x, 0.5 * (cube_scale + box_y), z),
                    Quat::identity(),
                    cube_scale,
                ),
                ..Default::default()
            })
            .with_bundle((
                PhysXMaterialDesc {
                    static_friction: 0.1,
                    dynamic_friction: 0.4,
                    restitution: 0.6,
                },
                PhysXColliderDesc::Box(0.5 * cube_scale, 0.5 * cube_scale, 0.5 * cube_scale),
                PhysXRigidBodyDesc::Dynamic {
                    density: 10.0,
                    angular_damping: 0.5,
                },
            ));
    }
}

pub fn spawn_character(
    mut commands: Commands,
    controller_type: Res<ControllerType>,
    character_settings: Res<CharacterSettings>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let box_y = 1.0;
    commands.spawn((
        GlobalTransform::identity(),
        Transform::from_translation(Vec3::new(
            0.0,
            0.5 * (box_y + character_settings.scale.y()),
            0.0,
        )),
        CharacterController::default(),
        PhysXMaterialDesc {
            static_friction: 0.5,
            dynamic_friction: 0.5,
            restitution: 0.0,
        },
        BodyTag,
    ));

    if *controller_type == ControllerType::KinematicTranslation {
        let body = commands
            .with_bundle((
                Mass::new(80.0),
                PhysXCapsuleControllerDesc {
                    height: character_settings.scale.y(),
                    radius: 0.5
                        * character_settings
                            .scale
                            .x()
                            .max(character_settings.scale.z()),
                    step_offset: 0.5,
                },
            ))
            .current_entity()
            .expect("Failed to spawn body");
        spawn_body_children(
            &mut commands,
            body,
            &controller_type,
            &character_settings,
            box_y,
            &mut materials,
            &mut meshes,
        );
    } else {
        let body = commands
            .with_bundle((
                PhysXColliderDesc::Capsule(
                    0.5 * character_settings
                        .scale
                        .x()
                        .max(character_settings.scale.z()),
                    character_settings.scale.y(),
                ),
                PhysXRigidBodyDesc::Dynamic {
                    density: 200.0,
                    angular_damping: 0.5,
                },
            ))
            .current_entity()
            .expect("Failed to spawn body");
        spawn_body_children(
            &mut commands,
            body,
            &controller_type,
            &character_settings,
            box_y,
            &mut materials,
            &mut meshes,
        );
    }
}

fn spawn_body_children(
    commands: &mut Commands,
    body: Entity,
    controller_type: &Res<ControllerType>,
    character_settings: &Res<CharacterSettings>,
    box_y: f32,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
) {
    let cube = meshes.add(Mesh::from(shape::Cube { size: 0.5 }));
    let red = materials.add(Color::hex("800000").unwrap().into());
    let (body_translation, head_translation) = if **controller_type
        == ControllerType::KinematicTranslation
    {
        (
            -0.5 * character_settings.head_scale * Vec3::unit_y(),
            0.5 * (character_settings.scale.y() - character_settings.head_scale) * Vec3::unit_y(),
        )
    } else {
        (
            0.5 * box_y * Vec3::unit_y(),
            0.5 * (box_y + character_settings.scale.y()) * Vec3::unit_y(),
        )
    };
    let yaw = commands
        .spawn((GlobalTransform::identity(), Transform::identity(), YawTag))
        .current_entity()
        .expect("Failed to spawn yaw");
    let body_model = commands
        .spawn(PbrComponents {
            material: red,
            mesh: cube,
            transform: Transform::new(Mat4::from_scale_rotation_translation(
                character_settings.scale - character_settings.head_scale * Vec3::unit_y(),
                Quat::identity(),
                body_translation,
            )),
            ..Default::default()
        })
        .current_entity()
        .expect("Failed to spawn body_model");
    let head = commands
        .spawn((
            GlobalTransform::identity(),
            Transform::from_translation_rotation(
                head_translation,
                Quat::from_rotation_y(character_settings.head_yaw),
            ),
            HeadTag,
        ))
        .current_entity()
        .expect("Failed to spawn head");
    let head_model = commands
        .spawn(PbrComponents {
            material: red,
            mesh: cube,
            transform: Transform::from_scale(character_settings.head_scale),
            ..Default::default()
        })
        .current_entity()
        .expect("Failed to spawn head_model");
    let camera = commands
        .spawn(Camera3dComponents {
            transform: Transform::new(Mat4::face_toward(
                character_settings.follow_offset,
                character_settings.focal_point,
                Vec3::unit_y(),
            )),
            ..Default::default()
        })
        .with_bundle((LookDirection::default(), CameraTag))
        .current_entity()
        .expect("Failed to spawn camera");
    commands
        .insert_one(body, LookEntity(camera))
        .push_children(body, &[yaw])
        .push_children(yaw, &[body_model, head])
        .push_children(head, &[head_model, camera]);
}

pub fn controller_to_physx_kinematic(
    translations: Res<Events<TranslationEvent>>,
    character_settings: Res<CharacterSettings>,
    mut reader: ResMut<ControllerEvents>,
    mut _physx: ResMut<PhysX>, // For synchronization
    _body: &BodyTag,
    mut physx_controller: Mut<PhysXController>,
    mut transform: Mut<Transform>,
    mut controller: Mut<CharacterController>,
) {
    let mut translation = Vec3::zero();
    for event in reader.translations.iter(&translations) {
        translation += **event;
    }
    // NOTE: This is just an example to stop falling past the initial body height
    // With a physics engine you would indicate that the body has collided with
    // something and should stop, depending on how your game works.
    let min_y = 0.5 * (1.0 + character_settings.scale.y());
    let position = physx_controller.get_position();
    if position.y() + translation.y() < min_y {
        *translation.y_mut() = min_y - position.y();
        controller.jumping = false;
    }
    let new_position = position + translation;
    physx_controller.set_position(new_position);
    transform.translate(translation);
}
