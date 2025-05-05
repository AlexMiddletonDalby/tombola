# Tombola

Physics sequencer for external MIDI devices built in Bevy, inspired by
the [tombola sequencer mode on the Teenage Engineering OP-1](https://youtu.be/SHoDUCAd4-I?si=-XzA_HOYTEwAPaGe)

I build this for fun, mostly as a learning exercise and a proof of concept for a bigger sequencer project I'm working
on (watch this space), but feel free to clone it and have a play!

## Building

Tombola is built in rust using [Bevy](https://bevyengine.org), with [Avian](https://github.com/Jondolf/avian) for
physics, [bevy_egui](https://github.com/vladbat00/bevy_egui) for menus and [midir](https://docs.rs/midir/latest/midir/)
for midi handling.

Simply pull the project and
use [cargo](https://doc.rust-lang.org/book/ch01-03-hello-cargo.html#building-and-running-a-cargo-project) to build.

## Controls

- Left click to spawn balls
- Left click and drag to add initial velocity to the ball (most noticeable when gravity is disabled)
- Right click to clear all balls
- Mouse wheel to quickly change ball size

## How to use

- Connect an external midi device, such as a synthesiser or sampler (If you have multiple midi devices connected to your
  system, you can choose between them in the MIDI section of the settings)
- Use the quick menu on the right side of the window to choose between 3 ball sizes. The size of the ball affects the
  pitch of
  the note which it will trigger when it hits the tombola:
    - Small: MIDI octave 4
    - Medium: MIDI octave 3
    - Large: MIDI octave 2
- Each pad in the spinning tombola is assigned a note. When a ball hits the pad, it will send the corresponding MIDI
  note. By default,
  the velocity of the MIDI note is determined by the speed at which the ball hits the pad.
- Use the 'World' section of the settings menu to adjust the parameters of the simulation, including gravity,
  bounciness, and the shape and spin of the tombola.
- Use the 'MIDI' section of the settings menu to choose which notes are assigned to tombola pads, and tweak other
  settings about how notes are triggered

## Future development

As this was build mostly for fun, I may or may not continue to add things. Here's a short list of things I'd still like
to do:

- Upgrade to Bevy 16
- Add a basic built-in synth to remove the external MIDI device requirement
- Add CLAP/VST plugin hosting to allow you to bring your own soft-synth
- Improve MIDI velocity calculations, as these are currently quite crude
- Improve visuals with some shaders