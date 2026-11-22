use bevy::prelude::*;
use midir::{MidiOutput, MidiOutputConnection};
use std::{
    fmt,
    sync::mpsc,
    thread::{self, JoinHandle},
    time::Duration,
};
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
        app.add_event::<MidiOutputEvent>();
        app.add_systems(
            Startup,
            (
                start_midi_thread,
                connect_to_first_available_midi_port.after(start_midi_thread),
            ),
        );
        app.add_systems(
            Update,
            (
                process_output_events,
                update_connection,
                terminate_midi_thread_if_app_exiting,
            ),
        );
    }
}

#[derive(Resource)]
pub struct Midi {
    thread: Option<JoinHandle<()>>,
    tx: mpsc::Sender<Action>,
    connected_port: Option<Port>,
    selected_port: Option<Port>,
}

impl Midi {
    pub fn get_available_ports(&self) -> Vec<Port> {
        let mut ports = Vec::new();

        let midi_out =
            MidiOutput::new("midir test output").expect("Should be able to create midi out");

        for port in midi_out.ports() {
            if let Ok(name) = midi_out.port_name(&port) {
                ports.push(Port {
                    name,
                    id: port.id(),
                });
            }
        }

        return ports;
    }

    pub fn get_port(&self) -> Option<Port> {
        return self.selected_port.clone();
    }

    pub fn set_port(&mut self, port: Port) {
        self.selected_port = Some(port);
    }
}

pub enum Action {
    ConnectToPort(Port),
    ClearConnection,
    SendMidiOutputEvent(MidiOutputEvent),
    SendMidiPanicEvent,
}

fn start_midi_thread(mut commands: Commands) {
    let (midi_tx, midi_rx): (mpsc::Sender<Action>, mpsc::Receiver<Action>) = mpsc::channel();

    commands.insert_resource(Midi {
        thread: Some(thread::spawn(move || midi_thread_process(midi_rx))),
        tx: midi_tx.clone(),
        connected_port: None,
        selected_port: None,
    });
}

fn midi_thread_process(rx: mpsc::Receiver<Action>) {
    let mut connection: Option<MidiOutputConnection> = None;

    for event in rx.iter() {
        match event {
            Action::ConnectToPort(port) => {
                println!("Attempting to connenct to midi port with id: {}", port.id);
                let midi_out = MidiOutput::new("My Test Output")
                    .expect("Should be able to make an output object");

                let out_ports = midi_out.ports();
                if let Some(midi_port) = out_ports.iter().find(|p| p.id() == port.id) {
                    connection = Some(
                        midi_out
                            .connect(midi_port, "my_port")
                            .expect("Should be able to connect to the port"),
                    );
                    println!("Successfully connected to: {}", port.name);
                } else {
                    println!("Error: requested port is not available");
                    connection = None;
                }
            }
            Action::ClearConnection => {
                connection = None;
            }
            Action::SendMidiOutputEvent(output_event) => {
                if let Some(conn) = &mut connection {
                    match output_event {
                        MidiOutputEvent::NoteOn {
                            note,
                            octave,
                            velocity,
                        } => {
                            let _ = conn.send(&[NOTE_ON_MSG, note.to_value(octave), velocity]);
                        }
                        MidiOutputEvent::NoteOff { note, octave } => {
                            let _ = conn.send(&[NOTE_OFF_MSG, note.to_value(octave), 0x7F]);
                        }
                    }
                }
            }
            Action::SendMidiPanicEvent => {
                if let Some(conn) = &mut connection {
                    let _ = conn.send(&[CC, PANIC, 0]);
                }
            }
        }
    }
}

fn connect_to_first_available_midi_port(mut midi: ResMut<Midi>) {
    let ports = midi.get_available_ports();
    if let Some(default_port) = ports.get(0) {
        let _ = midi.tx.send(Action::ConnectToPort(default_port.clone()));
        midi.connected_port = Some(default_port.clone());
        midi.selected_port = Some(default_port.clone());
    } else {
        let _ = midi.tx.send(Action::ClearConnection);
        midi.connected_port = None;
        midi.selected_port = None;
    }
}

fn update_connection(mut midi: ResMut<Midi>) {
    if let Some(selected_port) = midi.selected_port.clone() {
        if midi.get_available_ports().contains(&selected_port) {
            if midi.selected_port != midi.connected_port {
                let _ = midi.tx.send(Action::ConnectToPort(selected_port.clone()));
                midi.connected_port = Some(selected_port.clone());
            }
        } else {
            connect_to_first_available_midi_port(midi);
        }
    }
}

fn terminate_midi_thread_if_app_exiting(
    mut midi: ResMut<Midi>,
    mut exit_events: EventReader<AppExit>,
) {
    for _ in exit_events.read() {
        let _ = midi.tx.send(Action::SendMidiPanicEvent);
        if let Some(handle) = midi.thread.take() {
            let _ = handle.join();
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Port {
    pub name: String,
    pub id: String,
}

impl fmt::Display for Port {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
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

#[derive(Event, Clone)]
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

fn process_output_events(mut events: EventReader<MidiOutputEvent>, midi: ResMut<Midi>) {
    for event in events.read() {
        let _ = midi.tx.send(Action::SendMidiOutputEvent(event.clone()));
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
