use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::mesh::VertexAttributeValues;

use super::{HALF_SCREEN_HEIGHT, HALF_SCREEN_WIDTH};

pub const WALL_COLLISION_NONE: u8 = 0;
pub const WALL_COLLISION_HORIZONTAL: u8 = 1;
pub const WALL_COLLISION_VERTICAL: u8 = 2;

pub fn mesh_points_raw(mesh: &Mesh) -> Option<&Vec<[f32; 3]>> {
    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        .map(|position| match position {
            // TODO first and last points are probably always the same - can probably skip first.
            VertexAttributeValues::Float32x3(points) => points,
            _ => unreachable!(),
        })
}
pub fn points_collide_with_wall(points: &[[f32; 3]], transform: &Transform) -> u8 {
    let mut collision = WALL_COLLISION_NONE;
    for point in points {
        let point = transform.mul_vec3(Vec3::from(*point));
        if point.x <= -HALF_SCREEN_WIDTH || point.x >= HALF_SCREEN_WIDTH {
            collision |= WALL_COLLISION_HORIZONTAL;
        } else if point.y <= -HALF_SCREEN_HEIGHT || point.y >= HALF_SCREEN_HEIGHT {
            collision |= WALL_COLLISION_VERTICAL;
        }
    }
    collision
}

// TODO Move this to files or something?
pub struct LevelData {
    /// Target shape to tile onto.
    pub target: Mesh,

    /// Player 1 shape.
    pub player1: Mesh,

    /// Player 2 shape.
    pub player2: Mesh,
}

/// Generate target, player1, and player2 meshes for each level.
/// The second two meshes should tile to the first.
/// Can this be done programmatically? Can I only generate the two halves?
pub fn generate_levels() -> Vec<LevelData> {
    vec![LevelData {
        target: poly_mesh(vec![
            [0.0, 1.0, 0.0],
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ]),
        player1: poly_mesh(vec![
            [0.0, 1.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.3, 0.0],
            [-0.3, 0.3, 0.0],
            [-0.3, 0.6, 0.0],
            [0.0, 0.6, 0.0],
            [0.0, 1.0, 0.0],
        ]),
        player2: poly_mesh(vec![
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.3, 0.0],
            [-0.3, 0.3, 0.0],
            [-0.3, 0.6, 0.0],
            [0.0, 0.6, 0.0],
            [0.0, 1.0, 0.0],
        ]),
    }]
}

pub fn poly_mesh(positions: Vec<[f32; 3]>) -> Mesh {
    let normals = vec![[0.0, 0.0, 1.0]; positions.len()];
    let uvs = vec![[0.0, 1.0]; positions.len()];

    let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}
