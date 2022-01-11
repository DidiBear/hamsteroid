use bevy::{prelude::*, utils::HashSet};

pub struct InputsPlugin;

impl Plugin for InputsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<InputEvent>()
            .init_resource::<GamepadLobby>()
            .add_system_to_stage(CoreStage::PreUpdate, connection_system.system())
            .add_system(gamepad_system.system())
            .add_system(keyboard_system.system());
    }
}

pub enum InputEvent {
    Movement { direction: Vec2 },
    Stabilisation,
    Accelerate,
}

#[derive(Default)]
struct GamepadLobby {
    gamepads: HashSet<Gamepad>,
}

fn connection_system(
    mut lobby: ResMut<GamepadLobby>,
    mut gamepad_event: EventReader<GamepadEvent>,
) {
    for event in gamepad_event.iter() {
        match &event {
            GamepadEvent(gamepad, GamepadEventType::Connected) => {
                lobby.gamepads.insert(*gamepad);
                println!("{:?} Connected", gamepad);
            }
            GamepadEvent(gamepad, GamepadEventType::Disconnected) => {
                lobby.gamepads.remove(gamepad);
                println!("{:?} Disconnected", gamepad);
            }
            _ => (),
        }
    }
}

fn gamepad_system(
    lobby: Res<GamepadLobby>,
    button_inputs: Res<Input<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut input_events: EventWriter<InputEvent>,
) {
    for gamepad in lobby.gamepads.iter().cloned() {
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

            input_events.send(InputEvent::Movement { direction })
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
        input_events.send(InputEvent::Movement {
            direction: keyboard_direction(&keyboard_inputs),
        })
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
