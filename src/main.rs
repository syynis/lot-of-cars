use bevy::prelude::*;
use game::GamePlugin;
use lifetime::LifetimePlugin;

mod game;
pub mod lifetime;

fn main() {
    let mut app = App::new();
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
        LifetimePlugin,
    ))
    .insert_resource(ClearColor(Color::ANTIQUE_WHITE))
    .insert_resource(Msaa::Off)
    .add_systems(Startup, setup);

    #[cfg(feature = "inspector")]
    app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());

    #[cfg(debug_assertions)]
    {
        app.add_plugins(bevy_pancam::PanCamPlugin);
    }

    app.run();
}

#[derive(Component, Default)]
struct GameCamera;
fn setup(mut cmds: Commands) {
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
}
