use avian2d::math::PI;
use avian2d::prelude::Restitution;
use avian2d::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_systems(Startup, (spawn_ball, setup_pads, setup_camera))
        .insert_resource(Gravity(Vec2::NEG_Y * 700.0))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Component)]
struct Ball;

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Ball,
        RigidBody::Dynamic,
        Restitution::new(1.0),
        Collider::circle(15.0),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Mesh2d(meshes.add(Circle::new(15.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::linear_rgb(0.8, 0.8, 0.8)))),
    ));
}

#[derive(Component)]
struct Pad;

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
            Transform::from_xyz(centre.x, centre.y, 0.0),
        ))
        .with_children(|commands| {
            commands.spawn((
                Pad,
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
                Pad,
                restitution,
                Collider::rectangle(size, 5.0),
                Mesh2d(meshes.add(Rectangle::new(size, 5.0))),
                material.clone(),
                Transform::from_xyz(
                    centre.x,
                    centre.y - (size / 2.0) + (pad_thickness / 2.0),
                    0.0,
                ),
            ));

            commands.spawn((
                Pad,
                restitution,
                Collider::rectangle(size, 5.0),
                Mesh2d(meshes.add(Rectangle::new(size, 5.0))),
                material.clone(),
                Transform::from_xyz(
                    centre.x + (size / 2.0) - (pad_thickness / 2.0),
                    centre.y,
                    0.0,
                )
                .with_rotation(Quat::from_rotation_z(PI / 2.0)),
            ));

            commands.spawn((
                Pad,
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
