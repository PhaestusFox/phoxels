use bevy::{pbr::PbrPlugin, prelude::*};

mod diganostics;
mod map;
mod player;
mod shader;

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
    // let world_normal = normalize( cross( dpdy( in.world_position.xyz ), dpdx( in.world_position.xyz ) ) );
    // StandardMaterial
    app.add_plugins(bevy_mod_debugdump::CommandLineArgs);
    app.add_plugins(shader::MyMaterialPlugin);
    app.add_plugins((player::plugin, diganostics::plugin, map::plugin));
    app.run();
}
