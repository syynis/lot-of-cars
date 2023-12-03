use bevy::{math::cubic_splines::CubicCurve, prelude::*};
use bevy_xpbd_2d::prelude::*;

use crate::lifetime::Lifetime;

use super::GameState;

pub struct CarPlugin;

impl Plugin for CarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_car, handle_trajectory).run_if(in_state(GameState::Play)),
        );
    }
}

#[derive(Component)]
pub struct Car;

#[derive(Component)]
pub struct Trajectory {
    curve: CubicCurve<Vec2>,
    t: f32,
    duration: f32,
}

impl Trajectory {
    pub fn new(start: Vec2, end: Vec2, factor_c1: f32, factor_c2: f32, duration: f32) -> Self {
        let third = start.distance(end) / 3.;
        let dir = (end - start).normalize_or_zero();
        let perp = dir.perp();
        let c1 = start + dir * third + perp * factor_c1;
        let c2 = start + dir * third * 2. + perp * factor_c2;
        Self {
            curve: CubicBezier::new([[start, c1, c2, end]]).to_curve(),
            t: 0.,
            duration,
        }
    }
}

fn spawn_car(mut cmds: Commands, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::F) {
        let duration = 2.;
        cmds.spawn((
            Car,
            Trajectory::new(Vec2::ZERO, Vec2::ONE * 500., 250., 1000., duration),
            Lifetime::new(duration + 0.1),
            SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::splat(32.)),
                    ..default()
                },
                ..default()
            },
            LinearVelocity::default(),
            AngularVelocity::default(),
            RigidBody::Kinematic,
            Collider::cuboid(32., 32.),
            Position::default(),
        ));
    }
}

fn handle_trajectory(
    mut trajectory_q: Query<(&mut Position, &mut Trajectory)>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    for (mut pos, mut trajectory) in trajectory_q.iter_mut() {
        let dt = time.delta_seconds();
        **pos = trajectory.curve.position(trajectory.t);
        let subdivisions = 20;
        for (start, vel) in trajectory
            .curve
            .iter_positions(subdivisions)
            .zip(trajectory.curve.iter_velocities(subdivisions))
        {
            gizmos.line_2d(start, start + vel.normalize_or_zero() * 10., Color::RED);
        }

        let t = trajectory.t + (dt / trajectory.duration);
        trajectory.t = t;
    }
}
