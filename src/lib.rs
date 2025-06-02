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
        app.add_plugins(phoxels::PhoxelsPlugin::<BlockType, map::GeneratorDataType>::default());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Air,
    Stone,
    Dirt,
    Cobblestone = 16,

    Furnuse = 44,
    Grass = 77,
}

impl phoxels::core::Block for BlockType {
    fn id(&self) -> u8 {
        *self as u8
    }
    fn is_solid(&self) -> bool {
        match self {
            BlockType::Air => false,
            BlockType::Stone | BlockType::Cobblestone | BlockType::Dirt | BlockType::Grass => true,
            BlockType::Furnuse => true,
        }
    }
    fn is_transparent(&self) -> bool {
        match self {
            BlockType::Air => true,
            BlockType::Stone | BlockType::Cobblestone | BlockType::Dirt | BlockType::Grass => false,
            BlockType::Furnuse => false,
        }
    }
}

impl Default for BlockType {
    fn default() -> Self {
        BlockType::Air
    }
}
