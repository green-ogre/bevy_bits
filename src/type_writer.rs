use bevy::prelude::*;
use std::time::Duration;

#[derive(Debug, Default, Resource)]
pub struct TypeWriter {
    finished: bool,
    timer: Timer,
    string: String,
    index: usize,
}

impl TypeWriter {
    #[inline]
    pub fn new(string: String, chars_per_sec: f32) -> Self {
        Self {
            timer: Timer::from_seconds(1.0 / chars_per_sec, TimerMode::Repeating),
            string: string.trim().into(),
            index: 0,
            finished: false,
        }
    }

    #[inline]
    pub fn tick(&mut self, time: &Time, on_increment: impl FnOnce()) {
        self.timer.tick(time.delta());

        if self.timer.just_finished() {
            if self.index + 1 >= self.string.len() {
                self.finished = true;
                self.timer.pause();
            } else {
                self.index += 1;
            }

            on_increment();
        }
    }

    #[inline]
    pub fn restart(&mut self) {
        self.reset().start();
    }

    #[inline]
    pub fn reset(&mut self) -> &mut Self {
        self.timer.pause();
        self.timer.reset();
        self.index = 0;
        self.finished = false;

        self
    }

    #[inline]
    pub fn with_string(&mut self, string: String) -> &mut Self {
        self.string = string;
        self
    }

    #[inline]
    pub fn with_speed(&mut self, chars_per_sec: f32) -> &mut Self {
        self.timer
            .set_duration(Duration::from_secs_f32(1.0 / chars_per_sec));
        self
    }

    #[inline]
    pub fn start(&mut self) {
        self.timer.unpause();
    }

    #[inline]
    pub fn finished(&self) -> bool {
        self.finished
    }

    #[inline]
    pub fn revealed_text(&self) -> &str {
        &self.string[0..self.index]
    }

    #[inline]
    pub fn revealed_text_with_line_wrap(&self) -> String {
        let mut slice = self.string[0..self.index].to_owned();
        let padding = self
            .string
            .chars()
            .enumerate()
            .skip(slice.len())
            .find_map(|(i, c)| {
                if c == ' ' {
                    Some(i - slice.len())
                } else {
                    None
                }
            })
            .unwrap_or(0);
        if slice.ends_with(' ') {
            for _ in 0..padding {
                slice.push(' ');
            }
        }

        slice
    }
}
