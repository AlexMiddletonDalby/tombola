use bevy::prelude::Resource;

pub struct World {
    pub tombola_spin: f32,
    pub bounciness: f32,
    pub gravity: f32,
}

pub struct FixedNoteVelocity {
    pub enabled: bool,
    pub value: u8,
}

pub struct FixedNoteLength {
    pub enabled: bool,
    pub value: u64,
}

pub struct Midi {
    pub fixed_note_velocity: FixedNoteVelocity,
    pub fixed_note_length: FixedNoteLength,
}

#[derive(Resource)]
pub struct Settings {
    pub world: World,
    pub midi: Midi,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            world: World {
                tombola_spin: 1.5,
                bounciness: 1.0,
                gravity: 1.0,
            },
            midi: Midi {
                fixed_note_velocity: FixedNoteVelocity {
                    enabled: false,
                    value: 64,
                },
                fixed_note_length: FixedNoteLength {
                    enabled: false,
                    value: 100,
                },
            },
        }
    }
}
