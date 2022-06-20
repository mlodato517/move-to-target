mod levels;
use levels::{generate_levels, poly_mesh};

use std::time::Duration;

use bevy::core::Stopwatch;
use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use rand::prelude::*;

const PLAYER_SPEEDS: [f32; 2] = [200.0, 240.0];
const ROT_SPEEDS: [f32; 2] = [1.0, 0.6];

const INITIAL_TARGET_SPEED: f32 = PLAYER_SPEEDS[0] / 10.0;
const MIN_TARGET_SPEED: f32 = PLAYER_SPEEDS[0] / 20.0;
const MAX_TARGET_SPEED: f32 = PLAYER_SPEEDS[0] / 5.0;
const TARGET_ROT_SPEED: f32 = ROT_SPEEDS[0] / 10.0;

const TIME_STEP: f32 = 1.0 / 60.0;

const SIZE: Vec3 = bevy::math::const_vec3!([80.0, 80.0, 0.0]);
const TOLERANCE: f32 = 5.0;
const ANGLE_TOLERANCE: f32 = 0.15;

// TODO use bevy::window::Window
const HALF_SCREEN_WIDTH: f32 = 200.0;
const HALF_SCREEN_HEIGHT: f32 = 200.0;

#[derive(Component)]
struct Player {
    idx: usize,
    points: Vec<[f32; 3]>,
}
#[derive(Component)]
struct Target {
    points: Vec<[f32; 3]>,
}
#[derive(Clone, Copy, Component)]
struct Velocity(Vec2);
#[derive(Clone, Copy, Component)]
struct Rotation(f32);
#[derive(Clone, Copy, Component)]
struct Score;
#[derive(Clone, Copy, Component)]
struct Wall;
#[derive(Clone, Component)]
struct ResetTimer(Timer);

// TODO How to handle level 1 vs level 2, etc.? Also need screens before start and after death.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Level,
    Win,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_camera)
        .init_resource::<Stopwatch>()
        .add_state(GameState::Level)
        .add_system_set(SystemSet::on_enter(GameState::Level).with_system(setup_level))
        .add_system_set(
            SystemSet::on_update(GameState::Level)
                .with_system(move_players)
                .with_system(move_target)
                .with_system(check_positions) // need ordering?
                .with_system(update_score), // need ordering?
        )
        .add_system_set(SystemSet::on_exit(GameState::Level).with_system(teardown_level))
        .add_system_set(SystemSet::on_enter(GameState::Win).with_system(display_score))
        .add_system_set(SystemSet::on_update(GameState::Win).with_system(delay_to_level_1))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn centered_text(asset_server: Res<AssetServer>) -> (TextStyle, TextAlignment) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_style = TextStyle {
        font,
        font_size: 60.0,
        color: Color::WHITE,
    };
    let text_alignment = TextAlignment {
        vertical: VerticalAlign::Center,
        horizontal: HorizontalAlign::Center,
    };

    (text_style, text_alignment)
}
fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Spawn text to display timer.
    let (text_style, text_alignment) = centered_text(asset_server);
    commands
        .spawn_bundle(Text2dBundle {
            text: Text::with_section("time", text_style, text_alignment),
            transform: Transform::default().with_translation([0.0, 300.0, 0.0].into()),
            ..default()
        })
        .insert(Score); // Should this be a resource?

    // Spawn Players.
    let mut level_data = generate_levels();
    let level1 = level_data.pop().unwrap();
    let player1_points = mesh_points_raw(&level1.player1).unwrap().clone();
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
    let player2_points = mesh_points_raw(&level1.player2).unwrap().clone();
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

    // Spawn target.
    let target_points = mesh_points_raw(&level1.target).unwrap().clone();
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(level1.target)),
            transform: Transform::default()
                .with_scale(SIZE)
                .with_translation([0.0, 100.0, 0.0].into()),
            material: materials.add(ColorMaterial::default()),
            ..default()
        })
        .insert(Target {
            points: target_points,
        })
        .insert(Rotation(TARGET_ROT_SPEED))
        .insert(Velocity(
            [INITIAL_TARGET_SPEED, INITIAL_TARGET_SPEED].into(),
        ));

    // Spawn walls.
    // TODO follow https://github.com/bevyengine/bevy/blob/latest/examples/games/breakout.rs.
    let walls = poly_mesh(vec![
        [-HALF_SCREEN_WIDTH, -HALF_SCREEN_HEIGHT, 0.0],
        [-HALF_SCREEN_WIDTH, HALF_SCREEN_HEIGHT, 0.0],
        [HALF_SCREEN_WIDTH, HALF_SCREEN_HEIGHT, 0.0],
        [HALF_SCREEN_WIDTH, -HALF_SCREEN_HEIGHT, 0.0],
        [-HALF_SCREEN_WIDTH, -HALF_SCREEN_HEIGHT, 0.0],
    ]);
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(walls)),
            material: materials.add(ColorMaterial::default()),
            ..default()
        })
        .insert(Wall);
}

/// Updates text with elapsed time in seconds.
fn update_score(
    time: Res<Time>,
    mut stopwatch: ResMut<Stopwatch>,
    mut query: Query<&mut Text, With<Score>>,
) {
    let mut clock_text = query.single_mut();
    stopwatch.tick(time.delta());
    clock_text.sections[0].value = format!("{:.1}", stopwatch.elapsed_secs());
}

/// Despawns players and targets
// https://github.com/bevyengine/bevy/blob/main/examples/games/alien_cake_addict.rs#L180
fn teardown_level(
    mut commands: Commands,
    entities: Query<Entity, Or<(With<Player>, With<Target>)>>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn display_score(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    stopwatch: Res<Stopwatch>,
) {
    let score = stopwatch.elapsed_secs() as u64;
    let label = match score {
        0..=20 => "AMAZING!",
        21..=40 => "GREAT JOB",
        41..=60 => "PRETTY GOOD",
        61..=80 => "YA DID OK",
        _ => "ARE YOU ASLEEP?",
    };

    let (text_style, text_alignment) = centered_text(asset_server);
    commands.spawn_bundle(Text2dBundle {
        text: Text::with_section(label, text_style, text_alignment),
        transform: Transform::default().with_translation([0.0, 250.0, 0.0].into()),
        ..default()
    });

    commands
        .spawn()
        .insert(ResetTimer(Timer::new(Duration::from_secs(5), false)));
}

// TODO show a button that allows them to go back to the main menu.
fn delay_to_level_1(
    mut commands: Commands,
    mut state: ResMut<State<GameState>>,
    mut clock_query: Query<&mut ResetTimer>,
    entities: Query<Entity, Without<Camera>>,
    mut stopwatch: ResMut<Stopwatch>,
    time: Res<Time>,
) {
    let timer = &mut clock_query.single_mut().0;
    timer.tick(time.delta());
    if timer.finished() {
        for entity in entities.iter() {
            commands.entity(entity).despawn_recursive();
        }
        stopwatch.reset();
        state
            .set(GameState::Level)
            .expect("How should one handle this?");
    }
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
fn move_players(keyboard_input: Res<Input<KeyCode>>, mut query: Query<(&mut Transform, &Player)>) {
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

            if points_collide_with_wall(&player.points, &transform) != 0 {
                transform.translation.x = old_x;
            }
        }
        let down_pressed = keyboard_input.pressed(down);
        let up_pressed = keyboard_input.pressed(up);
        if down_pressed != up_pressed {
            let y_direction = if down_pressed { -1.0 } else { 1.0 };
            let old_y = transform.translation.y;
            transform.translation.y += y_direction * PLAYER_SPEEDS[player.idx] * TIME_STEP;

            if points_collide_with_wall(&player.points, &transform) != 0 {
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

            if points_collide_with_wall(&player.points, &transform) != 0 {
                transform.rotation = old_rotation;
            }
        }
    }
}

const WALL_COLLISION_NONE: u8 = 0;
const WALL_COLLISION_HORIZONTAL: u8 = 1;
const WALL_COLLISION_VERTICAL: u8 = 2;
fn move_target(mut query: Query<(&mut Transform, &Target, &mut Velocity, &mut Rotation)>) {
    let (mut transform, target, mut velocity, mut rotation) = query.single_mut();

    // Check if moving would get us out of bounds.
    let old_x = transform.translation.x;
    let old_y = transform.translation.y;
    transform.translation.x += velocity.0.x * TIME_STEP;
    transform.translation.y += velocity.0.y * TIME_STEP;
    let collision = points_collide_with_wall(&target.points, &transform);

    if collision & WALL_COLLISION_HORIZONTAL > 0 {
        transform.translation.x = old_x;
        velocity.0.x *= -1.0;
    }
    if collision & WALL_COLLISION_VERTICAL > 0 {
        transform.translation.y = old_y;
        velocity.0.y *= -1.0;
    }

    // Check if rotating would get us out of bounds.
    let old_rotation = transform.rotation;
    let rotate_by = Quat::from_rotation_z(rotation.0 * TIME_STEP);
    transform.rotation = transform.rotation.mul_quat(rotate_by);
    let collision = points_collide_with_wall(&target.points, &transform);

    if collision > 0 {
        transform.rotation = old_rotation;
        rotation.0 *= -1.0;
    }

    // TODO For deterministic mode, seed the RNG too.
    *velocity = accelerate_target(*velocity);
}

fn mesh_points_raw(mesh: &Mesh) -> Option<&Vec<[f32; 3]>> {
    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        .map(|position| match position {
            // TODO first and last points are probably always the same - can probably skip first.
            VertexAttributeValues::Float32x3(points) => points,
            _ => unreachable!(),
        })
}
fn points_collide_with_wall(points: &[[f32; 3]], transform: &Transform) -> u8 {
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

fn accelerate_target(mut velocity: Velocity) -> Velocity {
    // TODO generate fewer random numbers. Also, don't use a cryptographically secure RNG.
    if thread_rng().gen_bool(0.1) {
        velocity.0.x *= thread_rng().gen_range(0.5..2.0);
        velocity.0.x = handle_target_velocity_overflow(velocity.0.x);
        velocity.0.y *= thread_rng().gen_range(0.5..2.0);
        velocity.0.y = handle_target_velocity_overflow(velocity.0.y);
    }

    velocity
}

/// If the target is going too slowly or too quickly just set it back to the initial speed.
fn handle_target_velocity_overflow(speed: f32) -> f32 {
    if !(MIN_TARGET_SPEED..MAX_TARGET_SPEED).contains(&speed.abs()) {
        INITIAL_TARGET_SPEED * speed.signum()
    } else {
        speed
    }
}

fn check_positions(
    players_query: Query<&Transform, With<Player>>,
    target_query: Query<&Transform, With<Target>>,
    mut state: ResMut<State<GameState>>,
) {
    let target = target_query.single();

    // TODO understand why we can just check translation directly. Why do they all have the same
    // origin point?
    let all_players_there = players_query.iter().all(|player| {
        let delta_x = (player.translation.x - target.translation.x).abs();
        let delta_y = (player.translation.y - target.translation.y).abs();
        let delta_r = player.rotation.angle_between(target.rotation);
        delta_x < TOLERANCE && delta_y < TOLERANCE && delta_r < ANGLE_TOLERANCE
    });

    if all_players_there {
        state
            .set(GameState::Win)
            .expect("How should one handle this?");
    }
}
