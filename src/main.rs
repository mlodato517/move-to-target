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
struct Player(usize);
#[derive(Component)]
struct Target;
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

// TODO have this not have hacky enums for each level.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Level1,
    Win,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_camera)
        .init_resource::<Stopwatch>()
        .add_state(GameState::Level1)
        .add_system_set(SystemSet::on_enter(GameState::Level1).with_system(setup_level))
        .add_system_set(
            SystemSet::on_update(GameState::Level1)
                .with_system(move_players)
                .with_system(move_target)
                .with_system(check_positions) // need ordering?
                .with_system(update_score), // need ordering?
        )
        .add_system_set(SystemSet::on_exit(GameState::Level1).with_system(teardown_level))
        .add_system_set(SystemSet::on_enter(GameState::Win).with_system(display_score))
        .add_system_set(SystemSet::on_update(GameState::Win).with_system(delay_to_level_1))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn centered_text(asset_server: Res<AssetServer>) -> (TextStyle, TextAlignment) {
    // TODO store a font with the code.
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
    let level1 = level_data.swap_remove(0);
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(level1.player1)),
            transform: Transform::default()
                .with_scale(SIZE)
                .with_translation([-50.0, -100.0, 0.0].into()),
            material: materials.add(ColorMaterial::default()),
            ..default()
        })
        .insert(Player(0));
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(level1.player2)),
            transform: Transform::default()
                .with_scale(SIZE)
                .with_translation([50.0, -100.0, 0.0].into()),
            material: materials.add(ColorMaterial::default()),
            ..default()
        })
        .insert(Player(1));

    // Spawn target.
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(level1.target)),
            transform: Transform::default()
                .with_scale(SIZE)
                .with_translation([0.0, 100.0, 0.0].into()),
            material: materials.add(ColorMaterial::default()),
            ..default()
        })
        .insert(Target)
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
            .set(GameState::Level1)
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
fn move_players(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &Player, &Mesh2dHandle)>,
    meshes: Res<Assets<Mesh>>,
) {
    for (mut transform, player, mesh_handle) in query.iter_mut() {
        let Keys {
            up,
            left,
            down,
            right,
            rot_cw,
            rot_ccw,
        } = PLAYER_KEYS[player.0];

        let x_direction = if keyboard_input.pressed(left) {
            -1.0
        } else if keyboard_input.pressed(right) {
            1.0
        } else {
            0.0
        };

        let y_direction = if keyboard_input.pressed(down) {
            -1.0
        } else if keyboard_input.pressed(up) {
            1.0
        } else {
            0.0
        };

        let rotation_direction =
            if keyboard_input.pressed(rot_cw) && !keyboard_input.pressed(rot_ccw) {
                -1.0
            } else if keyboard_input.pressed(rot_ccw) && !keyboard_input.pressed(rot_cw) {
                1.0
            } else {
                0.0
            };

        // TODO don't get mesh points this way every time - just store indices on players/target.
        let mesh = meshes.get(mesh_handle.0.id).unwrap();

        if x_direction != 0.0 {
            // Check if moving would get us out of bounds
            let mut new_transform = *transform;
            new_transform.translation.x =
                transform.translation.x + x_direction * PLAYER_SPEEDS[player.0] * TIME_STEP;
            let collision = mesh_collides_with_wall(mesh, &new_transform);

            // Only update if not
            if collision == 0 {
                transform.translation.x = new_transform.translation.x;
            }
        }
        if y_direction != 0.0 {
            // Check if moving would get us out of bounds
            let mut new_transform = *transform;
            new_transform.translation.y =
                transform.translation.y + y_direction * PLAYER_SPEEDS[player.0] * TIME_STEP;
            let collision = mesh_collides_with_wall(mesh, &new_transform);

            // Only update if not
            if collision == 0 {
                transform.translation.y = new_transform.translation.y;
            }
        }
        if rotation_direction != 0.0 {
            // Check if rotating would get us out of bounds
            let mut new_transform = *transform;
            let rotate_by =
                Quat::from_rotation_z(rotation_direction * ROT_SPEEDS[player.0] * TIME_STEP);
            new_transform.rotation = transform.rotation.mul_quat(rotate_by);
            let collision = mesh_collides_with_wall(mesh, &new_transform);

            // Only update if not
            if collision == 0 {
                transform.rotation = new_transform.rotation;
            }
        }
    }
}

const WALL_COLLISION_NONE: u8 = 0;
const WALL_COLLISION_HORIZONTAL: u8 = 1;
const WALL_COLLISION_VERTICAL: u8 = 2;
fn move_target(
    mut query: Query<(&mut Transform, &Mesh2dHandle, &mut Velocity, &mut Rotation), With<Target>>,
    meshes: Res<Assets<Mesh>>,
) {
    let (mut transform, mesh_handle, mut velocity, mut rotation) = query.single_mut();

    let mesh = meshes.get(mesh_handle.0.id).unwrap();

    // Check if moving would get us out of bounds.
    let mut new_transform = *transform;
    new_transform.translation.x += velocity.0.x * TIME_STEP;
    new_transform.translation.y += velocity.0.y * TIME_STEP;
    let collision = mesh_collides_with_wall(mesh, &new_transform);

    // If so, reverse direction
    if collision & WALL_COLLISION_HORIZONTAL > 0 {
        velocity.0.x *= -1.0;
    }
    if collision & WALL_COLLISION_VERTICAL > 0 {
        velocity.0.y *= -1.0;
    }

    // Only then update position. Have to recalculate because velocity could've changed.
    transform.translation.x += velocity.0.x * TIME_STEP;
    transform.translation.y += velocity.0.y * TIME_STEP;

    // Check if rotating would get us out of bounds.
    let mut new_transform = *transform;
    let rotate_by = Quat::from_rotation_z(rotation.0 * TIME_STEP);
    new_transform.rotation = transform.rotation.mul_quat(rotate_by);
    let collision = mesh_collides_with_wall(mesh, &new_transform);

    // If so, reverse rotation direction
    if collision > 0 {
        rotation.0 *= -1.0;
    }

    // Only then update rotation.
    let rotate_by = Quat::from_rotation_z(rotation.0 * TIME_STEP);
    transform.rotation = transform.rotation.mul_quat(rotate_by);

    // For deterministic mode, seed the RNG too.
    *velocity = accelerate_target(*velocity);
}

fn mesh_points<'a>(
    mesh: &'a Mesh,
    transform: &'a Transform,
) -> Option<impl Iterator<Item = Vec3> + 'a> {
    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        .map(|position| match position {
            // TODO first and last points are probably always the same - can probably skip first.
            VertexAttributeValues::Float32x3(points) => points
                .iter()
                .map(|vec3| transform.mul_vec3(Vec3::from(*vec3))),
            _ => unreachable!(),
        })
}
fn mesh_collides_with_wall(mesh: &Mesh, transform: &Transform) -> u8 {
    mesh_points(mesh, transform)
        .map(|points| {
            let mut collision = WALL_COLLISION_NONE;
            for point in points {
                if point.x <= -HALF_SCREEN_WIDTH || point.x >= HALF_SCREEN_WIDTH {
                    collision |= WALL_COLLISION_HORIZONTAL;
                } else if point.y <= -HALF_SCREEN_HEIGHT || point.y >= HALF_SCREEN_HEIGHT {
                    collision |= WALL_COLLISION_VERTICAL;
                }
            }
            collision
        })
        .unwrap_or(WALL_COLLISION_NONE)
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
