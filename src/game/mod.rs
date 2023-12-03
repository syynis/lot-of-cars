pub mod car;
pub mod player;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_xpbd_2d::prelude::*;

use self::{car::CarPlugin, player::PlayerPlugin};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            PlayerPlugin,
            CarPlugin,
        ));

        app.add_state::<GameState>()
            .add_loading_state(
                LoadingState::new(GameState::AssetLoading).continue_to_state(GameState::Play),
            )
            .add_collection_to_loading_state::<_, GameAssets>(GameState::AssetLoading);
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
    #[asset(path = "player.png")]
    pub player: Handle<Image>,
}
