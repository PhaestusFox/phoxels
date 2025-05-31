use std::sync::{Arc, RwLock};

use bevy::{
    asset::RenderAssetUsages,
    platform::collections::{HashMap, HashSet},
    prelude::*,
    render::primitives::Aabb,
    tasks::Task,
    text::cosmic_text::fontdb::Query,
};
use indexmap::IndexMap;
use noise::{Fbm, MultiFractal, NoiseFn, SuperSimplex};
use phoxels::core::{
    BlockMeta, BlockOverride, BlockOverrides, PhoxelGenerator, PhoxelGeneratorData,
};

pub type GeneratorDataType = ChunkId;

use crate::{
    diganostics::VoxelCount,
    player::Player,
    // simple_shader::{BLOCK_ID, BLOCK_POS, VoxelMaterial as CustomMaterial},
};

use phoxels::prelude::VoxelMaterial as CustomMaterial;

const CHUNK_SIZE: i32 = 16;
const CHUNK_ARIA: i32 = CHUNK_SIZE * CHUNK_SIZE;
const CHUNK_VOLUME: i32 = CHUNK_ARIA * CHUNK_SIZE;
const GROUND_HIGHT: i32 = 8;

const MAP_SIZE: i32 = 100;

pub fn plugin(app: &mut App) {
    app.init_resource::<BlockDescriptor>();
    // .init_resource::<MapDescriptor>()
    // .init_resource::<MapData>();
    app.add_systems(Startup, spawn_world);
    let map_descriptor = MapDescriptor::from_world(app.world_mut());
    app.insert_resource(map_descriptor.clone());
    let var = std::sync::Mutex::new(phoxels::utils::BlockIter::<201>::new());
    app.insert_resource(phoxels::prelude::PhoxelGenerator::new(
        move |id: GeneratorDataType| {
            let mut chunk = phoxels::prelude::ChunkData::empty();
            let map_descriptor = map_descriptor.read().unwrap();
            // let (id, _) = id;
            for x in 0..CHUNK_SIZE as usize {
                for z in 0..CHUNK_SIZE as usize {
                    let h = map_descriptor
                        .get_height(x as i32 + id.x * CHUNK_SIZE, z as i32 + id.z * CHUNK_SIZE);
                    for y in (id.y * CHUNK_SIZE)..(id.y + 1) * CHUNK_SIZE {
                        if y > h {
                            if x == 1 && z == 1 {
                                chunk.set_block(
                                    x,
                                    (y - id.y * CHUNK_SIZE) as usize,
                                    z,
                                    BlockType::Furnuse,
                                );
                            }
                            break;
                        }
                        let block = if y == h {
                            BlockType::Grass
                        } else if y + 3 > h {
                            BlockType::Dirt
                        } else {
                            BlockType::Stone
                        };
                        chunk.set_block(x, (y - id.y * CHUNK_SIZE) as usize, z, block);
                    }
                }
            }
            chunk
        },
    ));
}

// fn sort_gen_order(mut chunks: ResMut<phoxels::ChunkMesher>) {
//     chunks.set_priority(|c1, c2| {
//         if c1.abs().length_squared() > c2.abs().length_squared() {
//             std::cmp::Ordering::Greater
//         } else if c1.abs().length_squared() < c2.abs().length_squared() {
//             std::cmp::Ordering::Less
//         } else {
//             std::cmp::Ordering::Equal
//         }
//     })
// }

// fn sort_chunk_order(mut chunks: ResMut<phoxels::ChunkGenerator>) {
//     chunks.set_priority(|c1, c2| {
//         if c1.abs().length_squared() < c2.abs().length_squared() {
//             std::cmp::Ordering::Greater
//         } else if c1.abs().length_squared() > c2.abs().length_squared() {
//             std::cmp::Ordering::Less
//         } else {
//             std::cmp::Ordering::Equal
//         }
//     })
// }

#[derive(Debug, Component, PartialEq, Eq, Hash, Clone, Copy, Deref, Default, Reflect)]
#[component(immutable, on_insert = ChunkId::on_add)]
#[require(Transform, Visibility)]
pub struct ChunkId(IVec3);

//Send + Sync + Clone + QueryData + FromReflect + GetTypeRegistration + Bundle + Reflect
impl ChunkId {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        ChunkId(IVec3::new(x, y, z))
    }

    fn on_add(mut world: bevy::ecs::world::DeferredWorld, ctx: bevy::ecs::component::HookContext) {
        let id = *world
            .entity(ctx.entity)
            .get::<ChunkId>()
            .expect("onadd of ChunkId");
        world
            .entity_mut(ctx.entity)
            .get_mut::<Transform>()
            .expect("ChunkId Requires Transform")
            .translation = id.to_vec3();
    }

    fn to_vec3(self) -> Vec3 {
        self.0.as_vec3() * CHUNK_SIZE as f32
    }
}

struct ChunkBlockIter {
    x: i32,
    y: i32,
    z: i32,
}

impl ChunkBlockIter {
    fn new() -> ChunkBlockIter {
        ChunkBlockIter { x: 0, y: 0, z: 0 }
    }
}

// #[allow(dead_code)]
// struct ChunkIter<'a>(ChunkBlockIter, &'a ChunkData);

// impl<'a> Iterator for ChunkIter<'a> {
//     type Item = BlockType;
//     fn next(&mut self) -> Option<Self::Item> {
//         let (x, y, z) = self.0.next()?;
//         Some(self.1.block(x, y, z))
//     }
// }

impl Iterator for ChunkBlockIter {
    type Item = (i32, i32, i32);
    fn next(&mut self) -> Option<Self::Item> {
        let out = if self.y >= CHUNK_SIZE {
            return None;
        } else {
            (self.x, self.y, self.z)
        };
        self.x += 1;
        if self.x >= CHUNK_SIZE {
            self.x -= CHUNK_SIZE;
            self.z += 1;
        }
        if self.z >= CHUNK_SIZE {
            self.z -= CHUNK_SIZE;
            self.y += 1
        }
        Some(out)
    }
}

fn spawn_world(
    mut commands: Commands,
    block_data: Res<BlockDescriptor>,
    asset_server: Res<AssetServer>,
    generator: Res<PhoxelGenerator<GeneratorDataType>>,
) {
    // map_descriptor.min_max_y();
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE * 100.).looking_at(Vec3::NEG_Y * 100., Vec3::Y),
    ));
    commands.spawn((Mesh3d(
        asset_server.add(Cuboid::from_size(Vec3::ONE * 2.).into()),
    ),));
    if MAP_SIZE == 0 {
        commands.spawn((
            MeshMaterial3d(block_data.material()),
            ChunkId::new(-1, 0, -1),
            generator.clone(),
        ));
        return;
    }

    for z in -MAP_SIZE..=MAP_SIZE {
        for x in -MAP_SIZE..=MAP_SIZE {
            commands.spawn((
                ChunkId::new(x, 0, z),
                // Transform::from_scale(Vec3::splat(0.5)),
                MeshMaterial3d(block_data.material()),
                Mesh3d(Default::default()),
                generator.clone(),
            ));
        }
    }
}

type GeneratorData = std::sync::Arc<RwLock<MapDescriptorInernal>>;

#[derive(Resource)]
pub struct BlockDescriptor {
    mesh: Handle<Mesh>,
    material: Handle<CustomMaterial>,
    pub terrain: Handle<Image>,
}

impl FromWorld for BlockDescriptor {
    fn from_world(world: &mut World) -> Self {
        let mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Cuboid::from_size(Vec3::ONE));
        let texture = world.resource::<AssetServer>().load("no_share/terrain.png");

        let mut material_with_override = CustomMaterial {
            base_color_texture: Some(texture.clone()),
            atlas_shape: UVec2::new(16, 16),
            ..Default::default()
        };

        let override_data = BlockOverride::default().top(1);
        material_with_override.set_override(BlockType::Grass, override_data);

        let override_data = BlockOverride::default()
            .back(1)
            .left(1)
            .right(1)
            .bottom(18)
            .top(18);
        material_with_override.set_override(BlockType::Furnuse, override_data);

        let material = world
            .resource_mut::<Assets<CustomMaterial>>()
            .add(material_with_override);
        BlockDescriptor {
            mesh,
            material,
            terrain: texture,
        }
    }
}

impl BlockDescriptor {
    pub fn mesh(&self) -> Handle<Mesh> {
        self.mesh.clone()
    }

    pub fn material(&self) -> Handle<CustomMaterial> {
        self.material.clone()
    }
}

#[derive(Clone, Resource, Deref, DerefMut)]
struct MapDescriptor(GeneratorData);

struct MapDescriptorInernal {
    noise: Fbm<SuperSimplex>,
}

impl MapDescriptorInernal {
    fn get_height(&self, x: i32, z: i32) -> i32 {
        let h = self.noise.get([x as f64, z as f64]) * GROUND_HIGHT as f64;
        GROUND_HIGHT + h as i32
    }

    #[allow(dead_code)]
    fn min_max_y(&self) {
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for x in -1000..1000 {
            for z in -1000..1000 {
                let n = self.noise.get([x as f64, z as f64]);
                min = n.min(min);
                max = n.max(max);
            }
        }
        for x in -1000..1000 {
            for z in -1000..1000 {
                let n = self.noise.get([x as f64 + 0.5, z as f64 - 0.5]);
                min = n.min(min);
                max = n.max(max);
            }
        }
        println!("min: {min}\nmax: {max}");
    }
}

impl FromWorld for MapDescriptor {
    fn from_world(_: &mut World) -> Self {
        let mut noise = Fbm::new(0);
        noise = noise.set_frequency(0.005);
        noise = noise.set_persistence(0.7);
        MapDescriptor::new(noise)
    }
}

impl MapDescriptor {
    fn new(noise: Fbm<SuperSimplex>) -> MapDescriptor {
        let mdi = MapDescriptorInernal { noise };
        mdi.min_max_y();
        MapDescriptor(GeneratorData::new(RwLock::new(mdi)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockType {
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
