use crate::midi;
use bevy::prelude::Resource;

pub struct NumBallsLimit {
    pub enabled: bool,
    pub limit: usize,
}

pub struct BounceLimit {
    pub enabled: bool,
    pub limit: usize,
}

pub struct World {
    pub tombola_spin: f32,
    pub bounciness: f32,
    pub gravity: f32,
    pub max_balls: NumBallsLimit,
    pub max_bounces: BounceLimit,
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
    pub tombola_notes: Vec<midi::Note>,
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
                max_balls: NumBallsLimit {
                    enabled: false,
                    limit: 10,
                },
                max_bounces: BounceLimit {
                    enabled: false,
                    limit: 5,
                },
            },
            midi: Midi {
                tombola_notes: vec![
                    midi::Note::C,
                    midi::Note::E,
                    midi::Note::G,
                    midi::Note::ASharp,
                    midi::Note::D,
                    midi::Note::F,
                ],
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
