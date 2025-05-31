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

use core::PhoxelGeneratorData;
use std::marker::PhantomData;

pub use crate::chunk::manager::{ChunkGenerator, ChunkMesher};

use bevy::prelude::{App, Plugin};

pub struct PhoxelsPlugin<T: PhoxelGeneratorData = ()>(PhantomData<T>);

impl<T: PhoxelGeneratorData> Plugin for PhoxelsPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            chunk::ChunkPlugin::<T>::default(),
            simple_shader::VoxelShaderPlugin,
        ));
        #[cfg(feature = "diagnostics")]
        app.add_plugins(diagnostics::PhoxelDiagnostics);
    }
}

impl<T: PhoxelGeneratorData> Default for PhoxelsPlugin<T> {
    fn default() -> Self {
        PhoxelsPlugin(PhantomData)
    }
}
