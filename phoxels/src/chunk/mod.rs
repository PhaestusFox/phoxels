use crate::core::*;
use bevy::{
    app::{App, Plugin, Update},
    ecs::schedule::IntoScheduleConfigs,
    prelude::{Component, Deref, DerefMut, IVec3, Transform, Vec3, Visibility},
    render::{mesh::Mesh, primitives::Aabb},
};

use manager::{ChunkGenerator, ChunkMesher};
pub use manager::{ChunkManager, GeneratorLimits};

pub(crate) mod manager;

const CHUNK_SIZE: ChunkSize = ChunkSize::Medium;

enum ChunkSize {
    Small = 8,
    Medium = 16,
    Large = 32,
}

impl ChunkSize {
    #[inline(always)]
    pub const fn size(&self) -> usize {
        match self {
            ChunkSize::Small => 8,
            ChunkSize::Medium => 16,
            ChunkSize::Large => 32,
        }
    }

    #[inline(always)]
    pub const fn aria(&self) -> usize {
        self.size() * self.size()
    }

    #[inline(always)]
    pub const fn volume(&self) -> usize {
        self.size() * self.size() * self.size()
    }
}

#[derive(bevy::prelude::Component, Clone)]
#[component(on_insert = ChunkData::on_insert, on_remove = ChunkData::on_remove)]
#[require(Aabb = Aabb::from_min_max(
    Vec3::ZERO,
    Vec3::ONE * CHUNK_SIZE.size() as f32,
))]
pub struct ChunkData {
    blocks: [BlockMeta; CHUNK_SIZE.volume()],
    #[cfg(feature = "diagnostics")]
    count: usize,
}

impl ChunkData {
    pub fn empty() -> Self {
        ChunkData {
            blocks: [BlockMeta::default(); CHUNK_SIZE.volume()],
            #[cfg(feature = "diagnostics")]
            count: 0,
        }
    }

    #[inline(always)]
    fn set_block_unchecked(&mut self, x: usize, y: usize, z: usize, block: impl Into<BlockMeta>) {
        let index = x + y * CHUNK_SIZE.size() + z * CHUNK_SIZE.aria();
        self.blocks[index] = block.into();
    }

    /// Set the block at the given coordinates
    /// Panics if the coordinates are out of bounds
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: impl Into<BlockMeta>) {
        debug_assert!(
            x < CHUNK_SIZE.size() && y < CHUNK_SIZE.size() && z < CHUNK_SIZE.size(),
            "block index out of bounds: ({}, {}, {})",
            x,
            y,
            z
        );
        #[cfg(feature = "diagnostics")]
        let block = block.into();
        #[cfg(feature = "diagnostics")]
        if self.blocks[x + y * CHUNK_SIZE.size() + z * CHUNK_SIZE.aria()] != block {
            if block == BlockMeta::EMPTY {
                self.count -= 1;
            } else {
                self.count += 1;
            }
        } else {
            return;
        }

        self.set_block_unchecked(x, y, z, block);
    }

    /// Get the block meta at the given coordinates
    /// Returns None if out of bounds
    pub fn get_block_meta(&self, x: usize, y: usize, z: usize) -> Option<&BlockMeta> {
        if x < CHUNK_SIZE.size() && y < CHUNK_SIZE.size() && z < CHUNK_SIZE.size() {
            let index = x + y * CHUNK_SIZE.size() + z * CHUNK_SIZE.aria();
            Some(&self.blocks[index])
        } else {
            None
        }
    }

    /// Get the block at the given coordinates
    /// return BlockMeta::EMPTY if out of bounds
    pub fn block(&self, x: usize, y: usize, z: usize) -> BlockMeta {
        let index = x + y * CHUNK_SIZE.size() + z * CHUNK_SIZE.aria();
        self.blocks.get(index).cloned().unwrap_or(BlockMeta::EMPTY)
    }

    /// Get the block at the given coordinates and convert it to the specified type
    pub fn get_block<T: From<BlockMeta>>(&self, x: usize, y: usize, z: usize) -> Option<T> {
        let block_meta = self.get_block_meta(x, y, z)?;
        Some(T::from(*block_meta))
    }

    pub(crate) async fn generate_mesh(self) -> Mesh {
        mesh_gen::make_mesh(self).await
    }

    fn on_insert(
        mut world: bevy::ecs::world::DeferredWorld,
        ctx: bevy::ecs::component::HookContext,
    ) {
        let Some(id) = world.entity(ctx.entity).get::<ChunkId>().cloned() else {
            #[cfg(feature = "log")]
            bevy::log::warn!("ChunkData has no ChunkId, cannot add to meshing queue");
            return;
        };
        #[cfg(feature = "diagnostics")]
        {
            let mut chunk_data = world.entity_mut(ctx.entity);
            let mut chunk_data = chunk_data
                .get_mut::<ChunkData>()
                .expect("ChunkData requires ChunkId");
            chunk_data.update_count();
            let c = chunk_data.voxel_count();
            let mut diagnostics = world.resource_mut::<crate::diagnostics::VoxelCount>();
            diagnostics.loaded += c as usize;
        }
        world.resource_mut::<ChunkMesher>().add_to_queue(id);
    }

    fn on_remove(
        mut world: bevy::ecs::world::DeferredWorld,
        ctx: bevy::ecs::component::HookContext,
    ) {
        #[cfg(feature = "diagnostics")]
        {
            let mut chunk_data = world.entity_mut(ctx.entity);
            let mut chunk_data = chunk_data
                .get_mut::<ChunkData>()
                .expect("ChunkData requires ChunkId");
            chunk_data.update_count();
            let c = chunk_data.voxel_count();
            let mut diagnostics = world.resource_mut::<crate::diagnostics::VoxelCount>();
            diagnostics.loaded -= c as usize;
        }
    }
    #[cfg(feature = "diagnostics")]
    fn update_count(&mut self) {
        let mut filled = 0;
        for block in self.blocks.iter() {
            if *block != BlockMeta::EMPTY {
                filled += 1;
            }
        }
        self.count = filled;
    }

    #[cfg(feature = "diagnostics")]
    pub fn voxel_count(&self) -> usize {
        self.count
    }
}

mod mesh_gen;
mod utils;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut)]
#[component(immutable, on_insert = ChunkId::on_insert)]
#[require(Transform, Visibility)]
//Aabb=Aabb::from_min_max(Vec3::NEG_ONE * CHUNK_SIZE as f32 / 2., Vec3::ONE * CHUNK_SIZE as f32 / 2.)
pub struct ChunkId(IVec3);

impl ChunkId {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        ChunkId(IVec3::new(x, y, z))
    }

    pub fn origin(&self) -> Vec3 {
        (self.0 * CHUNK_SIZE.size() as i32).as_vec3()
    }

    fn on_insert(
        mut world: bevy::ecs::world::DeferredWorld,
        ctx: bevy::ecs::component::HookContext,
    ) {
        let id = *world
            .entity(ctx.entity)
            .get::<ChunkId>()
            .expect("onadd of ChunkId");
        world
            .entity_mut(ctx.entity)
            .get_mut::<Transform>()
            .expect("ChunkId Requires Transform")
            .translation = id.origin();
        world
            .resource_mut::<ChunkManager>()
            .add_chunk(id, ctx.entity);

        if world.entity(ctx.entity).get::<ChunkData>().is_some() {
            world.resource_mut::<ChunkMesher>().add_to_queue(id);
        } else {
            world.resource_mut::<ChunkGenerator>().add_to_queue(id);
        }
    }
}

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkManager>()
            .init_resource::<ChunkGenerator>()
            .init_resource::<ChunkMesher>()
            .init_resource::<GeneratorLimits>();

        app.configure_sets(
            Update,
            ChunkSets::Generate
                .after(ChunkSets::Load)
                .before(ChunkSets::Mesh),
        );

        app.add_systems(
            Update,
            (
                manager::extract_finished_chunk_data,
                manager::start_generating_chunk_data,
            )
                .chain()
                .in_set(ChunkSets::Generate),
        );
        app.add_systems(
            Update,
            (
                manager::extract_finished_chunk_mesh,
                manager::start_generating_chunk_mesh,
            )
                .chain()
                .in_set(ChunkSets::Mesh),
        );
    }
}

#[derive(bevy::prelude::SystemSet, Hash, Debug, Clone, Eq, PartialEq)]
pub enum ChunkSets {
    /// Systems that run to load pre-existing ChunkData
    /// this is where you would put systems that load chunks from disk or network
    Load,
    /// Systems that run to generate ChunkData for ChunkId's With no data
    /// add a system.before() to update `ChunkLoaders` priority order to load chunks
    Generate,
    /// Systems that run to generate ChunkMesh for ChunkData
    /// add a system.before() to update `ChunkMesher` priority order to generate meshes
    Mesh,
}
