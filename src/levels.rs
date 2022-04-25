use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;

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
