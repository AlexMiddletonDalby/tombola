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
        app.insert_resource(MidiHandle(None));
        app.insert_resource(MidiConfig {
            active_port: String::new(),
            port_watcher: MidiOutput::new("port_watcher").unwrap(),
        });
        app.add_event::<MidiOutputEvent>();
        app.add_systems(Startup, connect_to_default_output_port);
        app.add_systems(Update, (process_output_events, update_midi_connection));
    }
}

pub struct Port {
    pub name: String,
    pub port: MidiOutputPort,
}

struct MidiConnection {
    pub connection: MidiOutputConnection,
    pub port_name: String,
}

impl Drop for MidiConnection {
    fn drop(&mut self) {
        let _ = self.connection.send(&[CC, PANIC, 0]);
    }
}

#[derive(Resource)]
struct MidiHandle(Option<MidiConnection>);

impl MidiHandle {
    pub fn connect_to(&mut self, port: &Port) {
        let output = MidiOutput::new("Output").unwrap();
        if let Ok(connection) = output.connect(&port.port, "Connection") {
            self.0 = Some(MidiConnection {
                connection,
                port_name: port.name.clone(),
            });
            println!("Connected to {}", port.name);
        } else {
            println!("Failed to connect to {}", port.name)
        }
    }
}

#[derive(Resource)]
pub struct MidiConfig {
    pub active_port: String,
    port_watcher: MidiOutput,
}

impl MidiConfig {
    pub fn get_ports(&self) -> Vec<Port> {
        let mut ports = Vec::new();
        for port in self.port_watcher.ports() {
            if let Ok(name) = self.port_watcher.port_name(&port) {
                ports.push(Port { port, name });
            }
        }

        ports
    }
}

fn connect_to_default_output_port(mut midi: ResMut<MidiHandle>, mut config: ResMut<MidiConfig>) {
    let ports = config.get_ports();
    if let Some(default_port) = ports.get(0) {
        midi.connect_to(&default_port);
        config.active_port = default_port.name.clone();
    } else {
        config.active_port = String::new();
        println!("No MIDI ports available");
    }
}

fn update_midi_connection(config: Res<MidiConfig>, mut handle: ResMut<MidiHandle>) {
    if config.active_port.is_empty() {
        return;
    }

    if let Some(connection) = &mut handle.0 {
        if connection.port_name != config.active_port {
            if let Some(port) = config
                .get_ports()
                .iter()
                .find(|port| port.name == config.active_port)
            {
                handle.connect_to(port);
            }
        }
    } else {
        println!(
            "No current connection, connecting to port {}",
            config.active_port
        );
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

fn process_output_events(mut events: EventReader<MidiOutputEvent>, mut midi: ResMut<MidiHandle>) {
    for event in events.read() {
        if let Some(handle) = &mut midi.0 {
            match event {
                MidiOutputEvent::NoteOn {
                    note,
                    octave,
                    velocity,
                } => {
                    let _ =
                        handle
                            .connection
                            .send(&[NOTE_ON_MSG, note.to_value(*octave), *velocity]);
                }
                MidiOutputEvent::NoteOff { note, octave } => {
                    let _ = handle
                        .connection
                        .send(&[NOTE_OFF_MSG, note.to_value(*octave), 0x7F]);
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
