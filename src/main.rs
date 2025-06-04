use bevy::{
    input::common_conditions::input_just_pressed,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};

use phoxels_game::*;
// mod simple_shader;

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
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                // Wasm builds will check for meta files (that don't exist) if this isn't set.
                // This causes errors and even panics in web builds on itch.
                // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            }),
        bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
    ));
    app.add_plugins(phoxels_game::GamePlugin);
    // StandardMaterial
    // app.add_plugins(bevy_mod_debugdump::CommandLineArgs);
    // app.add_plugins(simple_shader::VoxelShaderPlugin);
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
