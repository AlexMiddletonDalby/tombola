use avian2d::parry::na::clamp;
use midir::MidiOutputConnection;

const NOTE_ON_MSG: u8 = 0x90;
const NOTE_OFF_MSG: u8 = 0x80;
const CC: u8 = 0xB0;

#[derive(Clone, Copy)]
pub enum Note {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

const C3: u8 = 0x3C;
const C_SHARP3: u8 = 0x3D;
const D3: u8 = 0x3E;
const D_SHARP3: u8 = 0x3F;
const E3: u8 = 0x40;
const F3: u8 = 0x41;
const F_SHARP3: u8 = 0x42;
const G3: u8 = 0x43;
const G_SHARP3: u8 = 0x44;
const A3: u8 = 0x45;
const A_SHARP3: u8 = 0x46;
const B3: u8 = 0x47;

const PANIC: u8 = 0x7B;

fn to_note_value(note: Note, octave: i32) -> u8 {
    const BASE_OCTAVE: i32 = 3;
    const NOTES_PER_OCTAVE: i32 = 12;

    let base_note = match note {
        Note::C => C3,
        Note::CSharp => C_SHARP3,
        Note::D => D3,
        Note::DSharp => D_SHARP3,
        Note::E => E3,
        Note::F => F3,
        Note::FSharp => F_SHARP3,
        Note::G => G3,
        Note::GSharp => G_SHARP3,
        Note::A => A3,
        Note::ASharp => A_SHARP3,
        Note::B => B3,
    };

    let shift = octave - BASE_OCTAVE;
    (base_note as i32 + (shift * NOTES_PER_OCTAVE)) as u8
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

pub fn panic(midi_output: &mut Option<MidiOutputConnection>) {
    if let Some(midi_output) = midi_output {
        let _ = midi_output.send(&[CC, PANIC, 0]);
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
