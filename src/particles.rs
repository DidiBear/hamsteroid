use bevy::{
    prelude::*,
    render::{render_resource::WgpuFeatures, settings::WgpuSettings},
};

use bevy_hanabi::*;
use bevy_rapier2d::prelude::*;

use crate::{inputs::InputEvent, Player};

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

fn setup_particle_effects(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    commands
        .spawn()
        .insert(CollisionEffect)
        .insert(Name::new("Collision effect"))
        .insert_bundle(ParticleEffectBundle::new(effects.add(collision_effect())));

    commands
        .spawn()
        .insert(ExplosionEffect)
        .insert(Name::new("Explosion effect"))
        .insert_bundle(ParticleEffectBundle::new(effects.add(explosion_effect())));
}

fn collision_effect() -> EffectAsset {
    let mut gradient = Gradient::new();
    gradient.add_key(0., Color::GRAY.into());
    gradient.add_key(1., Color::BLACK.into());

    let spawner = Spawner::once(30.0.into(), false);

    EffectAsset {
        name: "Impact".into(),
        capacity: 32768,
        spawner,
        ..default()
    }
    .init(PositionSphereModifier {
        radius: 5.,
        speed: 100.0.into(),
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
    gradient.add_key(0., Color::rgba(1., 1., 0., 1.).into()); // yelow
    gradient.add_key(1., Color::rgba(1., 0., 0., 0.).into());

    let spawner = Spawner::once(100.0.into(), false);

    EffectAsset {
        name: "Explosion".into(),
        capacity: 32768,
        spawner,
        ..default()
    }
    .init(PositionSphereModifier {
        radius: 50.,
        speed: 200.0.into(),
        dimension: ShapeDimension::Surface,
        ..default()
    })
    .init(ParticleLifetimeModifier { lifetime: 1. })
    .render(SizeOverLifetimeModifier {
        gradient: Gradient::constant(Vec2::splat(5.)),
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
            println!("Received collision event: {:?}", collision_event);
            effect_transform.translation = transform.translation;
            effect.maybe_spawner().unwrap().reset();
        }
    }
}
fn trigger_input_effects(
    // mut impulse_cooldown: Local<ImpulseCooldown>,
    mut input_events: EventReader<InputEvent>,
    mut effect: Query<
        (&mut ParticleEffect, &mut Transform),
        (With<ExplosionEffect>, Without<Player>),
    >,
    player: Query<&Transform, With<Player>>,
) {
    // impulse_cooldown.0.tick(time.delta());

    for input_event in input_events.iter() {
        match input_event {
            InputEvent::Impulse { direction: _ } => {
                let (mut effect, mut effect_transform) = effect.single_mut();
                let transform = player.single();
                effect_transform.translation = transform.translation;
                effect.maybe_spawner().unwrap().reset();
            }
            InputEvent::Stabilisation => {}
            InputEvent::Accelerate => {}
            InputEvent::Force { direction: _ } => {}
        }
    }
}
