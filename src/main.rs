use bevy::{asset::AssetMetaCheck, prelude::*};
use game::GamePlugin;

mod game;
mod lifetime;

fn main() {
    let mut app = App::new();

    // Fix for itch.io issue
    app.insert_resource(AssetMetaCheck::Never);

    app.edit_schedule(Main, |schedule| {
        schedule.set_build_settings(bevy::ecs::schedule::ScheduleBuildSettings {
            ambiguity_detection: bevy::ecs::schedule::LogLevel::Error,
            ..default()
        });
    });
    #[cfg(not(debug_assertions))]
    let log_plugin = bevy::log::LogPlugin {
        filter: "off".to_string(),
        ..default()
    };
    #[cfg(debug_assertions)]
    let log_plugin = bevy::log::LogPlugin {
        level: bevy::log::Level::DEBUG,
        filter: "info,wgpu_core=error,wgpu_hal=error".into(),
    };
    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Lot of Cars".to_string(),
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(log_plugin),
        GamePlugin,
    ))
    .insert_resource(Msaa::Off);

    #[cfg(feature = "inspector")]
    app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());

    #[cfg(debug_assertions)]
    {
        app.add_plugins(bevy_pancam::PanCamPlugin);
    }

    app.run();
}
