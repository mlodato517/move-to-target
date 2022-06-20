mod player;
mod target;
mod utils;

use std::time::Duration;

use bevy::core::Stopwatch;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

const TIME_STEP: f32 = 1.0 / 60.0;

const SIZE: Vec3 = bevy::math::const_vec3!([80.0, 80.0, 0.0]);
const TOLERANCE: f32 = 5.0;
const ANGLE_TOLERANCE: f32 = 0.15;

// TODO use bevy::window::Window
const HALF_SCREEN_WIDTH: f32 = 200.0;
const HALF_SCREEN_HEIGHT: f32 = 200.0;

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
        .add_system_set(
            SystemSet::on_enter(GameState::Level)
                .with_system(setup_level)
                .with_system(target::spawn_target)
                .with_system(player::spawn_players),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Level)
                .with_system(target::move_target)
                .with_system(player::move_players)
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

    // Spawn walls.
    // TODO follow https://github.com/bevyengine/bevy/blob/latest/examples/games/breakout.rs.
    let walls = utils::poly_mesh(vec![
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
    entities: Query<Entity, Or<(With<player::Player>, With<target::Target>)>>,
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

fn check_positions(
    players_query: Query<&Transform, With<player::Player>>,
    target_query: Query<&Transform, With<target::Target>>,
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
