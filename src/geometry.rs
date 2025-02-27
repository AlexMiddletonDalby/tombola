use avian2d::math::PI;
use bevy::math::{Quat, Vec2};
use bevy::prelude::Transform;

pub fn deg_to_rad(degrees: f32) -> f32 {
    degrees * PI / 180.0
}

pub fn hexagon(centre: Vec2, side_length: f32) -> Vec<Transform> {
    let root_three = 3.0f32.sqrt();
    let x = side_length / 2.0;
    let y = (root_three * side_length) / 2.0;

    let vertices = vec![
        Vec2::new(-x, y),
        Vec2::new(x, y),
        Vec2::new(side_length, 0.0),
        Vec2::new(x, -y),
        Vec2::new(-x, -y),
        Vec2::new(-side_length, 0.0),
    ];

    let mut transforms = Vec::new();

    for x in 0..vertices.len() {
        let angle = deg_to_rad(-60.0 * x as f32);

        let mut y = x + 1;
        if y >= vertices.len() {
            y = 0;
        }

        let mid = vertices[x].midpoint(vertices[y]);
        transforms.push(
            Transform::from_xyz(centre.x + mid.x, centre.y + mid.y, 0.0)
                .with_rotation(Quat::from_rotation_z(angle)),
        );
    }

    transforms
}
