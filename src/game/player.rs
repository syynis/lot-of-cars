use bevy::prelude::*;
use bevy_xpbd_2d::{math::Scalar, prelude::*};
use leafwing_input_manager::prelude::*;

use super::{GameAssets, GameState};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());
        app.add_systems(
            OnEnter(GameState::Play),
            (setup, apply_deferred)
                .chain()
                .run_if(in_state(GameState::Play)),
        );
        app.add_systems(Update, movement.run_if(in_state(GameState::Play)));
    }
}

#[derive(Component, Clone)]
pub struct Player;

fn player_actions() -> InputMap<PlayerAction> {
    use PlayerAction::*;
    let mut input_map = InputMap::default();

    input_map.insert(KeyCode::W, Up);
    input_map.insert(KeyCode::D, Right);
    input_map.insert(KeyCode::S, Down);
    input_map.insert(KeyCode::A, Left);

    input_map
}

#[derive(Actionlike, Clone, Copy, Hash, Debug, PartialEq, Eq, Reflect)]
enum PlayerAction {
    Up,
    Right,
    Down,
    Left,
}

fn setup(mut cmds: Commands, assets: Res<GameAssets>) {
    cmds.spawn((
        (InputManagerBundle::<PlayerAction> {
            input_map: player_actions(),
            ..default()
        },),
        Name::new("PlayerActions"),
    ));

    cmds.spawn((
        Player,
        RigidBody::Kinematic,
        LinearVelocity::default(),
        AngularVelocity::default(),
        Position::default(),
        Collider::cuboid(16., 16.),
        SpriteBundle {
            texture: assets.player.clone_weak(),
            ..default()
        },
    ));
}

fn movement(
    actions: Query<&ActionState<PlayerAction>>,
    mut player: Query<&mut LinearVelocity, With<Player>>,
    time: Res<Time>,
) {
    let Ok(actions) = actions.get_single() else {
        return;
    };

    let mut vel = player.get_single_mut().expect("Player should exist");

    let left = actions.pressed(PlayerAction::Left);
    let right = actions.pressed(PlayerAction::Right);
    let up = actions.pressed(PlayerAction::Up);
    let down = actions.pressed(PlayerAction::Down);

    let horizontal = right as i8 - left as i8;
    let vertical = up as i8 - down as i8;
    let direction = Vec2::new(horizontal as Scalar, vertical as Scalar);

    if direction.length_squared() != 0.0 {
        vel.0 += direction * 1024. * time.delta_seconds();
    }
    vel.0 *= 0.8;
}
