use bevy::{input::system::exit_on_esc_system, prelude::*};
use bevy_prototype_character_controller::{
    controller::{CharacterController, CharacterControllerPlugin},
    events::{LookDeltaEvent, LookEvent, PitchEvent, TranslationEvent, YawEvent},
    look::LookDirection,
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
            scale: Vec3::new(0.5, 1.0, 0.3),
            head_scale: 0.3,
            head_yaw: 0.0,
            follow_offset: Vec3::new(0.0, 4.0, 8.0), // Relative to head
            focal_point: Vec3::zero(),               // Relative to head
        }
    }
}

#[derive(Default)]
pub struct ControllerEvents {
    pub translations: EventReader<TranslationEvent>,
    pub yaws: EventReader<YawEvent>,
    pub pitches: EventReader<PitchEvent>,
    pub looks: EventReader<LookEvent>,
    pub look_deltas: EventReader<LookDeltaEvent>,
}

pub struct FakeKinematicRigidBody;

pub struct BodyTag;
pub struct HeadTag;
pub struct CameraTag;

pub fn build_app(app: &mut AppBuilder) {
    app.add_resource(ClearColor(Color::hex("101010").unwrap()))
        .add_resource(Msaa { samples: 4 })
        .add_default_plugins()
        .add_plugin(CharacterControllerPlugin)
        .init_resource::<ControllerEvents>()
        .add_system(exit_on_esc_system.system())
        .add_startup_system(spawn_world.system())
        .add_startup_system(spawn_character.system());
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
    commands.spawn(PbrComponents {
        material: grey,
        mesh: cube,
        transform: Transform::new(Mat4::from_scale_rotation_translation(
            Vec3::new(20.0, 1.0, 20.0),
            Quat::identity(),
            -Vec3::unit_y(),
        )),
        ..Default::default()
    });

    // Cubes for some kind of reference in the scene to make it easy to see
    // what is happening
    let teal = materials.add(Color::hex("008080").unwrap().into());
    let cube_scale = 0.25;
    let mut rng = rand::thread_rng();
    for _ in 0..20 {
        let x = rng.gen_range(-10.0, 10.0);
        let z = rng.gen_range(-10.0, 10.0);
        commands.spawn(PbrComponents {
            material: teal,
            mesh: cube,
            transform: Transform::from_translation_rotation_scale(
                Vec3::new(x, 0.5 * (cube_scale - 1.0), z),
                Quat::identity(),
                cube_scale,
            ),
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
    let cube = meshes.add(Mesh::from(shape::Cube { size: 0.5 }));
    // Character
    let red = materials.add(Color::hex("800000").unwrap().into());
    commands
        .spawn((
            GlobalTransform::identity(),
            Transform::identity(),
            CharacterController::default(),
            FakeKinematicRigidBody,
            BodyTag,
        ))
        .with_children(|body| {
            body.spawn(PbrComponents {
                material: red,
                mesh: cube,
                transform: Transform::new(Mat4::from_scale_rotation_translation(
                    character_settings.scale,
                    Quat::identity(),
                    0.5 * (character_settings.scale.y() - 1.0) * Vec3::unit_y(),
                )),
                ..Default::default()
            })
            .spawn((
                GlobalTransform::identity(),
                Transform::from_translation_rotation(
                    0.5 * (character_settings.scale.y() + character_settings.head_scale)
                        * Vec3::unit_y(),
                    Quat::from_rotation_y(character_settings.head_yaw), // FIXME - this is a hack
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

pub fn controller_to_kinematic(
    translations: Res<Events<TranslationEvent>>,
    mut reader: ResMut<ControllerEvents>,
    _body: &BodyTag,
    _kinematic_body: &FakeKinematicRigidBody,
    mut transform: Mut<Transform>,
    mut controller: Mut<CharacterController>,
) {
    for translation in reader.translations.iter(&translations) {
        transform.translate(**translation);
    }
    // NOTE: This is just an example to stop falling past the initial body height
    // With a physics engine you would indicate that the body has collided with
    // something and should stop, depending on how your game works.
    if transform.translation().y() < 0.0 {
        *transform.value_mut().w_axis_mut().y_mut() = 0.0;
        controller.jumping = false;
    }
}

pub fn controller_to_yaw(
    mut reader: ResMut<ControllerEvents>,
    yaws: Res<Events<YawEvent>>,
    _body: &BodyTag,
    mut transform: Mut<Transform>,
) {
    for yaw in reader.yaws.iter(&yaws) {
        transform.set_rotation(Quat::from_rotation_ypr(**yaw, 0.0, 0.0));
    }
}

pub fn controller_to_pitch(
    mut reader: ResMut<ControllerEvents>,
    pitches: Res<Events<PitchEvent>>,
    _head: &HeadTag,
    mut transform: Mut<Transform>,
) {
    for pitch in reader.pitches.iter(&pitches) {
        transform.set_rotation(Quat::from_rotation_ypr(0.0, **pitch, 0.0));
    }
}
