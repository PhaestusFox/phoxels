use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};
pub mod diganostics;
pub mod map;
pub mod player;
pub mod shader;
use map::ChunkId;

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
        app.add_plugins(phoxels::PhoxelsPlugin::<map::GeneratorDataType>::default());
    }
}
