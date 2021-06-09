use bevy::{input::system::exit_on_esc_system, prelude::*};
use bevy_prototype_character_controller::{
    controller::{
        BodyTag, CameraTag, CharacterController, CharacterControllerPlugin, HeadTag, Mass, YawTag,
    },
    events::TranslationEvent,
    look::{LookDirection, LookEntity},
};
use rand::Rng;

pub struct CharacterSettings {
    pub scale: Vec3,
    pub head_scale: f32,
    pub head_yaw: f32,
    pub follow_offset: Vec3,
    pub focal_point: Vec3,
}

impl Default for CharacterSettings {
    fn default() -> Self {
        Self {
            scale: Vec3::new(0.5, 1.9, 0.3),
            head_scale: 0.3,
            head_yaw: 0.0,
            follow_offset: Vec3::new(0.0, 4.0, 8.0), // Relative to head
            focal_point: Vec3::ZERO,                 // Relative to head
        }
    }
}

pub struct FakeKinematicRigidBody;

pub fn build_app(app: &mut AppBuilder) {
    app.insert_resource(ClearColor(Color::hex("101010").unwrap()))
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(CharacterControllerPlugin)
        .add_system(exit_on_esc_system.system())
        .add_startup_system(spawn_world.system())
        .add_startup_system(spawn_character.system());
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
    commands.spawn_bundle(PbrBundle {
        material: grey,
        mesh: cube.clone(),
        transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(
            Vec3::new(20.0, 1.0, 20.0),
            Quat::IDENTITY,
            -Vec3::Y,
        )),
        ..Default::default()
    });

    // Cubes for some kind of reference in the scene to make it easy to see
    // what is happening
    let teal = materials.add(Color::hex("008080").unwrap().into());
    let cube_scale = 0.25;
    let mut rng = rand::thread_rng();
    for _ in 0..20 {
        let x = rng.gen_range(-10.0..10.0);
        let z = rng.gen_range(-10.0..10.0);
        commands.spawn_bundle(PbrBundle {
            material: teal.clone(),
            mesh: cube.clone(),
            transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(
                Vec3::splat(cube_scale),
                Quat::IDENTITY,
                Vec3::new(x, 0.5 * (cube_scale - 1.0), z),
            )),
            ..Default::default()
        });
    }
}

pub fn spawn_character(
    mut commands: Commands,
    character_settings: Res<CharacterSettings>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let red = materials.add(Color::hex("800000").unwrap().into());

    let body = commands
        .spawn_bundle((
            GlobalTransform::identity(),
            Transform::identity(),
            CharacterController::default(),
            FakeKinematicRigidBody,
            Mass::new(80.0),
            BodyTag,
        ))
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
                Vec3::new(0.0, character_settings.head_scale, 0.0),
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
                (0.5 * character_settings.scale.y + character_settings.head_scale) * Vec3::Y,
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

pub fn controller_to_kinematic(
    mut translations: EventReader<TranslationEvent>,
    mut query: Query<
        (&mut Transform, &mut CharacterController),
        (With<BodyTag>, With<FakeKinematicRigidBody>),
    >,
) {
    for (mut transform, mut controller) in query.iter_mut() {
        for translation in translations.iter() {
            transform.translation += **translation;
        }
        // NOTE: This is just an example to stop falling past the initial body height
        // With a physics engine you would indicate that the body has collided with
        // something and should stop, depending on how your game works.
        if transform.translation.y < 0.0 {
            transform.translation.y = 0.0;
            controller.jumping = false;
        }
    }
}
