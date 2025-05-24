use std::sync::RwLock;

use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
    tasks::Task,
};
use indexmap::IndexMap;
use noise::{Fbm, MultiFractal, NoiseFn, SuperSimplex};

use crate::{diganostics::VoxelCount, player::Player};

const CHUNK_SIZE: i32 = 16;
const CHUNK_ARIA: i32 = CHUNK_SIZE * CHUNK_SIZE;
const CHUNK_VOLUME: i32 = CHUNK_ARIA * CHUNK_SIZE;
const GROUND_HIGHT: i32 = 8;

const MAP_SIZE: i32 = 10;

pub fn plugin(app: &mut App) {
    app.init_resource::<BlockDescriptor>()
        .init_resource::<MapDescriptor>()
        .init_resource::<MapData>();
    app.add_systems(Startup, spawn_world)
        .add_systems(Last, start_generating_chunks)
        .add_systems(First, (extract_data, populate_chunks).chain());
}

#[derive(Component, PartialEq, Eq, Hash, Clone, Copy, Deref)]
#[component(immutable, on_insert = ChunkId::on_add)]
#[require(Transform, Visibility)]
struct ChunkId(IVec3);

impl std::fmt::Display for ChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

impl ChunkId {
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
        let mut map = world.resource_mut::<MapData>();
        map.to_gen_chunks.insert(id, ctx.entity);
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

#[allow(dead_code)]
struct ChunkIter<'a>(ChunkBlockIter, &'a ChunkData);

impl<'a> Iterator for ChunkIter<'a> {
    type Item = BlockType;
    fn next(&mut self) -> Option<Self::Item> {
        let (x, y, z) = self.0.next()?;
        Some(self.1.get_block(x, y, z))
    }
}

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

fn spawn_world(mut commands: Commands) {
    // map_descriptor.min_max_y();
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE * 100.).looking_at(Vec3::NEG_Y * 100., Vec3::Y),
    ));
    for z in -MAP_SIZE..=MAP_SIZE {
        for x in -MAP_SIZE..=MAP_SIZE {
            commands.spawn(ChunkId(IVec3::new(x, 0, z)));
        }
    }
}

type GeneratorData = std::sync::Arc<RwLock<MapDescriptorInernal>>;

#[derive(Resource)]
struct BlockDescriptor {
    blocks: Vec<Handle<StandardMaterial>>,
    mesh: Handle<Mesh>,
}

impl FromWorld for BlockDescriptor {
    fn from_world(world: &mut World) -> Self {
        let mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Cuboid::from_size(Vec3::ONE));
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let blocks = vec![
            materials.add(StandardMaterial {
                base_color: Color::linear_rgb(0., 0.8, 0.),
                ..Default::default()
            }),
            materials.add(StandardMaterial {
                base_color: Color::linear_rgb(0.88, 0.57, 0.39),
                ..Default::default()
            }),
            materials.add(StandardMaterial {
                base_color: Color::linear_rgb(0.4, 0.4, 0.4),
                ..Default::default()
            }),
        ];
        BlockDescriptor { blocks, mesh }
    }
}

impl BlockDescriptor {
    fn mesh(&self) -> Handle<Mesh> {
        self.mesh.clone()
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
    fn from_world(world: &mut World) -> Self {
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

#[derive(Resource, Default)]
struct MapData {
    loaded_chunks: HashMap<ChunkId, ChunkData>,
    to_populate: HashSet<Entity>,
    loading_chunks: HashMap<ChunkId, (Entity, Task<ChunkData>)>,
    old_loading: HashMap<ChunkId, (Entity, Task<ChunkData>)>,
    to_gen_chunks: IndexMap<ChunkId, Entity>,
}

struct ChunkData {
    blocks: [BlockType; CHUNK_VOLUME as usize],
}

impl ChunkData {
    fn new() -> ChunkData {
        ChunkData {
            blocks: [BlockType::Air; CHUNK_VOLUME as usize],
        }
    }

    async fn generate(id: ChunkId, map_descriptor: MapDescriptor) -> ChunkData {
        let mut chunk = ChunkData::new();
        let map_descriptor = map_descriptor.read().unwrap();
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let h = map_descriptor.get_height(x + id.x * CHUNK_SIZE, z + id.z * CHUNK_SIZE);
                for y in (id.y * CHUNK_SIZE)..(id.y + 1) * CHUNK_SIZE {
                    if y > h {
                        break;
                    }
                    let block = if y == h {
                        BlockType::Grass
                    } else if y + 3 > h {
                        BlockType::Dirt
                    } else {
                        BlockType::Stone
                    };
                    chunk.set_block(x, y - id.y * CHUNK_SIZE, z, block);
                }
            }
        }
        chunk
    }

    fn iter_block(&self) -> ChunkIter<'_> {
        ChunkIter(ChunkBlockIter::new(), self)
    }

    fn get_block(&self, x: i32, y: i32, z: i32) -> BlockType {
        assert!(x >= 0);
        assert!(y >= 0);
        assert!(z >= 0);
        assert!(x < CHUNK_SIZE);
        assert!(y < CHUNK_SIZE);
        assert!(z < CHUNK_SIZE);

        self.blocks[(y * CHUNK_ARIA + z * CHUNK_SIZE + x) as usize]
    }

    fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockType) {
        assert!(x >= 0);
        assert!(y >= 0);
        assert!(z >= 0);
        assert!(x < CHUNK_SIZE);
        assert!(y < CHUNK_SIZE);
        assert!(z < CHUNK_SIZE);

        self.blocks[(y * CHUNK_ARIA + z * CHUNK_SIZE + x) as usize] = block;
    }
}

#[derive(Debug, Clone, Copy)]
enum BlockType {
    Air,
    Grass,
    Dirt,
    Stone,
}

fn start_generating_chunks(
    mut map: ResMut<MapData>,
    pos: Single<&Transform, With<Player>>,
    map_desctiptor: Res<MapDescriptor>,
) {
    let loading_num = map.loading_chunks.len();
    let pool = bevy::tasks::AsyncComputeTaskPool::get();
    let max_loading = pool.thread_num();
    let to_gen = map.to_gen_chunks.len().min(max_loading - loading_num);
    if to_gen == 0 {
        return;
    }
    map.to_gen_chunks.sort_by(|c0, _, c1, _| {
        pos.translation
            .distance_squared(c1.to_vec3())
            .partial_cmp(&pos.translation.distance_squared(c0.to_vec3()))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for _ in 0..to_gen {
        let (next, target) = map.to_gen_chunks.pop().expect("to load len > 0");
        let task = pool.spawn(ChunkData::generate(next, map_desctiptor.clone()));
        map.loading_chunks.insert(next, (target, task));
    }
}

fn extract_data(mut map: ResMut<MapData>) {
    let MapData {
        ref mut old_loading,
        ref mut loading_chunks,
        ref mut loaded_chunks,
        ref mut to_populate,
        ..
    } = *map;
    std::mem::swap(old_loading, loading_chunks);
    for (id, (entity, task)) in old_loading.drain() {
        if !task.is_finished() {
            loading_chunks.insert(id, (entity, task));
            continue;
        };
        let data = bevy::tasks::block_on(task);
        loaded_chunks.insert(id, data);
        to_populate.insert(entity);
    }
}

fn populate_chunks(
    mut map: ResMut<MapData>,
    chunks: Query<&ChunkId>,
    mut commands: Commands,
    block_data: Res<BlockDescriptor>,
    mut voxel_count: ResMut<VoxelCount>,
) {
    let MapData {
        ref mut loaded_chunks,
        ref mut to_populate,
        ..
    } = *map;

    for entity in to_populate.drain() {
        let Ok(id) = chunks.get(entity) else {
            error!("Entity{entity} dose not have a ChunkId");
            continue;
        };

        let chunk = loaded_chunks
            .get(id)
            .expect("chunk to load before populate");

        commands.entity(entity).with_children(|p| {
            for (x, y, z) in ChunkBlockIter::new() {
                let block = chunk.get_block(x, y, z);
                let material = match block {
                    BlockType::Air => continue,
                    BlockType::Grass => block_data.blocks[0].clone(),
                    BlockType::Dirt => block_data.blocks[1].clone(),
                    BlockType::Stone => block_data.blocks[2].clone(),
                };
                p.spawn((
                    Transform::from_translation(IVec3::new(x, y, z).as_vec3()),
                    Mesh3d(block_data.mesh()),
                    MeshMaterial3d(material),
                ));
                voxel_count.inc();
            }
        });
    }
}
