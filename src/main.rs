#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

use bevy::{input::system::exit_on_esc_system, prelude::*};
use bevy_prototype_lyon::{
    plugin::ShapePlugin,
    prelude::{DrawMode, FillOptions, GeometryBuilder, ShapeColors},
    shapes,
};
use bevy_rapier2d::prelude::*;
use inputs::{InputEvent, InputsPlugin};

mod inputs;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
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
    mut materials: ResMut<Assets<ColorMaterial>>,
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

    let shape_ball = shapes::Circle {
        radius: 0.3 * rapier_config.scale,
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
                linear_damping: 1.,
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


fn apply_forces(
    mut input_events: EventReader<InputEvent>,
    mut rigid_bodies: Query<
        (
            &mut RigidBodyVelocity,
            &RigidBodyMassProps,
            &mut RigidBodyDamping,
        ),
        With<Player>,
    >,
) {
    for input_event in input_events.iter() {
        match input_event {
            InputEvent::Movement { direction } => {
                let impulse = *direction * 30.;

                for (mut velocity, mass_props, mut damping) in rigid_bodies.iter_mut() {
                    damping.linear_damping = 1.;
                    velocity.apply_impulse(mass_props, impulse.into());
                }
            }
            InputEvent::Focus => {
                for (_, _, mut damping) in rigid_bodies.iter_mut() {
                    damping.linear_damping = 10.;
                }
            }
        }
    }
}

