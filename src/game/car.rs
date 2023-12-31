use rand::*;
use std::time::Duration;

use bevy::{math::cubic_splines::CubicCurve, prelude::*, time::common_conditions::on_timer};
use bevy_xpbd_2d::prelude::*;

use crate::lifetime::Lifetime;

use super::{player::Player, GameAssets, GameCamera, GameState};

pub struct CarPlugin;

impl Plugin for CarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_cars.run_if(on_timer(Duration::from_millis(500))),
                handle_trajectory,
                car_sprite_from_rotation,
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
struct CarCollider;

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

    pub fn vel(&self) -> Vec2 {
        self.curve.velocity(self.t)
    }
}

fn spawn_cars(
    mut cmds: Commands,
    camera_q: Query<&OrthographicProjection, With<GameCamera>>,
    assets: Res<GameAssets>,
) {
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
        if rng.gen_bool(0.5) {
            (origin, end)
        } else {
            (end, origin)
        }
    } else {
        let origin_y = rng.gen_range(min.y..(min.y + size.y));
        let end_y = rng.gen_range(min.y..(max.y + size.y));

        let origin = Vec2::new(min.x, origin_y);
        let end = Vec2::new(max.x, end_y);
        if rng.gen_bool(0.5) {
            (origin, end)
        } else {
            (end, origin)
        }
    };

    // Control point offsets
    let factor_c1 = rng.gen_range(-1000.0..1000.0);
    let factor_c2 = rng.gen_range(-1000.0..1000.0);
    // Duration of car travel (this controls speed, lower => faster)
    let duration = rng.gen_range(2.0..4.0);

    // Car size
    let car_size = Vec2::new(20., 10.);
    cmds.spawn((
        Car,
        Trajectory::new(origin, end, factor_c1, factor_c2, duration),
        Lifetime::new(duration + 0.1),
        SpriteSheetBundle {
            texture_atlas: assets.blue_car.clone_weak(),
            transform: Transform::from_translation(origin.extend(0.)),
            ..default()
        },
    ))
    .with_children(|parent| {
        parent.spawn((
            CarCollider,
            RigidBody::Kinematic,
            Collider::cuboid(car_size.x, car_size.y),
            SpatialBundle::default(),
        ));
    });
}

fn car_sprite_from_rotation(
    mut car_q: Query<(&mut TextureAtlasSprite, &Children), With<Car>>,
    collider_q: Query<&Rotation, With<CarCollider>>,
) {
    for (mut sprite, children) in car_q.iter_mut() {
        let rotation = collider_q
            .get(*children.first().unwrap())
            .expect("Should have rotation");
        let angle = rotation.as_degrees();
        let angle = if angle.is_sign_positive() {
            (angle - 360.).abs()
        } else {
            angle.abs()
        }
        .clamp(0., 359.);
        let angle_per_index = 7.34; // 360 / 49

        let index = (angle / angle_per_index) as usize;
        sprite.index = index;
    }
}

fn handle_trajectory(
    mut trajectory_q: Query<(&mut Transform, &mut Trajectory, &Children), With<Car>>,
    mut collider_q: Query<&mut Transform, Without<Car>>,
    time: Res<Time>,
    #[cfg(debug_assertions)] mut gizmos: Gizmos,
) {
    let dt = time.delta_seconds();
    for (mut transform, mut trajectory, children) in trajectory_q.iter_mut() {
        transform.translation = trajectory.pos().extend(transform.translation.z);
        let vel = trajectory.vel().normalize_or_zero();

        let mut collider_transform = collider_q
            .get_mut(*children.first().unwrap())
            .expect("Should have transform");
        collider_transform.rotation = Quat::from_rotation_arc_2d(Vec2::X, vel);

        trajectory.t += dt / trajectory.duration;

        // Debug
        #[cfg(debug_assertions)]
        {
            let subdivisions = 0;
            for (start, vel) in trajectory
                .curve
                .iter_positions(subdivisions)
                .zip(trajectory.curve.iter_velocities(subdivisions))
            {
                gizmos.line_2d(start, start + vel.normalize_or_zero() * 50., Color::RED);
            }
        }
    }
}

fn car_car_contact(
    mut cmds: Commands,
    mut collision_events: EventReader<CollisionStarted>,
    collider_q: Query<&Parent, With<CarCollider>>,
) {
    for collision in collision_events.read() {
        if let Ok([e1, e2]) = collider_q.get_many([collision.0, collision.1]) {
            cmds.entity(e1.get()).despawn_recursive();
            cmds.entity(e2.get()).despawn_recursive();
        }
    }
}

fn car_player_contact(
    mut collision_events: EventReader<CollisionStarted>,
    collider_q: Query<(), (With<CarCollider>, With<Parent>)>,
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
            if collider_q.get(other).is_ok() {
                **pos = Vec2::ZERO;
            }
        }
    }
}
