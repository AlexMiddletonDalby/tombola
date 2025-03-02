use crate::size::Size;
use avian2d::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Ball {
    pub size: Size,
}

#[derive(Bundle)]
pub struct BallBundle {
    marker: Ball,
    transform: Transform,
    body: RigidBody,
    restitution: Restitution,
    collider: Collider,
    mesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,
}

impl BallBundle {
    pub fn new(
        position: Vec2,
        size: Size,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        BallBundle {
            marker: Ball { size },
            transform: Transform::from_xyz(position.x, position.y, 0.0),
            body: RigidBody::Dynamic,
            restitution: Restitution::new(1.0),
            collider: Collider::circle(size.to_radius()),
            mesh: Mesh2d(meshes.add(Circle::new(size.to_radius()))),
            material: MeshMaterial2d(materials.add(ColorMaterial::from_color(size.to_color()))),
        }
    }
}
