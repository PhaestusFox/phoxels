use std::marker::PhantomData;

use crate::{block::BlockId, core::*};
use bevy::{
    app::{App, Plugin, Update},
    color::palettes::css::BLACK,
    ecs::schedule::IntoScheduleConfigs,
    math::UVec3,
    prelude::Vec3,
    render::{mesh::Mesh, primitives::Aabb},
};

pub use manager::GeneratorLimits;
use manager::{ChunkGenerator, ChunkMesher};

pub(crate) mod manager;

pub const CHUNK_SIZE: ChunkSize = ChunkSize::Medium;

pub enum ChunkSize {
    Small = 8,
    Medium = 16,
    Large = 32,
}

impl ChunkSize {
    #[inline(always)]
    pub const fn size(&self) -> u32 {
        match self {
            ChunkSize::Small => 8,
            ChunkSize::Medium => 16,
            ChunkSize::Large => 32,
        }
    }

    #[inline(always)]
    pub const fn aria(&self) -> u32 {
        self.size() * self.size()
    }

    #[inline(always)]
    pub const fn volume(&self) -> u32 {
        self.size() * self.size() * self.size()
    }

    pub const fn bits_per_axis(&self) -> u32 {
        match self {
            ChunkSize::Small => 4,
            ChunkSize::Medium => 5,
            ChunkSize::Large => 6,
        }
    }
}

#[derive(bevy::prelude::Component, Clone, Debug)]
#[component(on_insert = ChunkData::on_insert, on_remove = ChunkData::on_remove)]
#[require(Aabb = Aabb::from_min_max(
    Vec3::ZERO,
    Vec3::ONE * CHUNK_SIZE.size() as f32,
))]
pub struct ChunkData {
    blocks: Vec<BlockId>,
    block_meta: [BlockMeta; 256],
    meta_fills: (u128, u128),
    size: UVec3,
    #[cfg(feature = "diagnostics")]
    count: usize,
}

impl ChunkData {
    pub fn empty() -> Self {
        ChunkData {
            blocks: vec![BlockId(0); CHUNK_SIZE.volume() as usize],
            meta_fills: (0, 0),
            size: UVec3::splat(CHUNK_SIZE.size()),
            block_meta: [BlockMeta::EMPTY; 256],
            #[cfg(feature = "diagnostics")]
            count: 0,
        }
    }

    pub fn solid(block: impl Block) -> Self {
        let mut block_meta = [BlockMeta::EMPTY; 256];
        block_meta[block.id() as usize] = BlockMeta::from(block);
        let meta_fills = if block.id() < 128 {
            (1 << block.id() as usize, 0)
        } else {
            (0, 1 << (block.id() - 128) as usize)
        };
        ChunkData {
            blocks: vec![BlockId(block.id()); CHUNK_SIZE.volume() as usize],
            size: UVec3::splat(CHUNK_SIZE.size()),
            block_meta,
            meta_fills,
            #[cfg(feature = "diagnostics")]
            count: CHUNK_SIZE.volume() as usize,
        }
    }

    #[inline(always)]
    fn set_block_unchecked(&mut self, x: u32, y: u32, z: u32, block: u8) {
        let index = self.get_index(x, y, z);
        self.blocks[index] = BlockId(block);
    }

    /// Set the block at the given coordinates
    /// Panics if the coordinates are out of bounds
    pub fn set_block(&mut self, x: u32, y: u32, z: u32, block: impl Block) {
        debug_assert!(
            x < self.size.x && y < self.size.y && z < self.size.z,
            "block index out of bounds: ({}, {}, {})",
            x,
            y,
            z
        );

        self.add_meta(block);

        let block_id = block.id();
        #[cfg(feature = "diagnostics")]
        let meta = BlockMeta::from(block);
        #[cfg(feature = "diagnostics")]
        if self.blocks[self.get_index(x, y, z)] != block_id {
            if meta == BlockMeta::EMPTY {
                self.count -= 1;
            } else {
                self.count += 1;
            }
        } else {
            return;
        }

        self.set_block_unchecked(x, y, z, block_id);
    }

    #[inline(always)]
    pub fn add_meta(&mut self, block: impl Block) {
        let add = if block.id() < 128 {
            self.meta_fills.0 & (1 << block.id())
        } else {
            self.meta_fills.1 & (1 << (block.id() - 128))
        } == 0;
        if add {
            self.block_meta[block.id() as usize] = BlockMeta::from(block);
            if block.id() < 128 {
                self.meta_fills.0 |= 1 << block.id()
            } else {
                self.meta_fills.1 |= 1 << (block.id() - 128)
            };
        }
    }

    /// Get the block meta at the given coordinates
    /// Returns None if out of bounds
    #[inline(always)]
    pub fn get_block_meta(&self, x: u32, y: u32, z: u32) -> Option<BlockMeta> {
        self.block_meta
            .get(self.get_block_id(x, y, z)?.0 as usize)
            .copied()
    }

    /// Get the block at the given coordinates
    /// return BlockMeta::EMPTY if out of bounds
    #[inline(always)]
    pub fn block_meta(&self, x: u32, y: u32, z: u32) -> BlockMeta {
        self.get_block_meta(x, y, z).unwrap_or(BlockMeta::EMPTY)
    }

    pub fn get_block_id(&self, x: u32, y: u32, z: u32) -> Option<BlockId> {
        if self.in_bounds(x, y, z) {
            Some(self.blocks[self.get_index(x, y, z)])
        } else {
            None
        }
    }

    pub fn texture(&self, x: u32, y: u32, z: u32) -> u32 {
        if self.in_bounds(x, y, z) {
            self.blocks[self.get_index(x, y, z)].0 as u32
        } else {
            0
        }
    }

    #[inline(always)]
    pub fn in_bounds(&self, x: u32, y: u32, z: u32) -> bool {
        x < self.size.x && y < self.size.y && z < self.size.z
    }

    #[inline(always)]
    pub fn get_index(&self, x: u32, y: u32, z: u32) -> usize {
        (y * self.size.z * self.size.x + z * self.size.x + x) as usize
    }

    #[inline(always)]
    pub(crate) async fn generate_mesh(self) -> Mesh {
        mesh_gen::make_mesh(self)
    }

    fn on_insert(
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
            diagnostics.loaded += c;
        }
        #[cfg(feature = "log")]
        bevy::log::trace!("Chunk({:?}) added to meshing que", ctx.entity);
        world.resource_mut::<ChunkMesher>().add_to_queue(ctx.entity);
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
            diagnostics.loaded -= c;
        }
    }
    #[cfg(feature = "diagnostics")]
    fn update_count(&mut self) {
        let mut filled = 0;
        for block in self.blocks.iter() {
            if self.block_meta[block.0 as usize] != BlockMeta::EMPTY {
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

pub(crate) mod mesh_gen;

// #[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut)]
// #[component(immutable, on_insert = ChunkId::on_insert)]
// #[require(Transform, Visibility)]
// //Aabb=Aabb::from_min_max(Vec3::NEG_ONE * CHUNK_SIZE as f32 / 2., Vec3::ONE * CHUNK_SIZE as f32 / 2.)
// pub struct ChunkId(IVec3);

// impl ChunkId {
//     pub fn new(x: i32, y: i32, z: i32) -> Self {
//         ChunkId(IVec3::new(x, y, z))
//     }

//     pub fn origin(&self) -> Vec3 {
//         (self.0 * CHUNK_SIZE.size() as i32).as_vec3()
//     }

//     fn on_insert(
//         mut world: bevy::ecs::world::DeferredWorld,
//         ctx: bevy::ecs::component::HookContext,
//     ) {
//         let id = *world
//             .entity(ctx.entity)
//             .get::<ChunkId>()
//             .expect("onadd of ChunkId");
//         world
//             .entity_mut(ctx.entity)
//             .get_mut::<Transform>()
//             .expect("ChunkId Requires Transform")
//             .translation = id.origin();
//         world
//             .resource_mut::<ChunkManager>()
//             .add_chunk(id, ctx.entity);

//         if world.entity(ctx.entity).get::<ChunkData>().is_some() {
//             world.resource_mut::<ChunkMesher>().add_to_queue(id);
//         } else {
//             world.resource_mut::<ChunkGenerator>().add_to_queue(id);
//         }
//     }
// }

pub struct ChunkPlugin<T: PhoxelGeneratorData = ()>(PhantomData<T>);

impl<T: PhoxelGeneratorData> Default for ChunkPlugin<T> {
    fn default() -> Self {
        ChunkPlugin(PhantomData)
    }
}

impl<T: PhoxelGeneratorData> Plugin for ChunkPlugin<T> {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkGenerator>()
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
                manager::start_generating_chunk_data::<T>,
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

#[test]
fn add_mesh_works() {
    #[derive(Clone, Copy)]
    struct TestBlock;
    impl Block for TestBlock {
        fn id(&self) -> u8 {
            1
        }
        fn is_solid(&self) -> bool {
            true
        }
        fn is_transparent(&self) -> bool {
            false
        }
    }
    let mut chunk = ChunkData::empty();
    assert_eq!(chunk.block_meta, [BlockMeta::EMPTY; 256]);
    assert_ne!(chunk.block_meta, [BlockMeta(1); 256]);
    chunk.set_block(0, 0, 0, TestBlock);
    assert_ne!(chunk.block_meta, [BlockMeta::EMPTY; 256]);
}
