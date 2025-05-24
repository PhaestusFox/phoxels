use bevy::prelude::*;

mod diganostics;
mod map;
mod player;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: bevy::window::PresentMode::AutoNoVsync,
                ..Default::default()
            }),
            ..Default::default()
        }),
        bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
    ));
    app.add_plugins((player::plugin, diganostics::plugin, map::plugin));
    app.run();
}
