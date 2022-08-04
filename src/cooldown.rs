//! Manage cooldown timers.

use std::time::Duration;

use bevy::prelude::*;

#[derive(Clone, Debug)]
pub struct Cooldown {
    timer: Timer,
}

impl Cooldown {
    pub fn from_seconds(seconds: f32) -> Self {
        let mut timer = Timer::from_seconds(seconds, false);
        // Start as available
        timer.tick(Duration::from_secs_f32(seconds));

        Self { timer }
    }

    pub fn start(&mut self) {
        self.timer.reset();
    }

    pub fn tick(&mut self, delta: Duration) -> &Self {
        self.timer.tick(delta);
        self
    }

    pub fn finished(&self) -> bool {
        self.timer.finished()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Cooldown;

    #[test]
    fn test_cooldown() {
        let mut cd = Cooldown::from_seconds(2.0);
        assert_eq!(cd.finished(), true);

        // Start the cooldown
        cd.start();
        assert_eq!(cd.finished(), false);
        assert_eq!(cd.tick(Duration::from_secs_f32(0.75)).finished(), false);
        assert_eq!(cd.tick(Duration::from_secs_f32(1.5)).finished(), true);
        assert_eq!(cd.tick(Duration::from_secs_f32(10.0)).finished(), true);

        // Re-start the cooldown
        cd.start();
        assert_eq!(cd.finished(), false);
        assert_eq!(cd.tick(Duration::from_secs_f32(0.75)).finished(), false);
        assert_eq!(cd.tick(Duration::from_secs_f32(0.75)).finished(), false);
        assert_eq!(cd.tick(Duration::from_secs_f32(0.75)).finished(), true);
    }
}
