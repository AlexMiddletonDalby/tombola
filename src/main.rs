mod geometry;
mod midi;

use avian2d::prelude::Restitution;
use avian2d::prelude::*;
use std::cmp::PartialEq;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

//--------------------------------------------------------------------------------------------------
//Enums

#[derive(Copy, Clone, PartialEq)]
enum Size {
    Small,
    Medium,
    Large,
}
impl Size {
    const fn to_octave(&self) -> i32 {
        match self {
            Size::Small => 4,
            Size::Medium => 3,
            Size::Large => 2,
        }
    }

    const fn to_radius(&self) -> f32 {
        match self {
            Size::Small => 10.0,
            Size::Medium => 15.0,
            Size::Large => 25.0,
        }
    }

    const fn to_color(&self) -> Color {
        match self {
            Size::Small => Color::linear_rgb(0.8, 0.3, 0.3),
            Size::Medium => Color::linear_rgb(0.8, 0.7, 0.3),
            Size::Large => Color::linear_rgb(0.1, 0.3, 0.8),
        }
    }
}

const SELECTOR_RADIUS: f32 = Size::Large.to_radius() + 10.0;
const SELECTOR_SPACING: f32 = 50.0;

//--------------------------------------------------------------------------------------------------
//Resources

#[derive(Resource)]
struct Midi {
    output_handle: Option<MidiOutputConnection>,
}

#[derive(Resource, Default)]
struct WorldMouse {
    position: Vec2,
}

#[derive(Resource)]
struct SelectedBall {
    size: Size,
}

//--------------------------------------------------------------------------------------------------
//App

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
                setup_camera,
                spawn_ball_selectors.after(setup_camera),
                spawn_tombola.after(setup_camera),
            ),
        )
        .add_systems(
            Update,
            (
                update_world_mouse,
                handle_click,
                handle_pad_collisions,
                fade_pads,
                update_selector_positions,
                update_highlight.after(update_selector_positions),
            ),
        )
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
//Components

#[derive(Component)]
struct Ball {
    size: Size,
}

#[derive(Component)]
struct BallSelector {
    size: Size,
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Pad {
    note: midi::Note,
    material_handle: MeshMaterial2d<ColorMaterial>,
}

impl Pad {
    fn default_color() -> Color {
        Color::linear_rgb(0.1, 0.1, 0.1)
    }

    fn hit_color() -> Color {
        Color::linear_rgb(1.0, 1.0, 1.0)
    }
}

#[derive(Component)]
struct Highlight;

//--------------------------------------------------------------------------------------------------
//Startup

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

fn pad(
    rect: Vec2,
    transform: Transform,
    note: midi::Note,
    commands: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let restitution = Restitution::new(1.0);
    let material = MeshMaterial2d(materials.add(Pad::default_color()));

    commands.spawn((
        Pad {
            note,
            material_handle: material.clone(),
        },
        restitution,
        Collider::rectangle(rect.x, rect.y),
        Mesh2d(meshes.add(Rectangle::new(rect.x, rect.y))),
        material.clone(),
        transform,
    ));
}

fn pad_hexagon(
    centre: Vec2,
    side_length: f32,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let thickness = 5.0;
    let rect = Vec2::new(side_length, thickness);

    commands
        .spawn((
            RigidBody::Kinematic,
            AngularVelocity(1.5),
            Transform::from_xyz(centre.x, centre.y, 0.0),
            Visibility::default(),
        ))
        .with_children(|commands| {
            let transforms = geometry::hexagon(centre, side_length);
            let notes = vec![
                midi::Note::C,
                midi::Note::E,
                midi::Note::G,
                midi::Note::ASharp,
                midi::Note::D,
                midi::Note::F,
            ];

            for (index, transform) in transforms.into_iter().enumerate() {
                pad(rect, transform, notes[index], commands, meshes, materials);
            }
        });
}

fn spawn_tombola(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    pad_hexagon(
        Vec2::new(10.0, 0.0),
        300.0,
        &mut commands,
        &mut meshes,
        &mut materials,
    );
}

fn spawn_ball_selector(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    position: Vec2,
    size: Size,
) {
    commands.spawn((
        BallSelector { size },
        Transform::from_xyz(position.x, position.y, 1.0),
        Mesh2d(meshes.add(Circle::new(size.to_radius()))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(size.to_color()))),
    ));
}

fn spawn_ball_selectors(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window.single();
    let x_pos = window.width() / 2.0 - SELECTOR_SPACING;

    commands.spawn((
        Highlight,
        Transform::from_xyz(x_pos, 100.0, 0.0),
        Mesh2d(meshes.add(Circle::new(SELECTOR_RADIUS))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgba(0.6, 0.6, 0.7, 0.2)))),
    ));

    spawn_ball_selector(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(x_pos, 100.0),
        Size::Small,
    );

    spawn_ball_selector(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(x_pos, 0.0),
        Size::Medium,
    );

    spawn_ball_selector(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(x_pos, -100.0),
        Size::Large,
    );
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
    let window = window.single();
    let x_pos = window.width() / 2.0 - SELECTOR_SPACING;

    for mut selector in selectors.iter_mut() {
        selector.translation.x = x_pos;
    }
}

fn get_selector_pos(selectors: &Vec<(&BallSelector, &Transform)>, selected: Size) -> Option<Vec2> {
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
        if let Some(pos) = get_selector_pos(&selectors.iter().collect(), selected_ball.size) {
            highlight.translation.x = pos.x;
            highlight.translation.y = pos.y;
        }
    }
}

fn pick_selector(selectors: Query<(&BallSelector, &Transform)>, pos: Vec2) -> Option<Size> {
    for (selector, transform) in selectors.iter() {
        let centre = transform.translation.truncate();

        let rect = Rect::new(
            centre.x + SELECTOR_RADIUS,
            centre.y + SELECTOR_RADIUS,
            centre.x - SELECTOR_RADIUS,
            centre.y - SELECTOR_RADIUS,
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
            commands.spawn((
                Ball {
                    size: selected_ball.size,
                },
                Transform::from_xyz(world_mouse.position.x, world_mouse.position.y, 0.0),
                RigidBody::Dynamic,
                Restitution::new(1.0),
                Collider::circle(selected_ball.size.to_radius()),
                Mesh2d(meshes.add(Circle::new(selected_ball.size.to_radius()))),
                MeshMaterial2d(
                    materials.add(ColorMaterial::from_color(selected_ball.size.to_color())),
                ),
            ));
        }
    } else if buttons.just_pressed(MouseButton::Right) {
        for ball in balls.iter() {
            commands.entity(ball).despawn();
        }
    }
}

fn handle_pad_collision(
    collision: &Contacts,
    pad: &Pad,
    ball: &Ball,
    ball_velocity: &LinearVelocity,
    mut midi_output: &mut Option<MidiOutputConnection>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    if collision.collision_started() {
        midi::note_on(
            pad.note,
            ball.size.to_octave(),
            midi::to_velocity(ball_velocity.length()),
            &mut midi_output,
        );

        if let Some(material) = materials.get_mut(pad.material_handle.0.id()) {
            material.color = Pad::hit_color();
        }
    }
    if collision.collision_stopped() {
        midi::note_off(pad.note, 4, &mut midi_output);
    }
}

fn handle_pad_collisions(
    mut collisions: EventReader<Collision>,
    pads: Query<&Pad>,
    balls: Query<(&Ball, &LinearVelocity)>,
    mut midi: ResMut<Midi>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for Collision(collision) in collisions.read() {
        if let Ok(pad) = pads.get(collision.entity1) {
            if let Ok((ball, velocity)) = balls.get(collision.entity2) {
                handle_pad_collision(
                    collision,
                    pad,
                    ball,
                    velocity,
                    &mut midi.output_handle,
                    &mut materials,
                );
            }
        }
        if let Ok(pad) = pads.get(collision.entity2) {
            if let Ok((ball, velocity)) = balls.get(collision.entity1) {
                handle_pad_collision(
                    collision,
                    pad,
                    ball,
                    velocity,
                    &mut midi.output_handle,
                    &mut materials,
                );
            }
        }
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
        if let Some(material) = materials.get_mut(pad.material_handle.0.id()) {
            material.color = material.color.mix(&Pad::default_color(), amount);
        }
    }
}
