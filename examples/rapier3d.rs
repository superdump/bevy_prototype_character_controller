use bevy::{input::system::exit_on_esc_system, prelude::*};
use bevy_prototype_character_controller::{
    controller::{CharacterController, CharacterControllerPlugin, Mass},
    events::{ForceEvent, ImpulseEvent, YawEvent},
    look::LookDirection,
};
use bevy_rapier3d::{
    na,
    physics::{RapierPhysicsPlugin, RigidBodyHandleComponent},
    rapier::{
        dynamics::{RigidBodyBuilder, RigidBodySet},
        geometry::ColliderBuilder,
        math::{Isometry, Vector},
    },
};
use rand::Rng;

// Take a look at example_utils/utils.rs for details!
#[path = "../example_utils/utils.rs"]
mod utils;
use utils::*;

fn main() {
    App::build()
        // Generic
        .add_resource(ClearColor(Color::hex("101010").unwrap()))
        .add_resource(Msaa { samples: 4 })
        .add_default_plugins()
        .add_system(exit_on_esc_system.system())
        // Character Controller
        .add_plugin(CharacterControllerPlugin)
        .init_resource::<ControllerEvents>()
        // Rapier
        .add_plugin(RapierPhysicsPlugin)
        // Character controller adaptations for Rapier
        .add_system(create_mass.system())
        .add_system(constrain_rotation.system())
        .add_system_to_stage_front(bevy::app::stage::PRE_UPDATE, body_to_velocity.system())
        // IMPORTANT: The impulse/force systems MUST run before the physics simulation step, so they
        // either need to be added to the end of PRE_UPDATE or the beginning of UPDATE
        // Option A. Apply impulses (changes in momentum)
        // .add_system_to_stage_front(bevy::app::stage::UPDATE, controller_to_rapier_dynamic_impulse.system())
        // Option B. Apply forces (rate of change of momentum)
        .add_system_to_stage_front(
            bevy::app::stage::UPDATE,
            controller_to_rapier_dynamic_force.system(),
        )
        // The yaw needs to be applied to the rigid body so this system has to
        // be implemented for the physics engine in question
        .add_system_to_stage_front(bevy::app::stage::UPDATE, controller_to_rapier_yaw.system())
        // Controllers generally all want to pitch in the same way
        .add_system_to_stage_front(bevy::app::stage::UPDATE, controller_to_pitch.system())
        // Specific to this demo
        .init_resource::<CharacterSettings>()
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

pub struct BodyYawTag;

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
            BodyYawTag,
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

pub struct ConstrainedTag;

pub fn constrain_rotation(
    mut commands: Commands,
    mut bodies: ResMut<RigidBodySet>,
    mut query: Query<Without<ConstrainedTag, With<BodyTag, (Entity, &RigidBodyHandleComponent)>>>,
) {
    for (entity, body_handle) in &mut query.iter() {
        let mut body = bodies
            .get_mut(body_handle.handle())
            .expect("Failed to get RigidBody");
        body.mass_properties.inv_principal_inertia_sqrt.x = 0.0;
        body.mass_properties.inv_principal_inertia_sqrt.y = 0.0;
        body.mass_properties.inv_principal_inertia_sqrt.z = 0.0;
        commands.insert_one(entity, ConstrainedTag);
    }
}

pub fn create_mass(
    mut commands: Commands,
    bodies: Res<RigidBodySet>,
    mut query: Query<Without<Mass, (Entity, &RigidBodyHandleComponent)>>,
) {
    for (entity, body_handle) in &mut query.iter() {
        let body = bodies
            .get(body_handle.handle())
            .expect("Failed to get RigidBody");
        let mass = 1.0 / body.mass_properties.inv_mass;
        commands.insert_one(entity, Mass::new(mass));
    }
}

pub fn body_to_velocity(
    bodies: Res<RigidBodySet>,
    _body: &BodyTag,
    body_handle: &RigidBodyHandleComponent,
    mut controller: Mut<CharacterController>,
) {
    let body = bodies
        .get(body_handle.handle())
        .expect("Failed to get RigidBody");
    let velocity = body.linvel;
    controller.velocity = Vec3::new(velocity[0], velocity[1], velocity[2]);
}

pub fn controller_to_rapier_dynamic_impulse(
    impulses: Res<Events<ImpulseEvent>>,
    mut reader: ResMut<ControllerEvents>,
    mut bodies: ResMut<RigidBodySet>,
    _body: &BodyTag,
    body_handle: &RigidBodyHandleComponent,
) {
    let mut impulse = Vec3::zero();
    for event in reader.impulses.iter(&impulses) {
        impulse += **event;
    }

    if impulse.length_squared() > 1E-6 {
        let mut body = bodies
            .get_mut(body_handle.handle())
            .expect("Failed to get character body");
        body.wake_up(true);
        body.apply_impulse(Vector::new(impulse.x(), impulse.y(), impulse.z()));
    }
}

pub fn controller_to_rapier_dynamic_force(
    forces: Res<Events<ForceEvent>>,
    mut reader: ResMut<ControllerEvents>,
    mut bodies: ResMut<RigidBodySet>,
    _body: &BodyTag,
    body_handle: &RigidBodyHandleComponent,
) {
    let mut force = Vec3::zero();
    for event in reader.forces.iter(&forces) {
        force += **event;
    }

    if force.length_squared() > 1E-6 {
        let mut body = bodies
            .get_mut(body_handle.handle())
            .expect("Failed to get character body");
        body.wake_up(true);
        body.apply_force(Vector::new(force.x(), force.y(), force.z()));
    }
}

pub fn controller_to_rapier_yaw(
    mut reader: ResMut<ControllerEvents>,
    yaws: Res<Events<YawEvent>>,
    mut bodies: ResMut<RigidBodySet>,
    _body: &BodyTag,
    body_handle: &RigidBodyHandleComponent,
) {
    let mut yaw = None;
    for event in reader.yaws.iter(&yaws) {
        yaw = Some(**event);
    }
    if let Some(yaw) = yaw {
        let mut body = bodies
            .get_mut(body_handle.handle())
            .expect("Failed to get character body");
        body.wake_up(true);
        let translation = body.position.translation;
        body.set_position(Isometry::from_parts(
            translation,
            na::UnitQuaternion::from_scaled_axis(Vector::y() * yaw),
        ));
    }
}
