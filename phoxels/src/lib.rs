mod block;
mod chunk;
mod simple_shader;

pub mod core {
    pub use crate::block::BlockMeta;
    pub use crate::prelude::*;
}

#[cfg(feature = "diagnostics")]
mod diagnostics;

pub mod prelude {
    pub use crate::PhoxelsPlugin;
    pub use crate::block::Block;
    pub use crate::chunk::ChunkData;
    pub use crate::chunk::ChunkId;
    pub use crate::chunk::ChunkSets;
    pub use crate::chunk::GeneratorLimits;
    pub use crate::chunk::manager::ChunkManager;
    pub use crate::chunk::manager::PhoxelGenerator;
    #[cfg(feature = "diagnostics")]
    pub use crate::diagnostics::VoxelCount;
    pub use crate::simple_shader::VoxelMaterial;
}

pub use crate::chunk::manager::{ChunkGenerator, ChunkMesher};

use bevy::prelude::{App, Plugin};

pub struct PhoxelsPlugin;

impl Plugin for PhoxelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((chunk::ChunkPlugin, simple_shader::VoxelShaderPlugin));
        #[cfg(feature = "diagnostics")]
        app.add_plugins(diagnostics::PhoxelDiagnostics);
    }
}
