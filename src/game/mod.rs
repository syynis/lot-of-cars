pub mod car;
pub mod player;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_xpbd_2d::prelude::*;

use crate::lifetime::LifetimePlugin;

use self::{car::CarPlugin, player::PlayerPlugin};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::rgb_u8(74, 84, 98)));

        app.add_plugins((
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            PlayerPlugin,
            CarPlugin,
            LifetimePlugin,
        ));

        app.add_state::<GameState>()
            .add_loading_state(
                LoadingState::new(GameState::AssetLoading).continue_to_state(GameState::Play),
            )
            .add_collection_to_loading_state::<_, GameAssets>(GameState::AssetLoading);
        app.add_systems(OnEnter(GameState::Play), setup);
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    AssetLoading,
    Play,
}

#[derive(Resource, Reflect, Default, AssetCollection, Debug)]
#[reflect(Resource)]
pub struct GameAssets {
    #[asset(path = "background.png")]
    pub background: Handle<Image>,
    #[asset(texture_atlas(tile_size_x = 8., tile_size_y = 8., columns = 3, rows = 3))]
    #[asset(path = "player.png")]
    pub player: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 25., tile_size_y = 25., columns = 7, rows = 7))]
    #[asset(path = "blue_car.png")]
    pub blue_car: Handle<TextureAtlas>,
}

#[derive(Component, Default)]
struct GameCamera;
fn setup(mut cmds: Commands, assets: Res<GameAssets>) {
    cmds.spawn((
        Camera2dBundle::default(),
        #[cfg(debug_assertions)]
        bevy_pancam::PanCam {
            grab_buttons: vec![MouseButton::Middle],
            enabled: true,
            ..default()
        },
        GameCamera,
    ));

    cmds.spawn(SpriteBundle {
        texture: assets.background.clone_weak(),
        transform: Transform::from_translation(Vec2::ZERO.extend(-100.))
            .with_scale(Vec3::new(1.5, 1.5, 1.)),
        ..default()
    });
}
