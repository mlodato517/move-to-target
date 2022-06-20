use super::{utils, SIZE, TIME_STEP};

use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

const PLAYER_SPEEDS: [f32; 2] = [200.0, 240.0];
const ROT_SPEEDS: [f32; 2] = [1.0, 0.6];

#[derive(Component)]
pub struct Player {
    idx: usize,
    points: Vec<[f32; 3]>,
}

pub fn spawn_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // HACK
    let level1 = utils::generate_levels().pop().unwrap();
    let player1_points = utils::mesh_points_raw(&level1.player1).unwrap().clone();
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(level1.player1)),
            transform: Transform::default()
                .with_scale(SIZE)
                .with_translation([-50.0, -100.0, 0.0].into()),
            material: materials.add(ColorMaterial::default()),
            ..default()
        })
        .insert(Player {
            idx: 0,
            points: player1_points,
        });
    let player2_points = utils::mesh_points_raw(&level1.player2).unwrap().clone();
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(level1.player2)),
            transform: Transform::default()
                .with_scale(SIZE)
                .with_translation([50.0, -100.0, 0.0].into()),
            material: materials.add(ColorMaterial::default()),
            ..default()
        })
        .insert(Player {
            idx: 1,
            points: player2_points,
        });
}

// TODO show keys in a legend on the sides of the game.
struct Keys {
    up: KeyCode,
    left: KeyCode,
    down: KeyCode,
    right: KeyCode,
    rot_cw: KeyCode,
    rot_ccw: KeyCode,
}
const PLAYER_KEYS: [Keys; 2] = [
    Keys {
        up: KeyCode::W,
        left: KeyCode::A,
        down: KeyCode::S,
        right: KeyCode::D,
        rot_ccw: KeyCode::Q,
        rot_cw: KeyCode::E,
    },
    Keys {
        up: KeyCode::I,
        left: KeyCode::J,
        down: KeyCode::K,
        right: KeyCode::L,
        rot_ccw: KeyCode::U,
        rot_cw: KeyCode::O,
    },
];
pub fn move_players(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &Player)>,
) {
    for (mut transform, player) in query.iter_mut() {
        let Keys {
            up,
            left,
            down,
            right,
            rot_cw,
            rot_ccw,
        } = PLAYER_KEYS[player.idx];

        // TODO Test with conversion to i8 and doing ((l ^ r) + (l - r)) as f32
        let left_pressed = keyboard_input.pressed(left);
        let right_pressed = keyboard_input.pressed(right);
        if left_pressed != right_pressed {
            let x_direction = if left_pressed { -1.0 } else { 1.0 };
            let old_x = transform.translation.x;
            transform.translation.x += x_direction * PLAYER_SPEEDS[player.idx] * TIME_STEP;

            if utils::points_collide_with_wall(&player.points, &transform) != 0 {
                transform.translation.x = old_x;
            }
        }
        let down_pressed = keyboard_input.pressed(down);
        let up_pressed = keyboard_input.pressed(up);
        if down_pressed != up_pressed {
            let y_direction = if down_pressed { -1.0 } else { 1.0 };
            let old_y = transform.translation.y;
            transform.translation.y += y_direction * PLAYER_SPEEDS[player.idx] * TIME_STEP;

            if utils::points_collide_with_wall(&player.points, &transform) != 0 {
                transform.translation.y = old_y;
            }
        }
        let rot_ccw_pressed = keyboard_input.pressed(rot_ccw);
        let rot_cw_pressed = keyboard_input.pressed(rot_cw);
        if rot_ccw_pressed != rot_cw_pressed {
            let rotation_direction = if rot_ccw_pressed { -1.0 } else { 1.0 };
            let old_rotation = transform.rotation;
            let rotate_by =
                Quat::from_rotation_z(rotation_direction * ROT_SPEEDS[player.idx] * TIME_STEP);
            transform.rotation = transform.rotation.mul_quat(rotate_by);

            if utils::points_collide_with_wall(&player.points, &transform) != 0 {
                transform.rotation = old_rotation;
            }
        }
    }
}
