#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

use bevy::{input::system::exit_on_esc_system, prelude::*};
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use bevy_prototype_lyon::{
    plugin::ShapePlugin,
    prelude::{DrawMode, FillOptions, GeometryBuilder, ShapeColors},
    shapes,
};
use bevy_rapier2d::prelude::*;
use inputs::{InputEvent, InputsPlugin};

mod inputs;

#[derive(Inspectable)]
struct Constants {
    default_damping: f32,
    stabilisation_damping: f32,
    impulse_value: f32,
    force_value: f32,
    acceleration_value: f32,
}

impl Default for Constants {
    fn default() -> Self {
        Constants {
            stabilisation_damping: 6.,
            default_damping: 1.,
            impulse_value: 15.,
            force_value: 6.,
            acceleration_value: 0.3,
        }
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Constants>::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(InputsPlugin)
        .add_plugin(ShapePlugin)
        .add_startup_system(setup.system().label("main-setup"))
        .add_startup_system(setup_physics.system().after("main-setup"))
        .add_system(exit_on_esc_system.system())
        .add_system(apply_forces.system())
        .run();
}

fn setup(mut commands: Commands, mut rapier_config: ResMut<RapierConfiguration>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    rapier_config.scale = 100.;
}

struct Player;

fn setup_physics(
    mut commands: Commands,
    constants: Res<Constants>,
    rapier_config: Res<RapierConfiguration>,
) {
    let collider_material = ColliderMaterial {
        friction: 0.,
        restitution: 0.9,
        ..Default::default()
    };

    let mut spawn_border = |w: f32, h: f32, pos: Vec2| {
        commands
            .spawn()
            .insert_bundle(GeometryBuilder::build_as(
                &shapes::Rectangle {
                    width: w * rapier_config.scale * 2.,
                    height: h * rapier_config.scale * 2.,
                    ..Default::default()
                },
                ShapeColors::new(Color::WHITE),
                DrawMode::Fill(FillOptions::default()),
                Transform::default(),
            ))
            .insert_bundle(ColliderBundle {
                shape: ColliderShape::cuboid(w, h),
                position: pos.into(),
                material: collider_material.clone(),
                ..Default::default()
            })
            .insert(ColliderPositionSync::Discrete);
    };

    spawn_border(12., 0.1, Vec2::new(0., 3.)); // top
    spawn_border(12., 0.1, Vec2::new(0., -3.)); // bottom
    spawn_border(0.1, 6., Vec2::new(-6., 0.)); // left
    spawn_border(0.1, 6., Vec2::new(6., 0.)); // right

    let shape_ball = shapes::Ellipse {
        radii: Vec2::new(0.35 * rapier_config.scale, 0.25 * rapier_config.scale),
        center: Vec2::ZERO,
    };
    let ccd = RigidBodyCcd {
        ccd_enabled: true,
        ..Default::default()
    };

    commands
        .spawn()
        .insert(Player)
        .insert_bundle(GeometryBuilder::build_as(
            &shape_ball,
            ShapeColors::new(Color::ORANGE),
            DrawMode::Fill(FillOptions::default()),
            Transform::default(),
        ))
        .insert_bundle(RigidBodyBundle {
            body_type: RigidBodyType::Dynamic,
            ccd,
            damping: RigidBodyDamping {
                linear_damping: constants.default_damping,
                ..Default::default()
            },
            forces: RigidBodyForces {
                gravity_scale: 0.,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::ball(0.3),
            // mass_properties: ColliderMassProps::Density(1.),
            material: collider_material.clone(),
            ..Default::default()
        })
        .insert(RigidBodyPositionSync::Discrete);

    commands
        .spawn()
        .insert_bundle(GeometryBuilder::build_as(
            &shape_ball,
            ShapeColors::new(Color::RED),
            DrawMode::Fill(FillOptions::default()),
            Transform::default(),
        ))
        .insert_bundle(RigidBodyBundle {
            position: Vec2::new(0.5, 0.5).into(),
            ccd,
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::ball(0.3),
            material: collider_material.clone(),
            ..Default::default()
        })
        .insert(RigidBodyPositionSync::Discrete);
}

fn apply_forces(
    constants: Res<Constants>,
    mut input_events: EventReader<InputEvent>,
    mut rigid_bodies: Query<
        (
            &mut RigidBodyVelocity,
            &RigidBodyMassProps,
            &mut RigidBodyDamping,
            &mut RigidBodyForces,
        ),
        With<Player>,
    >,
) {
    for input_event in input_events.iter() {
        match input_event {
            InputEvent::Impulse { direction } => {
                let impulse = *direction * constants.impulse_value;

                for (mut velocity, mass_props, mut damping, _) in rigid_bodies.iter_mut() {
                    damping.linear_damping = constants.default_damping;
                    velocity.apply_impulse(mass_props, impulse.into());
                }
            }
            InputEvent::Stabilisation => {
                for (_, _, mut damping, _) in rigid_bodies.iter_mut() {
                    damping.linear_damping = constants.stabilisation_damping;
                }
            }
            InputEvent::Accelerate => {
                for (mut velocity, mass_props, _, _) in rigid_bodies.iter_mut() {
                    let impulse = velocity.linvel * constants.acceleration_value;
                    velocity.apply_impulse(mass_props, impulse.into());
                }
            }
            InputEvent::Force { direction } => {
                let force = *direction * constants.force_value;

                for (_, _, mut damping, mut forces) in rigid_bodies.iter_mut() {
                    damping.linear_damping = constants.default_damping;
                    forces.force = force.into();
                }
            }
        }
    }
}
