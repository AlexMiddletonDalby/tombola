mod ball;
mod geometry;
mod midi;
mod pad;
mod size;
mod ui;

use avian2d::parry::na::clamp;
use avian2d::prelude::*;
use ball::{Ball, BallBundle};
use bevy::core_pipeline::bloom::Bloom;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};
use pad::{Pad, PadBundle};
use size::Size;
use std::time::Duration;
use ui::{BallSelector, BallSelectorBundle, Highlight, HighlightBundle};

#[derive(Resource)]
struct Midi {
    output_handle: Option<MidiOutputConnection>,
}

impl Drop for Midi {
    fn drop(&mut self) {
        midi::panic(&mut self.output_handle);
    }
}

#[derive(Resource, Default)]
struct WorldMouse {
    position: Vec2,
}

#[derive(Resource)]
struct SelectedBall {
    size: Size,
}

fn main() {
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

    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_systems(
            Startup,
            (
                spawn_tombola,
                setup_camera,
                spawn_ball_selectors.after(setup_camera),
            ),
        )
        .add_systems(
            Update,
            (
                update_world_mouse,
                handle_click,
                handle_collisions,
                note_off_pads,
                fade_pads,
                update_selector_positions,
                update_highlight.after(update_selector_positions),
                clean_up_balls,
            ),
        )
        .insert_resource(ClearColor(Color::linear_rgb(0., 0., 0.)))
        .insert_resource(Gravity(Vec2::NEG_Y * 700.0))
        .insert_resource(Midi {
            output_handle: handle,
        })
        .insert_resource(WorldMouse {
            position: Vec2::ZERO,
        })
        .insert_resource(SelectedBall { size: Size::Small })
        .run();
}

//--------------------------------------------------------------------------------------------------
//Startup

#[derive(Component)]
struct MainCamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera {
            hdr: true,
            ..default()
        },
        Camera2d,
        Bloom::OLD_SCHOOL,
        MainCamera,
    ));
}

fn spawn_tombola(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    const SIDE_LENGTH: f32 = 300.0;
    const THICKNESS: f32 = 5.0;

    let size = Vec2::new(SIDE_LENGTH, THICKNESS);
    let position = Vec2::new(0.0, 0.0);

    commands
        .spawn((
            RigidBody::Kinematic,
            AngularVelocity(1.5),
            Transform::from_xyz(position.x, position.y, 0.0),
            Visibility::default(),
        ))
        .with_children(|commands| {
            let transforms = geometry::hexagon(position, SIDE_LENGTH);
            let notes = vec![
                midi::Note::C,
                midi::Note::E,
                midi::Note::G,
                midi::Note::ASharp,
                midi::Note::D,
                midi::Note::F,
            ];

            for (index, transform) in transforms.into_iter().enumerate() {
                commands.spawn(PadBundle::new(
                    size,
                    transform,
                    notes[index],
                    &mut meshes,
                    &mut materials,
                ));
            }
        });
}

fn get_ball_selector_x(window: &Window) -> f32 {
    const SPACING: f32 = 50.0;

    window.width() / 2.0 - SPACING
}

fn spawn_ball_selectors(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let x_pos = get_ball_selector_x(window.single());

    commands.spawn(HighlightBundle::new(
        Vec2::new(x_pos, 100.0),
        &mut meshes,
        &mut materials,
    ));

    commands.spawn(BallSelectorBundle::new(
        Size::Small,
        Vec2::new(x_pos, 100.0),
        &mut meshes,
        &mut materials,
    ));

    commands.spawn(BallSelectorBundle::new(
        Size::Medium,
        Vec2::new(x_pos, 0.0),
        &mut meshes,
        &mut materials,
    ));

    commands.spawn(BallSelectorBundle::new(
        Size::Large,
        Vec2::new(x_pos, -100.0),
        &mut meshes,
        &mut materials,
    ));
}

//--------------------------------------------------------------------------------------------------
//Update

fn update_world_mouse(
    mut world_mouse: ResMut<WorldMouse>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = camera.single();
    let window = window.single();

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
            world_mouse.position = world_pos.origin.truncate();
        }
    }
}

fn update_selector_positions(
    mut selectors: Query<&mut Transform, With<BallSelector>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let x_pos = get_ball_selector_x(window.single());

    for mut selector in selectors.iter_mut() {
        selector.translation.x = x_pos;
    }
}

fn find_selector_position(
    selectors: &Vec<(&BallSelector, &Transform)>,
    selected: Size,
) -> Option<Vec2> {
    if let Some((_found, transform)) = selectors
        .iter()
        .find(|(selector, _)| selector.size == selected)
    {
        return Some(transform.translation.truncate());
    }

    None
}

fn update_highlight(
    mut highlight: Query<&mut Transform, With<Highlight>>,
    selectors: Query<(&BallSelector, &Transform), Without<Highlight>>,
    selected_ball: Res<SelectedBall>,
) {
    if let Ok(mut highlight) = highlight.get_single_mut() {
        if let Some(pos) = find_selector_position(&selectors.iter().collect(), selected_ball.size) {
            highlight.translation.x = pos.x;
            highlight.translation.y = pos.y;
        }
    }
}

fn pick_selector(selectors: Query<(&BallSelector, &Transform)>, pos: Vec2) -> Option<Size> {
    for (selector, transform) in selectors.iter() {
        let centre = transform.translation.truncate();

        let rect = Rect::new(
            centre.x + BallSelector::hitbox_size(),
            centre.y + BallSelector::hitbox_size(),
            centre.x - BallSelector::hitbox_size(),
            centre.y - BallSelector::hitbox_size(),
        );

        if rect.contains(pos) {
            return Some(selector.size);
        }
    }

    None
}

fn handle_click(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut selected_ball: ResMut<SelectedBall>,
    world_mouse: Res<WorldMouse>,
    buttons: Res<ButtonInput<MouseButton>>,
    selectors: Query<(&BallSelector, &Transform)>,
    balls: Query<Entity, With<Ball>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(selector) = pick_selector(selectors, world_mouse.position) {
            selected_ball.size = selector
        } else {
            commands.spawn(BallBundle::new(
                world_mouse.position,
                selected_ball.size,
                &mut meshes,
                &mut materials,
            ));
        }
    } else if buttons.just_pressed(MouseButton::Right) {
        for ball in balls.iter() {
            commands.entity(ball).despawn();
        }
    }
}

fn to_note_duration(speed: f32) -> Duration {
    const MAX_SPEED: f32 = 750.0;
    const MIN_SPEED: f32 = 50.0;

    let scaled0to1 = (clamp(speed, MIN_SPEED, MAX_SPEED) - MIN_SPEED) / (MAX_SPEED - MIN_SPEED);

    const MAX_DURATION: i32 = 400;
    let scaled = MAX_DURATION as f32 * scaled0to1;

    Duration::from_millis(scaled as u64)
}

fn collide(
    collision: &Contacts,
    pad: &mut Pad,
    ball: &Ball,
    ball_velocity: &LinearVelocity,
    mut midi_output: &mut Option<MidiOutputConnection>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    if collision.collision_started() {
        if pad.playing_notes.contains_key(&ball.size.to_octave()) {
            midi::note_off(pad.note, ball.size.to_octave(), &mut midi_output);
        }

        midi::note_on(
            pad.note,
            ball.size.to_octave(),
            midi::to_velocity(ball_velocity.length()),
            &mut midi_output,
        );

        pad.playing_notes.insert(
            ball.size.to_octave(),
            Timer::new(to_note_duration(ball_velocity.length()), TimerMode::Once),
        );

        if let Some(material) = materials.get_mut(pad.material.0.id()) {
            material.color = Pad::hit_color();
        }
    }
}

fn handle_collisions(
    mut collisions: EventReader<Collision>,
    mut pads: Query<&mut Pad>,
    balls: Query<(&Ball, &LinearVelocity)>,
    mut midi: ResMut<Midi>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for Collision(collision) in collisions.read() {
        if let Ok(mut pad) = pads.get_mut(collision.entity1) {
            if let Ok((ball, velocity)) = balls.get(collision.entity2) {
                collide(
                    collision,
                    &mut pad,
                    ball,
                    velocity,
                    &mut midi.output_handle,
                    &mut materials,
                );
            }
        }
        if let Ok(mut pad) = pads.get_mut(collision.entity2) {
            if let Ok((ball, velocity)) = balls.get(collision.entity1) {
                collide(
                    collision,
                    &mut pad,
                    ball,
                    velocity,
                    &mut midi.output_handle,
                    &mut materials,
                );
            }
        }
    }
}

fn note_off_pads(mut pads: Query<&mut Pad>, time: Res<Time>, mut midi: ResMut<Midi>) {
    for mut pad in pads.iter_mut() {
        let note = pad.note.clone();

        for (_, timer) in pad.playing_notes.iter_mut() {
            timer.tick(time.delta());
        }

        pad.playing_notes.retain(|octave, timer| {
            if timer.just_finished() {
                midi::note_off(note, octave.clone(), &mut midi.output_handle);
                return false;
            }

            true
        });
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

fn clean_up_balls(
    mut commands: Commands,
    mut balls: Query<(Entity, &Transform), With<Ball>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window.single();
    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;

    let rect = Rect::new(-half_width, -half_height, half_width, half_height);

    for (ball, transform) in balls.iter_mut() {
        if !rect.contains(transform.translation.truncate()) {
            commands.entity(ball).despawn();
        }
    }
}
