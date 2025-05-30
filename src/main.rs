use bevy::{input::common_conditions::input_just_pressed, prelude::*};

mod diganostics;
mod map;
mod player;
mod shader;
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
            .set(ImagePlugin::default_nearest()),
        bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
    ));
    app.insert_resource(phoxels::prelude::GeneratorLimits {
        max_generating_chunks: 100,
        max_meshing_chunks: 100,
    });
    app.add_plugins(phoxels::PhoxelsPlugin);
    // StandardMaterial
    // app.add_plugins(bevy_mod_debugdump::CommandLineArgs);
    // app.add_plugins(simple_shader::VoxelShaderPlugin);
    app.add_plugins((player::plugin, diganostics::plugin));
    app.add_plugins(map::plugin);
    app.add_systems(Update, warn_no_textures);
    app.add_systems(Update, test);
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

fn test(camera: Single<&GlobalTransform, With<Camera>>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::F12) {
        let b: bevy::math::Affine3 = (&camera.affine()).into();
        println!("{:.02?}", b.matrix3);
        let m: bevy::math::Affine3A = (&b).into();
        println!("{:.02?}", m.matrix3);
        let (scale, _rot, _translation) = m.to_scale_rotation_translation();
        println!("{:.02?}", scale);
    }
}
