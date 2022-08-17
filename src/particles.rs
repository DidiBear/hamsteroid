use bevy::{
    prelude::*,
    render::{render_resource::WgpuFeatures, settings::WgpuSettings},
};

use bevy_hanabi::*;
use bevy_rapier2d::prelude::*;

use crate::{inputs::InputEvent, Player, PLAYER_RADIUS};

pub struct ParticleEffectPlugin;

impl Plugin for ParticleEffectPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(create_wgpu_settings())
            .add_plugin(HanabiPlugin)
            .add_startup_system(setup_particle_effects)
            .add_system(trigger_collision_effects)
            .add_system(trigger_input_effects);
    }
}

fn create_wgpu_settings() -> WgpuSettings {
    let mut options = WgpuSettings::default();
    options
        .features
        .set(WgpuFeatures::VERTEX_WRITABLE_STORAGE, true);
    options
}

#[derive(Component)]
struct ExplosionEffect;

#[derive(Component)]
struct CollisionEffect;

#[derive(Component)]
pub struct PropulsorEffect;

fn setup_particle_effects(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    spawn_particle_effect(
        &mut commands,
        &mut effects,
        "Collision effect",
        CollisionEffect,
        collision_effect(),
    );
    spawn_particle_effect(
        &mut commands,
        &mut effects,
        "Explosion effect",
        ExplosionEffect,
        explosion_effect(),
    );
    spawn_particle_effect(
        &mut commands,
        &mut effects,
        "Propulsor effect",
        PropulsorEffect,
        propulsor_effect(),
    );
}

fn spawn_particle_effect(
    commands: &mut Commands,
    effects: &mut ResMut<Assets<EffectAsset>>,
    name: &'static str,
    tag: impl Component,
    effect: EffectAsset,
) {
    let spawner = effect.spawner;
    commands
        .spawn()
        .insert(tag)
        .insert(Name::new(name))
        .insert_bundle(ParticleEffectBundle::new(effects.add(effect)).with_spawner(spawner));
}

fn collision_effect() -> EffectAsset {
    let mut gradient = Gradient::new();
    gradient.add_key(0., Color::GRAY.into());
    gradient.add_key(1., Color::BLACK.into());

    let spawner = Spawner::once(15.0.into(), false);

    EffectAsset {
        name: "Impact".into(),
        capacity: 32768,
        spawner,
        ..default()
    }
    .init(PositionSphereModifier {
        radius: 5.,
        speed: 50.0.into(),
        dimension: ShapeDimension::Surface,
        ..default()
    })
    .init(ParticleLifetimeModifier { lifetime: 0.3 })
    .render(SizeOverLifetimeModifier {
        gradient: Gradient::constant(Vec2::splat(2.)),
    })
    .render(ColorOverLifetimeModifier { gradient })
}

fn explosion_effect() -> EffectAsset {
    let mut gradient = Gradient::new();
    gradient.add_key(0., Color::rgba(1., 1., 0., 1.).into());
    gradient.add_key(1., Color::rgba(1., 0., 0., 0.).into());

    let spawner = Spawner::once(100.0.into(), false);

    EffectAsset {
        name: "Explosion".into(),
        capacity: 32768,
        spawner,
        ..default()
    }
    .init(PositionSphereModifier {
        radius: 25.,
        speed: 200.0.into(),
        dimension: ShapeDimension::Surface,
        ..default()
    })
    .init(ParticleLifetimeModifier { lifetime: 0.5 })
    .render(SizeOverLifetimeModifier {
        gradient: Gradient::constant(Vec2::splat(5.)),
    })
    .render(ColorOverLifetimeModifier { gradient })
}

fn propulsor_effect() -> EffectAsset {
    let mut gradient = Gradient::new();
    gradient.add_key(0., Color::rgba(1., 1., 0., 1.).into());
    gradient.add_key(1., Color::rgba(1., 0., 0., 0.).into());

    let spawner = Spawner::once(20.0.into(), false);

    EffectAsset {
        name: "Propulsor".into(),
        capacity: 32768,
        spawner,
        ..default()
    }
    .init(PositionSphereModifier {
        radius: 10.,
        speed: 10.0.into(),
        dimension: ShapeDimension::Surface,
        ..default()
    })
    .init(ParticleLifetimeModifier { lifetime: 0.5 })
    .render(SizeOverLifetimeModifier {
        gradient: Gradient::constant(Vec2::splat(4.)),
    })
    .render(ColorOverLifetimeModifier { gradient })
}

fn trigger_collision_effects(
    mut collision_events: EventReader<CollisionEvent>,
    mut effect: Query<
        (&mut ParticleEffect, &mut Transform),
        (With<CollisionEffect>, Without<Player>),
    >,
    player: Query<&Transform, With<Player>>,
) {
    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(..) = collision_event {
            let (mut effect, mut effect_transform) = effect.single_mut();
            let transform = player.single();
            effect_transform.translation = transform.translation;
            effect.maybe_spawner().unwrap().reset();
        }
    }
}

fn trigger_input_effects(
    // mut impulse_cooldown: Local<ImpulseCooldown>,
    mut input_events: EventReader<InputEvent>,
    mut explosion_effect: Query<
        (&mut ParticleEffect, &mut Transform),
        (
            With<ExplosionEffect>,
            Without<Player>,
            Without<PropulsorEffect>,
        ),
    >,
    mut propulsor_effect: Query<
        (&mut ParticleEffect, &mut Transform),
        (
            With<PropulsorEffect>,
            Without<Player>,
            Without<ExplosionEffect>,
        ),
    >,
    player: Query<&Transform, With<Player>>,
) {
    // impulse_cooldown.0.tick(time.delta());

    for input_event in input_events.iter() {
        match input_event {
            InputEvent::Impulse { direction } => {
                let (mut effect, mut effect_transform) = explosion_effect.single_mut();
                let transform = player.single();

                let player_body = Vec3::from((*direction * -PLAYER_RADIUS, 0.));
                effect_transform.translation = transform.translation + player_body;

                effect.maybe_spawner().unwrap().reset();
            }
            InputEvent::Stabilisation => {}
            InputEvent::Accelerate => {
                let (mut effect, mut effect_transform) = explosion_effect.single_mut();
                effect_transform.translation = player.single().translation;

                effect.maybe_spawner().unwrap().reset();
            }
            InputEvent::Force { direction } => {
                let (mut effect, mut effect_transform) = propulsor_effect.single_mut();
                let transform = player.single();

                let player_body = Vec3::from((*direction * -PLAYER_RADIUS, 0.));
                effect_transform.translation = transform.translation + player_body;
                effect.maybe_spawner().unwrap().reset();
            }
        }
    }
}
