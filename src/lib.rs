use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "standerd_position")]
        app.insert_resource(WireframeConfig {
            global: true,
            ..Default::default()
        });
        #[cfg(feature = "standerd_position")]
        app.add_plugins(WireframePlugin::default());
        app.insert_resource(phoxels::prelude::GeneratorLimits {
            max_generating_chunks: 100,
            max_meshing_chunks: 100,
        });
        app.add_plugins(phoxels::PhoxelsPlugin);
    }
}
