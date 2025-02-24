mod midi;

use avian2d::math::PI;
use avian2d::prelude::Restitution;
use avian2d::prelude::*;

use bevy::prelude::*;

use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

#[derive(Resource)]
struct Midi {
    output_handle: Option<MidiOutputConnection>,
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
        .add_systems(Startup, (spawn_ball, setup_pads, setup_camera))
        .add_systems(Update, handle_pad_collisions)
        .insert_resource(Gravity(Vec2::NEG_Y * 700.0))
        .insert_resource(Midi {
            output_handle: handle,
        })
        .run();
}

//Startup
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Component)]
struct Ball {
    octave: i32,
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Ball { octave: 2 },
        RigidBody::Dynamic,
        Restitution::new(1.0),
        Collider::circle(25.0),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Mesh2d(meshes.add(Circle::new(25.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::linear_rgb(0.1, 0.3, 0.8)))),
    ));

    commands.spawn((
        Ball { octave: 3 },
        RigidBody::Dynamic,
        Restitution::new(1.0),
        Collider::circle(15.0),
        Transform::from_xyz(-100.0, 0.0, 0.0),
        Mesh2d(meshes.add(Circle::new(15.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::linear_rgb(0.8, 0.7, 0.3)))),
    ));

    commands.spawn((
        Ball { octave: 4 },
        RigidBody::Dynamic,
        Restitution::new(1.0),
        Collider::circle(10.0),
        Transform::from_xyz(100.0, 0.0, 0.0),
        Mesh2d(meshes.add(Circle::new(10.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::linear_rgb(0.8, 0.3, 0.3)))),
    ));
}

#[derive(Component)]
struct Pad {
    note: midi::Note,
}

fn spawn_pad_box(
    centre: Vec2,
    size: f32,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let restitution = Restitution::new(1.0);
    let material = MeshMaterial2d(materials.add(Color::linear_rgb(0.2, 0.2, 0.2)));
    let pad_thickness = 5.0;

    commands
        .spawn((
            RigidBody::Kinematic,
            AngularVelocity(1.5),
            Transform::from_xyz(centre.x, centre.y, 0.0).with_rotation(Quat::from_rotation_z(0.2)),
            Visibility::default(),
        ))
        .with_children(|commands| {
            commands.spawn((
                Pad {
                    note: midi::Note::C,
                },
                restitution,
                Collider::rectangle(size, pad_thickness),
                Mesh2d(meshes.add(Rectangle::new(size, pad_thickness))),
                material.clone(),
                Transform::from_xyz(
                    centre.x,
                    centre.y + (size / 2.0) - (pad_thickness / 2.0),
                    0.0,
                ),
            ));

            commands.spawn((
                Pad {
                    note: midi::Note::E,
                },
                restitution,
                Collider::rectangle(size, 5.0),
                Mesh2d(meshes.add(Rectangle::new(size, 5.0))),
                material.clone(),
                Transform::from_xyz(
                    centre.x,
                    centre.y - (size / 2.0) + (pad_thickness / 2.0),
                    0.0,
                )
                .with_rotation(Quat::from_rotation_z(-PI)),
            ));

            commands.spawn((
                Pad {
                    note: midi::Note::G,
                },
                restitution,
                Collider::rectangle(size, 5.0),
                Mesh2d(meshes.add(Rectangle::new(size, 5.0))),
                material.clone(),
                Transform::from_xyz(
                    centre.x + (size / 2.0) - (pad_thickness / 2.0),
                    centre.y,
                    0.0,
                )
                .with_rotation(Quat::from_rotation_z(-PI / 2.0)),
            ));

            commands.spawn((
                Pad {
                    note: midi::Note::A,
                },
                restitution,
                Collider::rectangle(size, 5.0),
                Mesh2d(meshes.add(Rectangle::new(size, 5.0))),
                material.clone(),
                Transform::from_xyz(centre.x - (size / 2.0) + (pad_thickness / 2.0), 0.0, 0.0)
                    .with_rotation(Quat::from_rotation_z(PI / 2.0)),
            ));
        });
}

fn setup_pads(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    spawn_pad_box(
        Vec2::new(0.0, 0.0),
        500.0,
        &mut commands,
        &mut meshes,
        &mut materials,
    );
}

//Update
fn handle_pad_collision(
    collision: &Contacts,
    pad: &Pad,
    ball: &Ball,
    ball_velocity: &LinearVelocity,
    mut midi_output: &mut Option<MidiOutputConnection>,
) {
    if collision.collision_started() {
        midi::note_on(
            pad.note,
            ball.octave,
            midi::to_velocity(ball_velocity.length()),
            &mut midi_output,
        );
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
) {
    for Collision(collision) in collisions.read() {
        if let Ok(pad) = pads.get(collision.entity1) {
            if let Ok((ball, velocity)) = balls.get(collision.entity2) {
                handle_pad_collision(collision, pad, ball, velocity, &mut midi.output_handle);
            }
        }
        if let Ok(pad) = pads.get(collision.entity2) {
            if let Ok((ball, velocity)) = balls.get(collision.entity1) {
                handle_pad_collision(collision, pad, ball, velocity, &mut midi.output_handle);
            }
        }
    }
}
