use avian2d::parry::na::clamp;
use midir::MidiOutputConnection;

const NOTE_ON_MSG: u8 = 0x90;
const NOTE_OFF_MSG: u8 = 0x80;

#[derive(Clone, Copy)]
pub enum Note {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

const C3: u8 = 0x3C;
const D3: u8 = 0x3E;
const E3: u8 = 0x40;
const F3: u8 = 0x41;
const G3: u8 = 0x43;
const A3: u8 = 0x45;
const B3: u8 = 0x47;

fn to_note_value(note: Note, octave: i32) -> u8 {
    const BASE_OCTAVE: i32 = 3;
    const NOTES_PER_OCTAVE: i32 = 12;

    let base_note = match note {
        Note::C => C3,
        Note::D => D3,
        Note::E => E3,
        Note::F => F3,
        Note::G => G3,
        Note::A => A3,
        Note::B => B3,
    };

    let shift = octave - BASE_OCTAVE;
    base_note + (shift * NOTES_PER_OCTAVE) as u8
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
