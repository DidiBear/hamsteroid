#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]
#![deny(clippy::unwrap_used, clippy::indexing_slicing)]

use bevy::{input::system::exit_on_esc_system, prelude::*};
use bevy_inspector_egui::{Inspectable, InspectorPlugin, WorldInspectorPlugin};
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use cooldown::Cooldown;
use inputs::{InputEvent, InputsPlugin};

mod cooldown;
mod inputs;

#[derive(Inspectable)]
struct Constants {
    // Movement configs
    default_damping: f32,
    stabilisation_damping: f32,
    impulse_value: f32,
    force_value: f32,
    acceleration_value: f32,

    // Trail configs
    trail_size_scale: f32,
}

impl Default for Constants {
    fn default() -> Self {
        Constants {
            // Movement configs
            stabilisation_damping: 6.,
            default_damping: 1.,
            impulse_value: 15.,
            force_value: 6.,
            acceleration_value: 0.3,
            // Trail configs
            trail_size_scale: 0.5,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Constants>::new())
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(InputsPlugin)
        .add_plugin(ShapePlugin)
        .add_startup_system(setup.label("main-setup"))
        .add_startup_system(setup_physics.after("main-setup"))
        .add_system(exit_on_esc_system)
        .add_system(apply_forces)
        .add_system(update_heat_color)
        .add_system(update_trails)
        .run();
}

fn setup(mut commands: Commands, mut rapier_config: ResMut<RapierConfiguration>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    rapier_config.scale = 100.;
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Trail;

#[derive(Component)]
struct Heat {
    amount: f32,
}

impl Heat {
    fn inc(&mut self, value: f32) {
        self.amount = (self.amount + value).clamp(0., 1.);
    }
}

fn setup_physics(
    mut commands: Commands,
    constants: Res<Constants>,
    rapier_config: Res<RapierConfiguration>,
) {
    let scale = rapier_config.scale;
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
                    extents: Vec2::new(w, h) * scale * 2.,
                    ..Default::default()
                },
                DrawMode::Fill(FillMode::color(Color::WHITE)),
                Transform::default(),
            ))
            .insert_bundle(ColliderBundle {
                shape: ColliderShape::cuboid(w, h).into(),
                position: pos.into(),
                material: collider_material.clone().into(),
                ..Default::default()
            })
            .insert(ColliderPositionSync::Discrete);
    };

    spawn_border(12., 0.1, Vec2::new(0., 3.)); // top
    spawn_border(12., 0.1, Vec2::new(0., -3.)); // bottom
    spawn_border(0.1, 6., Vec2::new(-6., 0.)); // left
    spawn_border(0.1, 6., Vec2::new(6., 0.)); // right

    let shape_ball = shapes::Circle {
        radius: 0.3 * scale,
        center: Vec2::ZERO,
    };
    let ccd = RigidBodyCcd {
        ccd_enabled: true,
        ..Default::default()
    };

    commands
        .spawn()
        .insert(Player)
        .insert(Heat { amount: 0. })
        .insert_bundle(GeometryBuilder::build_as(
            &shape_ball,
            DrawMode::Fill(FillMode::color(Color::ORANGE)),
            Transform::default(),
        ))
        .insert_bundle(RigidBodyBundle {
            body_type: RigidBodyType::Dynamic.into(),
            ccd: ccd.clone().into(),
            damping: RigidBodyDamping {
                linear_damping: constants.default_damping,
                ..Default::default()
            }
            .into(),
            forces: RigidBodyForces {
                gravity_scale: 0.,
                ..Default::default()
            }
            .into(),
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::ball(0.3).into(),
            material: collider_material.clone().into(),
            ..Default::default()
        })
        .insert(RigidBodyPositionSync::Discrete)
        .with_children(|commands| {
            let mut color = Color::ORANGE;
            color.set_a(0.5);

            commands
                .spawn()
                .insert(Trail)
                .insert_bundle(GeometryBuilder::build_as(
                    &shapes::Polygon::default(),
                    DrawMode::Fill(FillMode::color(color)),
                    Transform::default(),
                ));
        });

    commands
        .spawn()
        .insert_bundle(GeometryBuilder::build_as(
            &shape_ball,
            DrawMode::Fill(FillMode::color(Color::RED)),
            Transform::default(),
        ))
        .insert_bundle(RigidBodyBundle {
            position: Vec2::new(0.5, 0.5).into(),
            ccd: ccd.clone().into(),
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::ball(0.3).into(),
            material: collider_material.clone().into(),
            ..Default::default()
        })
        .insert(RigidBodyPositionSync::Discrete);
}

struct ImpulseCooldown(Cooldown);

impl Default for ImpulseCooldown {
    fn default() -> Self {
        ImpulseCooldown(Cooldown::from_seconds(0.35))
    }
}

fn apply_forces(
    constants: Res<Constants>,
    mut impulse_cooldown: Local<ImpulseCooldown>,
    time: Res<Time>,
    mut input_events: EventReader<InputEvent>,
    mut rigid_bodies: Query<
        (
            &mut RigidBodyVelocityComponent,
            &RigidBodyMassPropsComponent,
            &mut RigidBodyDampingComponent,
            &mut RigidBodyForcesComponent,
            &mut Heat,
        ),
        With<Player>,
    >,
) {
    impulse_cooldown.0.tick(time.delta());

    for input_event in input_events.iter() {
        match input_event {
            InputEvent::Impulse { direction } => {
                if !impulse_cooldown.0.finished() {
                    continue;
                }
                impulse_cooldown.0.start();

                let impulse = *direction * constants.impulse_value;

                for (mut velocity, mass_props, mut damping, _, mut heat) in rigid_bodies.iter_mut()
                {
                    damping.linear_damping = constants.default_damping;
                    velocity.apply_impulse(mass_props, impulse.into());
                    heat.inc(0.1)
                }
            }
            InputEvent::Stabilisation => {
                for (_, _, mut damping, _, mut heat) in rigid_bodies.iter_mut() {
                    damping.linear_damping = constants.stabilisation_damping;
                    heat.inc(-1.)
                }
            }
            InputEvent::Accelerate => {
                if !impulse_cooldown.0.finished() {
                    continue;
                }
                impulse_cooldown.0.start();

                for (mut velocity, mass_props, _, _, mut heat) in rigid_bodies.iter_mut() {
                    let impulse = velocity.linvel * constants.acceleration_value;
                    velocity.apply_impulse(mass_props, impulse.into());
                    heat.inc(0.1)
                }
            }
            InputEvent::Force { direction } => {
                let force = *direction * constants.force_value;

                for (_, _, mut damping, mut forces, mut heat) in rigid_bodies.iter_mut() {
                    damping.linear_damping = constants.default_damping;
                    forces.force = force.into();
                    heat.inc(0.01)
                }
            }
        }
    }
}

fn update_heat_color(mut colors: Query<(&Heat, &mut DrawMode), (With<Player>, Changed<Heat>)>) {
    for (heat, draw_mode) in colors.iter_mut() {
        if let DrawMode::Fill(mut fill_mode) = *draw_mode {
            let percent = heat.amount;
            fill_mode.color = Color::RED * percent + Color::MIDNIGHT_BLUE * (1. - percent);
        }
    }
}

#[derive(Default)]
struct Position(Vec2);

fn update_trails(
    mut trail_paths: Query<(&mut Path, &Parent), With<Trail>>,
    mut previous_pos: Local<Position>,
    transforms: Query<&Transform, Changed<RigidBodyVelocityComponent>>,
    rapier_config: Res<RapierConfiguration>,
    constants: Res<Constants>,
) {
    let scale = rapier_config.scale;
    let size = 0.3 * scale;

    for (mut path, parent) in trail_paths.iter_mut() {
        let pos = if let Ok(transform) = transforms.get(parent.0) {
            transform.translation.truncate()
        } else {
            continue;
        };

        let delta = previous_pos.0 - pos;
        previous_pos.0 = pos;

        if delta == Vec2::ZERO {
            continue;
        }
        let trail = delta.normalize() * (size * constants.trail_size_scale + delta.length());

        let polygon = shapes::Polygon {
            points: vec![
                trail,
                Vec2::new(trail.y, -trail.x).normalize() * size, // 90 degrees clockwize
                Vec2::new(-trail.y, trail.x).normalize() * size, // 90 degrees counterclockwize
            ],
            closed: true,
        };
        *path = ShapePath::build_as(&polygon);
    }
}
