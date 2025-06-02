mod block;
mod chunk;
mod simple_shader;

pub mod core {
    pub use crate::block::BlockMeta;
    pub use crate::chunk::CHUNK_SIZE;
    pub use crate::chunk::manager::PhoxelGeneratorData;
    pub use crate::prelude::*;
}

pub mod dev {
    pub use crate::chunk::mesh_gen::make_mesh;
}

#[cfg(feature = "diagnostics")]
mod diagnostics;

pub mod prelude {
    pub use crate::PhoxelsPlugin;
    pub use crate::block::Block;
    pub use crate::block::BlockId;
    pub use crate::chunk::ChunkData;
    pub use crate::chunk::ChunkSets;
    pub use crate::chunk::GeneratorLimits;
    pub use crate::chunk::manager::PhoxelGenerator;
    #[cfg(feature = "diagnostics")]
    pub use crate::diagnostics::VoxelCount;
    pub use crate::simple_shader::VoxelMaterial;
    pub use crate::simple_shader::{BlockOverride, BlockOverrides};
}

pub mod utils;

use core::{Block, PhoxelGeneratorData};
use std::marker::PhantomData;

pub use crate::chunk::manager::{ChunkGenerator, ChunkMesher};

use bevy::prelude::{App, Plugin};

pub struct PhoxelsPlugin<T: Block, D: PhoxelGeneratorData = ()>(PhantomData<(T, D)>);

impl<D: PhoxelGeneratorData, T: Block + Default> Plugin for PhoxelsPlugin<T, D> {
    fn build(&self, app: &mut App) {
        app.add_plugins(chunk::ChunkPlugin::<T, D>::default());
        app.add_plugins(simple_shader::VoxelShaderPlugin);
        #[cfg(feature = "diagnostics")]
        app.add_plugins(diagnostics::PhoxelDiagnostics::<T>::default());
    }
}

impl<T: Block, D: PhoxelGeneratorData> Default for PhoxelsPlugin<T, D> {
    fn default() -> Self {
        PhoxelsPlugin(PhantomData)
    }
}
