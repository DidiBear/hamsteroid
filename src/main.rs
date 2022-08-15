#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(clippy::unwrap_used, clippy::indexing_slicing)]
#![allow(
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::module_name_repetitions
)]

use bevy::{prelude::*, window::close_on_esc};
use bevy_inspector_egui::{Inspectable, InspectorPlugin, WorldInspectorPlugin};
use bevy_rapier2d::prelude::*;

// use bevy_flycam::{FlyCam, NoCameraPlayerPlugin, PlayerPlugin};

mod cooldown;
mod inputs;
mod particles;

use cooldown::Cooldown;
use inputs::{InputEvent, InputsPlugin};
use particles::ParticleEffectPlugin;

const Z: f32 = 0.0;

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

    // Heat config
    heat_increase: f32,
}

impl Default for Constants {
    fn default() -> Self {
        Self {
            // Movement configs
            stabilisation_damping: 6.,
            default_damping: 1.,
            impulse_value: 1500.,
            force_value: 600.,
            acceleration_value: 0.3,
            // Trail configs
            trail_size_scale: 0.5,
            // Heat config
            heat_increase: 0.2,
        }
    }
}

fn main() {
    App::new()
        .insert_resource(Msaa::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(ParticleEffectPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugin(InspectorPlugin::<Constants>::new())
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.)) // scale = cm
        .add_plugin(InputsPlugin)
        .add_plugin(RapierDebugRenderPlugin::default())
        // .add_plugin(NoCameraPlayerPlugin)
        .add_startup_system(setup_camera)
        .add_startup_system(setup_physics)
        .add_system(close_on_esc)
        .add_system(apply_forces)
        .add_system(cancel_force.before(apply_forces))
        .add_system(update_heat_color)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Trail;

#[derive(Component)]
struct Heat {
    /// Between 0 and 1.
    amount: f32,
}

impl Heat {
    fn inc(&mut self, value: f32) {
        self.amount = (self.amount + value).clamp(0., 1.);
    }
}

fn setup_physics(mut commands: Commands, constants: Res<Constants>) {
    commands
        .spawn()
        .insert(Name::new("Center"))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 0.0, Z)))
        .insert(Collider::ball(5.));

    let friction = Friction::coefficient(0.);
    let restitution = Restitution::coefficient(0.9);

    let mut spawn_border = |name: &str, w: f32, h: f32, pos: Vec2| {
        commands
            .spawn()
            .insert(Name::new(name.to_string()))
            .insert_bundle((Collider::cuboid(w, h), friction, restitution))
            .insert_bundle(TransformBundle::from(Transform::from_xyz(pos.x, pos.y, Z)));
    };

    spawn_border("Top", 1000., 10., Vec2::new(0., 300.));
    spawn_border("Bottom", 1000., 10., Vec2::new(0., -300.));
    spawn_border("Left", 10., 500., Vec2::new(-600., 0.));
    spawn_border("Right", 10., 500., Vec2::new(600., 0.));

    commands
        .spawn()
        .insert(Name::new("Player"))
        .insert(Player)
        .insert(Heat { amount: 0. })
        .insert(RigidBody::Dynamic)
        .insert_bundle(TransformBundle::from(Transform::from_xyz(-100., 0., Z)))
        .insert(Ccd::enabled())
        .insert(GravityScale(0.))
        .insert(Velocity::default())
        .insert(Damping::splat(constants.default_damping))
        .insert(ExternalImpulse::default())
        .insert(ExternalForce::default())
        .insert_bundle((
            Collider::ball(30.),
            friction,
            restitution,
            ActiveEvents::COLLISION_EVENTS,
            ColliderDebugColor(Color::MIDNIGHT_BLUE),
        ));
    // .with_children(|commands| {
    //     let mut color = Color::ORANGE;
    //     color.set_a(0.5);

    //     commands
    //         .spawn()
    //         .insert(Trail)
    //         .insert_bundle(GeometryBuilder::build_as(
    //             &shapes::Polygon::default(),
    //             DrawMode::Fill(FillMode::color(color)),
    //             Transform::default(),
    //         ));
    // });

    commands
        .spawn()
        .insert(Name::new("Other ball"))
        .insert(RigidBody::Dynamic)
        .insert_bundle(TransformBundle::from(Transform::from_xyz(-110., 100., Z)))
        .insert(Ccd::enabled())
        .insert_bundle((Collider::ball(30.), friction, restitution));
}

/// Cancel the external force applied to the player.
fn cancel_force(mut player: Query<&mut ExternalForce, (With<Player>, Changed<ExternalForce>)>) {
    for mut ext_force in &mut player {
        ext_force.force = Vec2::ZERO;
    }
}

struct ImpulseCooldown(Cooldown);

impl Default for ImpulseCooldown {
    fn default() -> Self {
        Self(Cooldown::from_seconds(0.35))
    }
}

fn apply_forces(
    constants: Res<Constants>,
    mut impulse_cooldown: Local<ImpulseCooldown>,
    time: Res<Time>,
    mut input_events: EventReader<InputEvent>,
    mut player: Query<
        (
            &Velocity,
            &mut ExternalImpulse,
            &mut ExternalForce,
            &mut Damping,
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

                for (_, mut ext_impulse, _, mut damping, mut heat) in &mut player {
                    *damping = Damping::splat(constants.default_damping);
                    ext_impulse.impulse = impulse;
                    heat.inc(0.2);
                }
            }
            InputEvent::Stabilisation => {
                for (_, _, _, mut damping, mut heat) in &mut player {
                    *damping = Damping::splat(constants.stabilisation_damping);
                    heat.inc(-1.);
                }
            }
            InputEvent::Accelerate => {
                if !impulse_cooldown.0.finished() {
                    continue;
                }
                impulse_cooldown.0.start();

                for (velocity, mut ext_impulse, _, _, mut heat) in &mut player {
                    let impulse = velocity.linvel * constants.acceleration_value;
                    ext_impulse.impulse = impulse;
                    heat.inc(0.2);
                }
            }
            InputEvent::Force { direction } => {
                let force = *direction * constants.force_value;

                for (_, _, mut ext_force, mut damping, _) in &mut player {
                    damping.linear_damping = constants.default_damping;
                    ext_force.force = force;
                }
            }
        }
    }
}

fn update_heat_color(
    mut player: Query<(&Heat, &mut ColliderDebugColor), (With<Player>, Changed<Heat>)>,
) {
    for (heat, mut debug_color) in &mut player {
        let percent = heat.amount;
        debug_color.0 = Color::RED * percent + Color::MIDNIGHT_BLUE * (1. - percent);
    }
}

// #[derive(Default)]
// struct Position(Vec2);

// fn update_trails(
//     mut trail_paths: Query<(&mut Path, &Parent), With<Trail>>,
//     mut previous_pos: Local<Position>,
//     transforms: Query<&Transform, Changed<Velocity>>,
//     constants: Res<Constants>,
// ) {
//     let size = 0.3 * SCALE;

//     for (mut path, parent) in trail_paths.iter_mut() {
//         let pos = if let Ok(transform) = transforms.get(parent.get()) {
//             transform.translation.truncate()
//         } else {
//             continue;
//         };

//         let delta = previous_pos.0 - pos;
//         previous_pos.0 = pos;

//         if delta == Vec2::ZERO {
//             continue;
//         }
//         let trail = delta.normalize() * (size * constants.trail_size_scale + delta.length());

//         let polygon = shapes::Polygon {
//             points: vec![
//                 trail,
//                 Vec2::new(trail.y, -trail.x).normalize() * size, // 90 degrees clockwize
//                 Vec2::new(-trail.y, trail.x).normalize() * size, // 90 degrees counterclockwize
//             ],
//             closed: true,
//         };
//         *path = ShapePath::build_as(&polygon);
//     }
// }

trait DampingExt {
    fn splat(value: f32) -> Damping;
}

impl DampingExt for Damping {
    fn splat(value: f32) -> Damping {
        Damping {
            linear_damping: value,
            angular_damping: value,
        }
    }
}
