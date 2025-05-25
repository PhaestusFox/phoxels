use bevy::{input::common_conditions::input_just_pressed, prelude::*};

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
    app.add_systems(Update, warn_no_textures);
    app.run();
}

fn warn_no_textures(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut done: Local<bool>,
    texture: Res<map::BlockDescriptor>,
) {
    if *done {
        return;
    }
    if let bevy::asset::LoadState::Failed(_) = asset_server.load_state(&texture.terrain) {
        commands.spawn((
            Text(
                r#"
Failed to load terrain.png;
you can get one from https://bdcraft.net/
see credit.txt for details"#
                    .to_string(),
            ),
            TextFont {
                font_size: 75.,
                ..Default::default()
            },
        ));
        *done = true;
    }
    if asset_server.is_loaded(&texture.terrain) {
        *done = true;
    }
}
