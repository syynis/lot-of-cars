use rand::*;
use std::time::Duration;

use bevy::{math::cubic_splines::CubicCurve, prelude::*, time::common_conditions::on_timer};
use bevy_xpbd_2d::prelude::*;

use crate::{lifetime::Lifetime, GameCamera};

use super::{player::Player, GameState};

pub struct CarPlugin;

impl Plugin for CarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_cars.run_if(on_timer(Duration::from_millis(3000))),
                handle_trajectory,
                car_car_contact.run_if(on_event::<CollisionStarted>()),
                car_player_contact.run_if(on_event::<CollisionStarted>()),
            )
                .run_if(in_state(GameState::Play)),
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

    pub fn pos(&self) -> Vec2 {
        self.curve.position(self.t)
    }
}

fn spawn_cars(mut cmds: Commands, camera_q: Query<&OrthographicProjection, With<GameCamera>>) {
    let Ok(proj) = camera_q.get_single() else {
        return;
    };

    let mut rng = rand::thread_rng();

    let min = proj.area.min;
    let max = proj.area.max;
    let size = proj.area.size();

    let (origin, end) = if rng.gen_bool(0.5) {
        let origin_x = rng.gen_range(min.x..(min.x + size.x));
        let end_x = rng.gen_range(min.x..(max.x + size.x));

        let origin = Vec2::new(origin_x, min.y);
        let end = Vec2::new(end_x, size.y);
        (origin, end)
    } else {
        let origin_y = rng.gen_range(min.y..(min.y + size.y));
        let end_y = rng.gen_range(min.y..(max.y + size.y));

        let origin = Vec2::new(min.x, origin_y);
        let end = Vec2::new(max.x, end_y);
        (origin, end)
    };

    // Control point offsets
    let factor_c1 = rng.gen_range(-1000.0..1000.0);
    let factor_c2 = rng.gen_range(-1000.0..1000.0);
    // Duration of car travel (this controls speed, lower => faster)
    let duration = rng.gen_range(12.5..14.5);

    // Car size
    let car_size = Vec2::new(10., 18.);
    cmds.spawn((
        Car,
        Trajectory::new(origin, end, factor_c1, factor_c2, duration),
        Lifetime::new(duration + 0.1),
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(car_size),
                ..default()
            },
            ..default()
        },
        LinearVelocity::default(),
        AngularVelocity::default(),
        RigidBody::Kinematic,
        Collider::cuboid(car_size.x, car_size.y),
        Position::new(origin),
    ));
}

fn handle_trajectory(
    mut trajectory_q: Query<(&mut Position, &mut Trajectory)>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    for (mut pos, mut trajectory) in trajectory_q.iter_mut() {
        let dt = time.delta_seconds();
        **pos = trajectory.pos();

        // TODO change rotation

        let subdivisions = 20;
        for (start, vel) in trajectory
            .curve
            .iter_positions(subdivisions)
            .zip(trajectory.curve.iter_velocities(subdivisions))
        {
            gizmos.line_2d(start, start + vel.normalize_or_zero() * 100., Color::RED);
        }

        let t = trajectory.t + (dt / trajectory.duration);
        trajectory.t = t;
    }
}

fn car_car_contact(
    mut cmds: Commands,
    mut collision_events: EventReader<CollisionStarted>,
    car_q: Query<Entity, With<Car>>,
) {
    for collision in collision_events.read() {
        if let Ok([e1, e2]) = car_q.get_many([collision.0, collision.1]) {
            cmds.entity(e1).despawn_recursive();
            cmds.entity(e2).despawn_recursive();
        }
    }
}

fn car_player_contact(
    mut collision_events: EventReader<CollisionStarted>,
    car_q: Query<Entity, With<Car>>,
    mut player_q: Query<(Entity, &mut Position), With<Player>>,
) {
    let Ok((player_entity, mut pos)) = player_q.get_single_mut() else {
        return;
    };
    for collision in collision_events.read() {
        let other = if collision.0 == player_entity {
            Some(collision.1)
        } else if collision.1 == player_entity {
            Some(collision.0)
        } else {
            None
        };

        if let Some(other) = other {
            if car_q.get(other).is_ok() {
                **pos = Vec2::ZERO;
            }
        }
    }
}
