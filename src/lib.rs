use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};

// pub mod pixel_perfect;
// pub mod animated_sprites;
pub mod text;
// pub use tokens::*;

pub fn close_on_escape(mut input: EventReader<KeyboardInput>, mut writer: EventWriter<AppExit>) {
    for e in input.read() {
        if matches!(e, KeyboardInput {
            key_code,
            state,
            ..
        }
            if *key_code == KeyCode::Escape && *state == ButtonState::Pressed
        ) {
            writer.send(AppExit::Success);
        }
    }
}
