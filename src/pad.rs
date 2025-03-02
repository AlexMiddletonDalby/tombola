use crate::midi;
use avian2d::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Pad {
    pub note: midi::Note,
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
        size: Vec2,
        transform: Transform,
        note: midi::Note,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        let material = MeshMaterial2d(materials.add(Pad::default_color()));

        PadBundle {
            marker: Pad {
                note,
                material: material.clone(),
            },
            transform,
            restitution: Restitution::new(1.0),
            collider: Collider::rectangle(size.x, size.y),
            mesh: Mesh2d(meshes.add(Rectangle::new(size.x, size.y))),
            material: material.clone(),
        }
    }
}
