use crate::midi;
use avian2d::prelude::*;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub struct Pad {
    pub index: usize,
    pub note: midi::Note,
    pub playing_notes: HashMap<i32, Timer>,
    pub material: MeshMaterial2d<ColorMaterial>,
}

impl Pad {
    pub fn default_color() -> Color {
        Color::linear_rgb(0.3, 0.3, 0.3)
    }

    pub fn hit_color() -> Color {
        Color::linear_rgb(5.0, 5.0, 30.0)
    }
}

#[derive(Bundle)]
pub struct PadBundle {
    marker: Pad,
    transform: Transform,
    restitution: Restitution,
    collider: Collider,
    mesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,
}

impl PadBundle {
    pub fn new(
        index: usize,
        size: Vec2,
        transform: Transform,
        note: midi::Note,
        bounciness: f32,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        let material = MeshMaterial2d(materials.add(Pad::default_color()));

        PadBundle {
            marker: Pad {
                index,
                note,
                material: material.clone(),
                playing_notes: HashMap::new(),
            },
            transform,
            restitution: Restitution::new(bounciness),
            collider: Collider::rectangle(size.x, size.y),
            mesh: Mesh2d(meshes.add(Rectangle::new(size.x, size.y))),
            material: material.clone(),
        }
    }
}
