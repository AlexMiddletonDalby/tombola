use bevy::prelude::*;
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};
use std::time::Duration;
use strum_macros::EnumIter;

const NOTE_ON_MSG: u8 = 0x90;
const NOTE_OFF_MSG: u8 = 0x80;
const CC: u8 = 0xB0;
const PANIC: u8 = 0x7B;

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

pub struct MidiPlugin;

impl Plugin for MidiPlugin {
    fn build(&self, app: &mut App) {
        let mut handle: Option<MidiOutputConnection> = None;

        let midi_out = MidiOutput::new("My Test Output").unwrap();
        let out_ports = midi_out.ports();
        let port: Option<&MidiOutputPort> = out_ports.get(0);
        if port.is_some() {
            println!(
                "Acquired MIDI port: {}",
                midi_out.port_name(port.unwrap()).unwrap()
            );

            if let Ok(connect_result) = midi_out.connect(port.unwrap(), "test") {
                handle = Some(connect_result);
            } else {
                println!("Failed to connect to MIDI port");
            }
        } else {
            println!("Failed to acquire MIDI port");
        }

        app.insert_resource(Midi {
            output_handle: handle,
        });
        app.add_event::<MidiOutputEvent>();
        app.add_systems(Update, process_output_events);
    }
}

#[derive(Resource)]
struct Midi {
    output_handle: Option<MidiOutputConnection>,
}

impl Drop for Midi {
    fn drop(&mut self) {
        if let Some(midi_output) = &mut self.output_handle {
            let _ = midi_output.send(&[CC, PANIC, 0]);
        }
    }
}

#[derive(Clone, Copy, PartialEq, EnumIter)]
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

impl Note {
    fn to_value(&self, octave: i32) -> u8 {
        const BASE_OCTAVE: i32 = 3;
        const NOTES_PER_OCTAVE: i32 = 12;

        let base_note = match self {
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

    pub fn to_string(&self) -> String {
        match self {
            Note::C => "C".to_owned(),
            Note::CSharp => "C#".to_owned(),
            Note::D => "D".to_owned(),
            Note::DSharp => "Eb".to_owned(),
            Note::E => "E".to_owned(),
            Note::F => "F".to_owned(),
            Note::FSharp => "F#".to_owned(),
            Note::G => "G".to_owned(),
            Note::GSharp => "G#".to_owned(),
            Note::A => "A".to_owned(),
            Note::ASharp => "Bb".to_owned(),
            Note::B => "B".to_owned(),
        }
    }
}

#[derive(Event)]
pub enum MidiOutputEvent {
    NoteOn {
        note: Note,
        octave: i32,
        velocity: u8,
    },
    NoteOff {
        note: Note,
        octave: i32,
    },
}

fn process_output_events(mut events: EventReader<MidiOutputEvent>, mut midi: ResMut<Midi>) {
    for event in events.read() {
        if let Some(output) = &mut midi.output_handle {
            match event {
                MidiOutputEvent::NoteOn {
                    note,
                    octave,
                    velocity,
                } => {
                    let _ = output.send(&[NOTE_ON_MSG, note.to_value(*octave), *velocity]);
                }
                MidiOutputEvent::NoteOff { note, octave } => {
                    let _ = output.send(&[NOTE_OFF_MSG, note.to_value(*octave), 0x7F]);
                }
            }
        }
    }
}

pub fn to_velocity(speed: f32) -> u8 {
    const MAX_SPEED: f32 = 750.0;
    const MIN_SPEED: f32 = 50.0;

    let scaled0to1 = (speed.clamp(MIN_SPEED, MAX_SPEED) - MIN_SPEED) / (MAX_SPEED - MIN_SPEED);

    const MAX_MIDI_VEL: u8 = 0x7F;
    let scaled = MAX_MIDI_VEL as f32 * scaled0to1;

    scaled as u8
}

pub fn to_note_duration(speed: f32) -> Duration {
    const MAX_SPEED: f32 = 750.0;
    const MIN_SPEED: f32 = 50.0;

    let scaled0to1 = (speed.clamp(MIN_SPEED, MAX_SPEED) - MIN_SPEED) / (MAX_SPEED - MIN_SPEED);

    const MAX_DURATION: i32 = 400;
    let scaled = MAX_DURATION as f32 * scaled0to1;

    Duration::from_millis(scaled as u64)
}
