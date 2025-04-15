use avian2d::math::PI;
use bevy::math::ops::{cos, sin};
use bevy::math::{Quat, Vec2};
use bevy::prelude::Transform;
use strum_macros::EnumIter;

#[derive(Clone, Copy, PartialEq, EnumIter)]
pub enum Shape {
    Square,
    Pentagon,
    Hexagon,
    Heptagon,
    Octagon,
}

impl Shape {
    pub fn to_string(&self) -> String {
        match self {
            Shape::Square => "Square".to_string(),
            Shape::Pentagon => "Pentagon".to_string(),
            Shape::Hexagon => "Hexagon".to_string(),
            Shape::Heptagon => "Heptagon".to_string(),
            Shape::Octagon => "Octagon".to_string(),
        }
    }

    pub fn get_num_sides(&self) -> usize {
        match self {
            Shape::Square => 4,
            Shape::Pentagon => 5,
            Shape::Hexagon => 6,
            Shape::Heptagon => 7,
            Shape::Octagon => 8,
        }
    }

    pub fn get_side_transforms(&self, centre: Vec2, apothem: f32) -> Vec<Transform> {
        polygon(centre, apothem, self.get_num_sides())
    }
}

fn to_transforms(vertices: Vec<Vec2>, centre: Vec2, turn_angle: f32) -> Vec<Transform> {
    let mut transforms = Vec::new();

    let mut angle = 0.0;

    for x in 0..vertices.len() {
        transforms.push(
            Transform::from_xyz(centre.x + vertices[x].x, centre.y + vertices[x].y, 0.0)
                .with_rotation(Quat::from_rotation_z(angle)),
        );

        angle -= turn_angle;
    }

    transforms
}

pub fn polygon(centre: Vec2, apothem: f32, num_sides: usize) -> Vec<Transform> {
    let mut mid_points = Vec::new();
    if num_sides < 3 {
        return Vec::new();
    }

    let angle_increment = 2.0 * PI / num_sides as f32;

    for i in 0..num_sides {
        let angle = i as f32 * angle_increment;
        mid_points.push(Vec2::new(apothem * sin(angle), apothem * cos(angle)));
    }

    to_transforms(mid_points, centre, angle_increment)
}
