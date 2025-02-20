use avian2d::parry::na::clamp;
use midir::MidiOutputConnection;

const NOTE_ON_MSG: u8 = 0x90;
const NOTE_OFF_MSG: u8 = 0x80;

#[derive(Clone, Copy)]
pub enum Note {
    C,
    E,
    G,
    A,
}

pub const C3: u8 = 0x30;
pub const E3: u8 = 0x34;
pub const G3: u8 = 0x37;
pub const A3: u8 = 0x39;

pub const C4: u8 = 0x3c;
pub const E4: u8 = 0x40;
pub const G4: u8 = 0x43;
pub const A4: u8 = 0x45;

pub const C5: u8 = 0x48;
pub const E5: u8 = 0x4C;
pub const G5: u8 = 0x4F;
pub const A5: u8 = 0x51;

fn to_note_value(note: Note, octave: i32) -> u8 {
    if octave <= 3 {
        return match note {
            Note::C => C3,
            Note::E => E3,
            Note::G => G3,
            Note::A => A3,
        };
    }

    if octave == 4 {
        return match note {
            Note::C => C4,
            Note::E => E4,
            Note::G => G4,
            Note::A => A4,
        };
    }

    match note {
        Note::C => C5,
        Note::E => E5,
        Note::G => G5,
        Note::A => A5,
    }
}

pub fn note_on(
    note: Note,
    octave: i32,
    velocity: u8,
    midi_output: &mut Option<MidiOutputConnection>,
) {
    if let Some(midi_output) = midi_output {
        let _ = midi_output.send(&[NOTE_ON_MSG, to_note_value(note, octave), velocity]);
    }
}

pub fn note_off(note: Note, octave: i32, midi_output: &mut Option<MidiOutputConnection>) {
    const VELOCITY: u8 = 0x7F;

    if let Some(midi_output) = midi_output {
        let _ = midi_output.send(&[NOTE_OFF_MSG, to_note_value(note, octave), VELOCITY]);
    }
}

pub fn to_velocity(speed: f32) -> u8 {
    const MAX_SPEED: f32 = 750.0;
    const MIN_SPEED: f32 = 50.0;

    let scaled0to1 = (clamp(speed, MIN_SPEED, MAX_SPEED) - MIN_SPEED) / (MAX_SPEED - MIN_SPEED);

    const MAX_MIDI_VEL: u8 = 0x7F;
    let scaled = MAX_MIDI_VEL as f32 * scaled0to1;

    scaled as u8
}
