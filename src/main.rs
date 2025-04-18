mod ball;
mod geometry;
mod midi;
mod pad;
mod settings;
mod size;
mod ui;

use crate::midi::Note;
use crate::ui::CursorBundle;
use avian2d::math::PI;
use avian2d::prelude::*;
use ball::{Ball, BallBundle};
use bevy::core_pipeline::bloom::Bloom;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::math::ops::tan;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPreUpdateSet};
use midi::{MidiOutputEvent, MidiPlugin};
use pad::{Pad, PadBundle};
use settings::Settings;
use size::Size;
use std::time::{Duration, SystemTime};
use ui::{BallSelector, BallSelectorBundle, Highlight, HighlightBundle};

#[derive(Resource, Default)]
struct WorldMouse {
    position: Vec2,
}

#[derive(Resource)]
struct SelectedBall {
    size: Size,
}

fn get_gravity(gravity_factor: f32) -> Vec2 {
    Vec2::NEG_Y * 700.0 * gravity_factor
}

fn main() {
    let settings = Settings::default();

    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            EguiPlugin,
            MidiPlugin,
        ))
        .add_systems(
            Startup,
            (
                spawn_default_tombola,
                setup_camera,
                spawn_ball_selectors.after(setup_camera),
                spawn_cursor,
            ),
        )
        .add_systems(
            Update,
            (
                update_world_mouse,
                handle_click.after(EguiPreUpdateSet::InitContexts),
                handle_scroll,
                handle_collisions,
                note_off_pads,
                fade_pads,
                update_selector_positions,
                update_highlight.after(update_selector_positions),
                update_cursor_size,
                update_cursor_position,
                update_cursor_visibility.after(update_cursor_position),
                clean_up_balls,
                update_gravity,
                update_tombola_shape,
                update_tombola_notes.after(update_tombola_shape),
                update_tombola_spin,
                update_bounciness,
            ),
        )
        .insert_resource(ClearColor(Color::linear_rgb(0., 0., 0.)))
        .insert_resource(Gravity(get_gravity(settings.world.gravity)))
        .insert_resource(WorldMouse {
            position: Vec2::ZERO,
        })
        .insert_resource(SelectedBall { size: Size::Small })
        .insert_resource(settings)
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
                commands.spawn(PadBundle::new(
                    index,
                    Vec2::new(size.x + (THICKNESS / 2.0), size.y),
                    transform,
                    notes[index],
                    bounciness,
                    &mut meshes,
                    &mut materials,
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

fn spawn_cursor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    selected_ball: Res<SelectedBall>,
) {
    commands.spawn(CursorBundle::new(
        selected_ball.size,
        Vec2::default(),
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
    if let Ok(window) = window.get_single() {
        if let Some(cursor_pos) = window.cursor_position() {
            if let Ok(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                world_mouse.position = world_pos.origin.truncate();
            }
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

fn update_highlight(
    mut highlight: Query<&mut Transform, With<Highlight>>,
    selectors: Query<(&BallSelector, &Transform), Without<Highlight>>,
    selected_ball: Res<SelectedBall>,
) {
    if let Ok(mut highlight) = highlight.get_single_mut() {
        if let Some(pos) =
            ui::find_selector_position(&selectors.iter().collect(), selected_ball.size)
        {
            highlight.translation.x = pos.x;
            highlight.translation.y = pos.y;
        }
    }
}

fn update_cursor_position(
    mut cursors: Query<&mut Transform, With<ui::Cursor>>,
    world_mouse: Res<WorldMouse>,
) {
    if let Ok(mut cursor) = cursors.get_single_mut() {
        cursor.translation.x = world_mouse.position.x;
        cursor.translation.y = world_mouse.position.y;
    }
}

fn update_cursor_visibility(
    mut cursors: Query<&mut Visibility, With<ui::Cursor>>,
    window: Query<&Window, With<PrimaryWindow>>,
    world_mouse: Res<WorldMouse>,
    selectors: Query<(&BallSelector, &Transform)>,
    mut egui: EguiContexts,
) {
    if let Ok(window) = window.get_single() {
        if let Ok(mut cursor_visibility) = cursors.get_single_mut() {
            let is_over_ui = egui.ctx_mut().is_pointer_over_area();
            let is_off_screen = window.cursor_position().is_none();
            let is_over_selector =
                ui::pick_selector(&selectors.iter().collect(), world_mouse.position).is_some();

            *cursor_visibility = if is_over_ui || is_off_screen || is_over_selector {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        }
    }
}

fn update_cursor_size(
    mut cursors: Query<(
        &mut ui::Cursor,
        &mut Mesh2d,
        &mut MeshMaterial2d<ColorMaterial>,
    )>,
    selected_ball: Res<SelectedBall>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if let Ok((mut cursor, mut mesh, mut material)) = cursors.get_single_mut() {
        cursor.size = selected_ball.size;
        *mesh = cursor.get_mesh(&mut meshes);
        *material = cursor.get_material(&mut materials);
    }
}

fn handle_click(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut selected_ball: ResMut<SelectedBall>,
    mut settings: ResMut<Settings>,
    world_mouse: Res<WorldMouse>,
    buttons: Res<ButtonInput<MouseButton>>,
    selectors: Query<(&BallSelector, &Transform)>,
    balls: Query<(Entity, &Ball)>,
    egui: EguiContexts,
) {
    let handled = ui::show_settings_menu(egui, settings.as_mut());
    if handled {
        return;
    }

    if buttons.just_pressed(MouseButton::Left) {
        if let Some(selector) = ui::pick_selector(&selectors.iter().collect(), world_mouse.position)
        {
            selected_ball.size = selector
        } else {
            commands.spawn(BallBundle::new(
                world_mouse.position,
                selected_ball.size,
                settings.world.bounciness,
                &mut meshes,
                &mut materials,
            ));
        }
    } else if buttons.just_pressed(MouseButton::Right) {
        for (entity, _ball) in balls.iter() {
            commands.entity(entity).despawn();
        }
    }
}

fn handle_scroll(mut scrolls: EventReader<MouseWheel>, mut selected_ball: ResMut<SelectedBall>) {
    for event in scrolls.read() {
        match event.unit {
            MouseScrollUnit::Line => {
                if event.y < -0.099 {
                    selected_ball.size = selected_ball.size.increment();
                } else if event.y > 0.099 {
                    selected_ball.size = selected_ball.size.decrement();
                }
            }
            MouseScrollUnit::Pixel => {
                if event.y < -50.0 {
                    selected_ball.size = selected_ball.size.increment();
                } else if event.y > 50.0 {
                    selected_ball.size = selected_ball.size.decrement();
                }
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
    if let Ok((entity, tombola)) = tombola.get_single() {
        if tombola.shape != settings.world.tombola_shape {
            commands.entity(entity).despawn_recursive();

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
    if let Ok(mut tombola) = spin.get_single_mut() {
        tombola.0 = -settings.world.tombola_spin;
    }
}

fn update_tombola_notes(mut pads: Query<&mut Pad>, settings: Res<Settings>) {
    for mut pad in pads.iter_mut() {
        pad.note = settings.midi.tombola_notes[pad.index];
    }
}

fn update_bounciness(mut bouncy_things: Query<&mut Restitution>, settings: Res<Settings>) {
    for mut thing in bouncy_things.iter_mut() {
        thing.coefficient = settings.world.bounciness;
    }
}

fn update_gravity(mut gravity: ResMut<Gravity>, settings: Res<Settings>) {
    gravity.0 = get_gravity(settings.world.gravity);
}

fn collide(
    collision: &Contacts,
    pad: &mut Pad,
    ball: &mut Ball,
    ball_velocity: &LinearVelocity,
    midi: &mut EventWriter<MidiOutputEvent>,
    materials: &mut Assets<ColorMaterial>,
    settings: &Settings,
) {
    if collision.collision_started() {
        if pad.playing_notes.contains_key(&ball.size.to_octave()) {
            midi.send(MidiOutputEvent::NoteOff {
                note: pad.note,
                octave: ball.size.to_octave(),
            });
        }

        midi.send(MidiOutputEvent::NoteOn {
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
}

fn handle_collisions(
    mut collisions: EventReader<Collision>,
    mut midi: EventWriter<MidiOutputEvent>,
    mut pads: Query<&mut Pad>,
    mut balls: Query<(&mut Ball, &LinearVelocity)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    settings: Res<Settings>,
) {
    for Collision(collision) in collisions.read() {
        if let Ok(mut pad) = pads.get_mut(collision.entity1) {
            if let Ok((mut ball, velocity)) = balls.get_mut(collision.entity2) {
                collide(
                    collision,
                    &mut pad,
                    &mut ball,
                    velocity,
                    &mut midi,
                    &mut materials,
                    &settings,
                );
            }
        }
        if let Ok(mut pad) = pads.get_mut(collision.entity2) {
            if let Ok((mut ball, velocity)) = balls.get_mut(collision.entity1) {
                collide(
                    collision,
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
                midi.send(MidiOutputEvent::NoteOff {
                    note,
                    octave: octave.clone(),
                });
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

fn oldest_ball(balls: &Vec<(Entity, SystemTime)>) -> Option<Entity> {
    balls
        .iter()
        .max_by(|a, b| {
            let (_, a_spawn_time) = a;
            let (_, b_spawn_time) = b;

            b_spawn_time.cmp(&a_spawn_time)
        })
        .map(|(entity, _)| *entity)
}

fn despawn_oldest_balls(
    num_to_remove: usize,
    balls: &Query<(Entity, &Ball, &Transform)>,
    mut commands: Commands,
) {
    let mut remaining_balls: Vec<(Entity, SystemTime)> = vec![];
    for (entity, ball, _) in balls.iter() {
        remaining_balls.push((entity, ball.spawn_time));
    }

    for _ in 0..num_to_remove {
        if let Some(oldest) = oldest_ball(&remaining_balls) {
            commands.entity(oldest).despawn();
            remaining_balls.retain(|(entity, _)| entity != &oldest);
        }
    }
}

fn clean_up_balls(
    mut commands: Commands,
    mut balls: Query<(Entity, &Ball, &Transform)>,
    window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<Settings>,
) {
    let window = window.single();
    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;
    let rect = Rect::new(-half_width, -half_height, half_width, half_height);

    for (entity, ball, transform) in balls.iter_mut() {
        if !rect.contains(transform.translation.truncate())
            || (settings.world.max_bounces.enabled
                && ball.bounces >= settings.world.max_bounces.limit)
        {
            commands.entity(entity).despawn();
        }
    }

    if settings.world.max_balls.enabled && balls.iter().count() > settings.world.max_balls.limit {
        despawn_oldest_balls(
            balls.iter().count() - settings.world.max_balls.limit,
            &balls,
            commands,
        );
    }
}
