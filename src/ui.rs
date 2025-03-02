use crate::size::Size;
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::math::Vec2;
use bevy::prelude::{
    Bundle, Circle, ColorMaterial, Component, Mesh, Mesh2d, MeshMaterial2d, ResMut, Transform,
};

#[derive(Component)]
pub struct BallSelector {
    pub size: Size,
}

impl BallSelector {
    pub fn hitbox_size() -> f32 {
        Size::Large.to_radius() + 10.0
    }
}

#[derive(Bundle)]
pub struct BallSelectorBundle {
    marker: BallSelector,
    transform: Transform,
    mesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,
}

impl BallSelectorBundle {
    pub fn new(
        size: Size,
        position: Vec2,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        BallSelectorBundle {
            marker: BallSelector { size },
            transform: Transform::from_xyz(position.x, position.y, 1.0),
            mesh: Mesh2d(meshes.add(Circle::new(size.to_radius()))),
            material: MeshMaterial2d(materials.add(ColorMaterial::from_color(size.to_color()))),
        }
    }
}

#[derive(Component)]
pub struct Highlight;

#[derive(Bundle)]
pub struct HighlightBundle {
    marker: Highlight,
    transform: Transform,
    mesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,
}

impl HighlightBundle {
    pub fn new(
        position: Vec2,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        HighlightBundle {
            marker: Highlight,
            transform: Transform::from_xyz(position.x, position.y, 0.0),
            mesh: Mesh2d(meshes.add(Circle::new(BallSelector::hitbox_size()))),
            material: MeshMaterial2d(
                materials.add(ColorMaterial::from_color(Color::srgba(0.3, 0.3, 0.4, 0.2))),
            ),
        }
    }
}
