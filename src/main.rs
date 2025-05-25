use bevy::{pbr::PbrPlugin, prelude::*};

mod diganostics;
mod map;
mod player;
mod shader;
mod simple_shader;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(ImagePlugin::default_nearest()),
        bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
    ));
    // let world_normal = normalize( cross( dpdy( in.world_position.xyz ), dpdx( in.world_position.xyz ) ) );
    // StandardMaterial
    app.add_plugins(bevy_mod_debugdump::CommandLineArgs);
    app.add_plugins(simple_shader::VoxelShaderPlugin);
    app.add_plugins((player::plugin, diganostics::plugin));
    app.add_plugins(map::plugin);
    app.run();
}
