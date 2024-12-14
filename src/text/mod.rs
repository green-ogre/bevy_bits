use bevy::prelude::*;
use std::borrow::Cow;

mod parse;

#[derive(Debug, Clone, Component)]
pub struct TypeWriterSection {
    pub text: Text,
    pub commands: &'static [IndexedCommand],
}

impl TypeWriterSection {
    pub fn len(&self) -> usize {
        self.text.value.len()
    }

    pub fn is_empty(&self) -> bool {
        self.text.value.is_empty()
    }
}

impl From<&'static str> for TypeWriterSection {
    fn from(value: &'static str) -> Self {
        Self {
            text: Text::from(value),
            commands: &[],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IndexedCommand {
    pub index: usize,
    pub command: TypeWriterCommand,
}

#[derive(Debug, Clone, Copy)]
pub enum TypeWriterCommand {
    Clear,
    AwaitClear,
    ClearAfter(f32),
    /// Relative speed
    Speed(f32),
    Pause(f32),
    Delete(usize),
}

/// String with a collection of modifiers.
#[derive(Debug, Clone)]
pub struct Text {
    pub value: Cow<'static, str>,
    pub modifiers: &'static [IndexedTextMod],
}

impl Text {
    pub fn from_value(value: String) -> Self {
        Self {
            value: Cow::Owned(value),
            modifiers: &[],
        }
    }
}

impl From<String> for Text {
    fn from(value: String) -> Self {
        Self {
            value: Cow::Owned(value),
            modifiers: &[],
        }
    }
}

impl From<&'static str> for Text {
    fn from(value: &'static str) -> Self {
        Self {
            value: Cow::Borrowed(value),
            modifiers: &[],
        }
    }
}

/// Text modifier that applies to a [`Text`] section.
#[derive(Debug, Clone)]
pub struct IndexedTextMod {
    pub start: usize,
    /// Non inclusive
    pub end: usize,
    pub text_mod: TextMod,
}

/// Modifies visual qualities of [`Text`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextMod {
    Color(Srgba),
    Wave,
    Shake,
}

impl TextMod {
    pub fn is_shader_effect(&self) -> bool {
        match self {
            Self::Wave => true,
            Self::Shake => true,
            Self::Color(_) => false,
        }
    }
}
