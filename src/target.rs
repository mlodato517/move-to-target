use super::{utils, SIZE, TIME_STEP};

use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use rand::prelude::*;

const INITIAL_TARGET_SPEED: f32 = 20.0;
const MIN_TARGET_SPEED: f32 = 10.0;
const MAX_TARGET_SPEED: f32 = 40.0;
const TARGET_ROT_SPEED: f32 = 0.1;

#[derive(Component)]
pub struct Target {
    points: Vec<[f32; 3]>,
}
#[derive(Clone, Copy, Component)]
pub struct Velocity(Vec2);
#[derive(Clone, Copy, Component)]
pub struct Rotation(f32);

pub fn spawn_target(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // HACK
    let level_data = utils::generate_levels().pop().unwrap();
    let target_points = utils::mesh_points_raw(&level_data.target).unwrap().clone();
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(level_data.target)),
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
}

pub fn move_target(mut query: Query<(&mut Transform, &Target, &mut Velocity, &mut Rotation)>) {
    let (mut transform, target, mut velocity, mut rotation) = query.single_mut();

    // Check if moving would get us out of bounds.
    let old_x = transform.translation.x;
    let old_y = transform.translation.y;
    transform.translation.x += velocity.0.x * TIME_STEP;
    transform.translation.y += velocity.0.y * TIME_STEP;
    let collision = utils::points_collide_with_wall(&target.points, &transform);

    if collision & utils::WALL_COLLISION_HORIZONTAL > 0 {
        transform.translation.x = old_x;
        velocity.0.x *= -1.0;
    }
    if collision & utils::WALL_COLLISION_VERTICAL > 0 {
        transform.translation.y = old_y;
        velocity.0.y *= -1.0;
    }

    // Check if rotating would get us out of bounds.
    let old_rotation = transform.rotation;
    let rotate_by = Quat::from_rotation_z(rotation.0 * TIME_STEP);
    transform.rotation = transform.rotation.mul_quat(rotate_by);
    let collision = utils::points_collide_with_wall(&target.points, &transform);

    if collision > 0 {
        transform.rotation = old_rotation;
        rotation.0 *= -1.0;
    }

    // TODO For deterministic mode, seed the RNG too.
    *velocity = accelerate_target(*velocity);
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
