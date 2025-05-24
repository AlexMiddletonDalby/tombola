use crate::ball::Ball;
use crate::geometry;
use crate::midi;
use crate::midi::{MidiOutputEvent, Note};
use crate::pad::{Pad, PadBundle};
use crate::settings::Settings;
use avian2d::math::PI;
use avian2d::prelude::{
    AngularVelocity, CollisionEventsEnabled, CollisionStarted, LinearVelocity, RigidBody,
};
use bevy::math::ops::tan;
use bevy::prelude::*;
use std::time::Duration;

pub struct TombolaPlugin;

impl Plugin for TombolaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_default_tombola);
        app.add_systems(
            Update,
            (
                update_tombola_shape,
                update_tombola_notes.after(update_tombola_shape),
                update_tombola_spin,
                handle_pad_collisions,
                fade_pads,
                note_off_pads,
            ),
        );
    }
}

#[derive(Component)]
struct Tombola {
    shape: geometry::Shape,
}

fn spawn_tombola(
    commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    shape: geometry::Shape,
    spin: f32,
    bounciness: f32,
    notes: &Vec<midi::Note>,
) {
    const THICKNESS: f32 = 5.0;
    const APOTHEM: f32 = 225.0;

    let side_length = 2.0 * APOTHEM * tan(PI / shape.get_num_sides() as f32);
    let size = Vec2::new(side_length, THICKNESS);
    let position = Vec2::new(0.0, 0.0);

    commands
        .spawn((
            Tombola { shape },
            RigidBody::Kinematic,
            AngularVelocity(-spin),
            Transform::from_xyz(position.x, position.y, 0.0),
            Visibility::default(),
        ))
        .with_children(|commands| {
            let transforms = shape.get_side_transforms(position, APOTHEM);
            for (index, transform) in transforms.into_iter().enumerate() {
                commands.spawn((
                    PadBundle::new(
                        index,
                        Vec2::new(size.x + (THICKNESS / 2.0), size.y),
                        transform,
                        notes[index],
                        bounciness,
                        &mut meshes,
                        &mut materials,
                    ),
                    CollisionEventsEnabled,
                ));
            }
        });
}

fn spawn_default_tombola(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    settings: Res<Settings>,
) {
    spawn_tombola(
        &mut commands,
        &mut meshes,
        &mut materials,
        settings.world.tombola_shape,
        settings.world.tombola_spin,
        settings.world.bounciness,
        &settings.midi.tombola_notes,
    );
}

fn collide(
    pad: &mut Pad,
    ball: &mut Ball,
    ball_velocity: &LinearVelocity,
    midi: &mut EventWriter<MidiOutputEvent>,
    materials: &mut Assets<ColorMaterial>,
    settings: &Settings,
) {
    if pad.playing_notes.contains_key(&ball.size.to_octave()) {
        midi.write(MidiOutputEvent::NoteOff {
            note: pad.note,
            octave: ball.size.to_octave(),
        });
    }

    midi.write(MidiOutputEvent::NoteOn {
        note: pad.note,
        octave: ball.size.to_octave(),
        velocity: if settings.midi.fixed_note_velocity.enabled {
            settings.midi.fixed_note_velocity.value
        } else {
            midi::to_velocity(ball_velocity.length())
        },
    });

    pad.playing_notes.insert(
        ball.size.to_octave(),
        Timer::new(
            if settings.midi.fixed_note_length.enabled {
                Duration::from_millis(settings.midi.fixed_note_length.value)
            } else {
                midi::to_note_duration(ball_velocity.length())
            },
            TimerMode::Once,
        ),
    );

    if let Some(material) = materials.get_mut(pad.material.0.id()) {
        material.color = Pad::hit_color();
    }

    ball.bounces += 1;
}

fn handle_pad_collisions(
    mut collisions: EventReader<CollisionStarted>,
    mut midi: EventWriter<MidiOutputEvent>,
    mut pads: Query<&mut Pad>,
    mut balls: Query<(&mut Ball, &LinearVelocity)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    settings: Res<Settings>,
) {
    for CollisionStarted(entity1, entity2) in collisions.read() {
        if let Ok(mut pad) = pads.get_mut(*entity1) {
            if let Ok((mut ball, velocity)) = balls.get_mut(*entity2) {
                collide(
                    &mut pad,
                    &mut ball,
                    velocity,
                    &mut midi,
                    &mut materials,
                    &settings,
                );
            }
        }
        if let Ok(mut pad) = pads.get_mut(*entity2) {
            if let Ok((mut ball, velocity)) = balls.get_mut(*entity1) {
                collide(
                    &mut pad,
                    &mut ball,
                    velocity,
                    &mut midi,
                    &mut materials,
                    &settings,
                );
            }
        }
    }
}

fn update_tombola_shape(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut settings: ResMut<Settings>,
    tombola: Query<(Entity, &Tombola)>,
) {
    if let Ok((entity, tombola)) = tombola.single() {
        if tombola.shape != settings.world.tombola_shape {
            commands.entity(entity).despawn();

            let num_sides = settings.world.tombola_shape.get_num_sides();
            settings.midi.tombola_notes.resize(num_sides, Note::C);

            spawn_tombola(
                &mut commands,
                &mut meshes,
                &mut materials,
                settings.world.tombola_shape,
                settings.world.tombola_spin,
                settings.world.bounciness,
                &settings.midi.tombola_notes,
            );
        }
    }
}

fn update_tombola_spin(
    mut spin: Query<&mut AngularVelocity, With<Tombola>>,
    settings: Res<Settings>,
) {
    if let Ok(mut tombola) = spin.single_mut() {
        tombola.0 = -settings.world.tombola_spin;
    }
}

fn update_tombola_notes(mut pads: Query<&mut Pad>, settings: Res<Settings>) {
    for mut pad in pads.iter_mut() {
        pad.note = settings.midi.tombola_notes[pad.index];
    }
}

fn fade_pads(
    mut pads: Query<&mut Pad>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
) {
    const FADE_SPEED: f32 = 5.0;
    let amount = FADE_SPEED * time.delta_secs();

    for pad in pads.iter_mut() {
        if let Some(material) = materials.get_mut(pad.material.0.id()) {
            material.color = material.color.mix(&Pad::default_color(), amount);
        }
    }
}

fn note_off_pads(
    mut pads: Query<&mut Pad>,
    time: Res<Time>,
    mut midi: EventWriter<MidiOutputEvent>,
) {
    for mut pad in pads.iter_mut() {
        let note = pad.note.clone();

        for (_, timer) in pad.playing_notes.iter_mut() {
            timer.tick(time.delta());
        }

        pad.playing_notes.retain(|octave, timer| {
            if timer.just_finished() {
                midi.write(MidiOutputEvent::NoteOff {
                    note,
                    octave: octave.clone(),
                });
                return false;
            }

            true
        });
    }
}
