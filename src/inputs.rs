use bevy::prelude::*;

pub struct InputsPlugin;

impl Plugin for InputsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InputEvent>()
            .add_system(gamepad_system)
            .add_system(keyboard_system);
    }
}

pub enum InputEvent {
    Impulse { direction: Vec2 },
    Force { direction: Vec2 },
    Stabilisation,
    Accelerate,
}

fn gamepad_system(
    gamepads: Res<Gamepads>,
    button_inputs: Res<Input<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut input_events: EventWriter<InputEvent>,
) {
    for gamepad in gamepads.iter().cloned() {
        let south_button = GamepadButton(gamepad, GamepadButtonType::South);
        if button_inputs.just_pressed(south_button) {
            input_events.send(InputEvent::Stabilisation);
        }
        if button_inputs.just_released(south_button) {
            dbg!("pressed south !");
            let value_at = |axis| axes.get(GamepadAxis(gamepad, axis)).unwrap();

            let x = value_at(GamepadAxisType::LeftStickX);
            let y = value_at(GamepadAxisType::LeftStickY);

            let direction = Vec2::new(x, y).normalize();
            dbg!(direction);

            input_events.send(InputEvent::Impulse { direction })
        }
    }
}

fn keyboard_system(
    keyboard_inputs: Res<Input<KeyCode>>,
    mut input_events: EventWriter<InputEvent>,
) {
    if keyboard_inputs.just_pressed(KeyCode::A) {
        input_events.send(InputEvent::Accelerate);
    }
    if keyboard_inputs.just_pressed(KeyCode::Space) {
        input_events.send(InputEvent::Stabilisation);
    }
    if keyboard_inputs.just_released(KeyCode::Space) {
        let direction = keyboard_direction(&keyboard_inputs);
        if direction != Vec2::ZERO {
            input_events.send(InputEvent::Impulse { direction })
        }
    }
    if !keyboard_inputs.pressed(KeyCode::Space) {
        let direction = keyboard_direction(&keyboard_inputs);
        if direction != Vec2::ZERO {
            input_events.send(InputEvent::Force { direction })
        }
    }
}

fn keyboard_direction(keyboard_inputs: &Input<KeyCode>) -> Vec2 {
    let mut direction = Vec2::ZERO;
    if keyboard_inputs.pressed(KeyCode::Up) {
        direction += Vec2::new(0., 1.);
    }
    if keyboard_inputs.pressed(KeyCode::Down) {
        direction += Vec2::new(0., -1.);
    }
    if keyboard_inputs.pressed(KeyCode::Left) {
        direction += Vec2::new(-1., 0.);
    }
    if keyboard_inputs.pressed(KeyCode::Right) {
        direction += Vec2::new(1., 0.);
    }
    direction.normalize_or_zero()
}
