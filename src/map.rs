use std::sync::{Arc, RwLock};

use bevy::{
    asset::RenderAssetUsages,
    platform::collections::{HashMap, HashSet},
    prelude::*,
    tasks::Task,
};
use indexmap::IndexMap;
use noise::{Fbm, MultiFractal, NoiseFn, SuperSimplex};

use crate::{
    diganostics::VoxelCount,
    player::Player,
    simple_shader::{BLOCK_ID, VoxelMaterial as CustomMaterial},
};

const CHUNK_SIZE: i32 = 16;
const CHUNK_ARIA: i32 = CHUNK_SIZE * CHUNK_SIZE;
const CHUNK_VOLUME: i32 = CHUNK_ARIA * CHUNK_SIZE;
const GROUND_HIGHT: i32 = 8;

const MAP_SIZE: i32 = 1;
const TASK_MULT: usize = 10;

pub fn plugin(app: &mut App) {
    app.init_resource::<BlockDescriptor>()
        .init_resource::<MapDescriptor>()
        .init_resource::<MapData>();
    app.add_systems(Startup, spawn_world)
        .add_systems(Last, start_generating_chunks)
        .add_systems(First, (extract_chunkdata, populate_chunks).chain());
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
        map.to_gen_data.insert(id, ctx.entity);
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
        Some(self.1.block(x, y, z))
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
    material: Handle<CustomMaterial>,
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
        let texture = world
            .resource::<AssetServer>()
            .load("no_share/mc_textures.png");
        let material = world
            .resource_mut::<Assets<CustomMaterial>>()
            .add(CustomMaterial {
                base_color_texture: Some(texture),
                ..Default::default()
            });
        BlockDescriptor {
            blocks,
            mesh,
            material,
        }
    }
}

impl BlockDescriptor {
    fn mesh(&self) -> Handle<Mesh> {
        self.mesh.clone()
    }

    fn material(&self) -> Handle<CustomMaterial> {
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

#[derive(Resource, Default)]
struct MapData {
    loaded_chunks: HashMap<ChunkId, ChunkData>,
    to_populate: HashMap<Entity, Handle<Mesh>>,
    generating_chunks: HashMap<ChunkId, (Entity, Task<ChunkData>)>,
    old_generating: HashMap<ChunkId, (Entity, Task<ChunkData>)>,
    meshing_chunks: HashMap<ChunkId, (Entity, Task<Mesh>)>,
    old_meshing: HashMap<ChunkId, (Entity, Task<Mesh>)>,
    to_gen_data: IndexMap<ChunkId, Entity>,
    to_gen_mesh: IndexMap<ChunkId, Entity>,
}

#[derive(Clone)]
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

    #[inline(always)]
    fn block(&self, x: i32, y: i32, z: i32) -> BlockType {
        assert!(x >= 0);
        assert!(y >= 0);
        assert!(z >= 0);
        assert!(x < CHUNK_SIZE);
        assert!(y < CHUNK_SIZE);
        assert!(z < CHUNK_SIZE);

        self.blocks[(y * CHUNK_ARIA + z * CHUNK_SIZE + x) as usize]
    }

    fn get_block(&self, x: i32, y: i32, z: i32) -> Option<BlockType> {
        if x < 0 || y < 0 || z < 0 || x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return None;
        }
        Some(self.block(x, y, z))
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
    // Back face
    const BACK_FACE: [Vertex; 4] = [
        Vertex::LeftBottomBack,  // left bottom back
        Vertex::RightBottomBack, // right bottom back
        Vertex::RightTopBack,    // right top back
        Vertex::LeftTopBack,     // left top back
    ];

    // Front face
    const FRONT_FACE: [Vertex; 4] = [
        Vertex::LeftBottomFront,  // left bottom front
        Vertex::LeftTopFront,     // left top front
        Vertex::RightTopFront,    // right top front
        Vertex::RightBottomFront, // right bottom front
    ];

    // Left face
    const LEFT_FACE: [Vertex; 4] = [
        Vertex::LeftBottomBack,  // left bottom back
        Vertex::LeftTopBack,     // left top back
        Vertex::LeftTopFront,    // left top front
        Vertex::LeftBottomFront, // left bottom front
    ];

    // Right face
    const RIGHT_FACE: [Vertex; 4] = [
        Vertex::RightBottomBack,  // right bottom back
        Vertex::RightBottomFront, // right bottom front
        Vertex::RightTopFront,    // right top front
        Vertex::RightTopBack,     // right top back
    ];

    // Bottom face
    const BOTTOM_FACE: [Vertex; 4] = [
        Vertex::LeftBottomBack,   // left bottom back
        Vertex::LeftBottomFront,  // left bottom front
        Vertex::RightBottomFront, // right bottom front
        Vertex::RightBottomBack,  // right bottom back
    ];

    // Top face
    const TOP_FACE: [Vertex; 4] = [
        Vertex::LeftTopBack,   // left top back
        Vertex::RightTopBack,  // right top back
        Vertex::RightTopFront, // right top front
        Vertex::LeftTopFront,  // left top front
    ];

    const INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

    const NORMALS: [[f32; 3]; 8] = [
        [-0.57735027, -0.57735027, 0.57735027],  // vertex 0
        [0.57735027, -0.57735027, 0.57735027],   // vertex 1
        [0.57735027, 0.57735027, 0.57735027],    // vertex 2
        [-0.57735027, 0.57735027, 0.57735027],   // vertex 3
        [-0.57735027, -0.57735027, -0.57735027], // vertex 4
        [0.57735027, -0.57735027, -0.57735027],  // vertex 5
        [0.57735027, 0.57735027, -0.57735027],   // vertex 6
        [-0.57735027, 0.57735027, -0.57735027],  // vertex 7
    ];

    async fn make_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(
            bevy::render::mesh::PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();
        for (x, y, z) in ChunkBlockIter::new() {
            let block = self.block(x, y, z);
            if block == BlockType::Air {
                continue;
            }
            let mut m_block = VertexSet::default();
            if let Some(BlockType::Air) = self.get_block(x, y + 1, z).or(Some(BlockType::Air)) {
                m_block.add_face(&ChunkData::TOP_FACE);
            };
            if let Some(BlockType::Air) = self.get_block(x, y - 1, z).or(Some(BlockType::Air)) {
                m_block.add_face(&ChunkData::BOTTOM_FACE);
            };
            if let Some(BlockType::Air) = self.get_block(x - 1, y, z).or(Some(BlockType::Air)) {
                m_block.add_face(&ChunkData::LEFT_FACE);
            };
            if let Some(BlockType::Air) = self.get_block(x + 1, y, z).or(Some(BlockType::Air)) {
                m_block.add_face(&ChunkData::RIGHT_FACE);
            };
            if let Some(BlockType::Air) = self.get_block(x, y, z - 1).or(Some(BlockType::Air)) {
                m_block.add_face(&ChunkData::FRONT_FACE);
            };
            if let Some(BlockType::Air) = self.get_block(x, y, z + 1).or(Some(BlockType::Air)) {
                m_block.add_face(&ChunkData::BACK_FACE);
            };

            indices.extend(m_block.indices.iter().map(|i| positions.len() as u32 + i));
            uvs.extend_from_slice(&vec![block as u32; m_block.vertexs.len()]);
            positions.extend(m_block.vertexs.iter().map(|p| {
                let p = p.to_pos();
                [p[0] + x as f32, p[1] + y as f32, p[2] + z as f32]
            }));
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(BLOCK_ID, uvs);
        mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
        mesh
    }

    fn count(&self) -> u32 {
        let mut r = 0;
        for block in self.iter_block() {
            if block != BlockType::Air {
                r += 1;
            }
        }
        r
    }
}

#[derive(Default, Debug)]
struct VertexSet {
    left_bottom_back: Option<u32>,
    right_bottom_back: Option<u32>,
    right_top_back: Option<u32>,
    left_top_back: Option<u32>,
    left_bottom_front: Option<u32>,
    right_bottom_front: Option<u32>,
    right_top_front: Option<u32>,
    left_top_front: Option<u32>,
    vertexs: Vec<Vertex>,
    indices: Vec<u32>,
}

impl VertexSet {
    fn add_face(&mut self, face: &[Vertex; 4]) {
        for i in [0, 1, 2, 0, 2, 3] {
            let v = face[i];
            let i = self.index(v);
            self.indices.push(i);
        }
    }

    fn index(&mut self, vertex: Vertex) -> u32 {
        let update = || {
            let i = self.vertexs.len();
            self.vertexs.push(vertex);
            i as u32
        };
        *match vertex {
            Vertex::LeftBottomBack => self.left_bottom_back.get_or_insert_with(update),
            Vertex::RightBottomBack => self.right_bottom_back.get_or_insert_with(update),
            Vertex::RightTopBack => self.right_top_back.get_or_insert_with(update),
            Vertex::LeftTopBack => self.left_top_back.get_or_insert_with(update),
            Vertex::LeftBottomFront => self.left_bottom_front.get_or_insert_with(update),
            Vertex::RightBottomFront => self.right_bottom_front.get_or_insert_with(update),
            Vertex::RightTopFront => self.right_top_front.get_or_insert_with(update),
            Vertex::LeftTopFront => self.left_top_front.get_or_insert_with(update),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Vertex {
    LeftBottomBack,   //[0., 0., 1.],
    RightBottomBack,  //[1., 0., 1.],
    RightTopBack,     //[1., 1., 1.],
    LeftTopBack,      //[0., 1., 1.],
    LeftBottomFront,  //[0., 0., 0.],
    RightBottomFront, //[1., 0., 0.],
    RightTopFront,    //[1., 1., 0.],
    LeftTopFront,     //[0., 1., 0.],
}

impl Vertex {
    fn to_pos(self) -> [f32; 3] {
        match self {
            Vertex::LeftBottomBack => [0., 0., 1.],
            Vertex::RightBottomBack => [1., 0., 1.],
            Vertex::RightTopBack => [1., 1., 1.],
            Vertex::LeftTopBack => [0., 1., 1.],
            Vertex::LeftBottomFront => [0., 0., 0.],
            Vertex::RightBottomFront => [1., 0., 0.],
            Vertex::RightTopFront => [1., 1., 0.],
            Vertex::LeftTopFront => [0., 1., 0.],
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockType {
    Air,
    Stone,
    Cobblestone,
    Dirt,
    Grass,
}

fn start_generating_chunks(
    mut map: ResMut<MapData>,
    pos: Single<&Transform, With<Player>>,
    map_desctiptor: Res<MapDescriptor>,
) {
    let loading_num = map.generating_chunks.len();
    let pool = bevy::tasks::AsyncComputeTaskPool::get();
    let max_loading = pool.thread_num() * TASK_MULT;
    let to_gen = map.to_gen_data.len().min(max_loading - loading_num);
    if to_gen != 0 {
        map.to_gen_data.sort_by(|c0, _, c1, _| {
            pos.translation
                .distance_squared(c1.to_vec3())
                .partial_cmp(&pos.translation.distance_squared(c0.to_vec3()))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for _ in 0..to_gen {
            let (next, target) = map.to_gen_data.pop().expect("to load len > 0");
            let task = pool.spawn(ChunkData::generate(next, map_desctiptor.clone()));
            map.generating_chunks.insert(next, (target, task));
        }
    }

    let meshing_num = map.meshing_chunks.len();
    let to_gen = map.to_gen_mesh.len().min(max_loading - meshing_num);
    if to_gen != 0 {
        map.to_gen_mesh.sort_by(|c0, _, c1, _| {
            pos.translation
                .distance_squared(c1.to_vec3())
                .partial_cmp(&pos.translation.distance_squared(c0.to_vec3()))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for _ in 0..to_gen {
            let (next, target) = map.to_gen_mesh.pop().expect("to load len > 0");
            let Some(data) = map.loaded_chunks.get(&next) else {
                error!("Failed to find data for {next}");
                continue;
            };
            let task = pool.spawn(data.clone().make_mesh());
            map.meshing_chunks.insert(next, (target, task));
        }
    }
}

fn extract_chunkdata(mut map: ResMut<MapData>, mut mesh_asset: ResMut<Assets<Mesh>>) {
    let MapData {
        old_generating: ref mut old_loading,
        generating_chunks: ref mut loading_chunks,
        ref mut loaded_chunks,
        ref mut to_populate,
        ref mut to_gen_mesh,
        ref mut old_meshing,
        ref mut meshing_chunks,
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
        to_gen_mesh.insert(id, entity);
    }

    std::mem::swap(old_meshing, meshing_chunks);
    for (id, (entity, task)) in old_meshing.drain() {
        if !task.is_finished() {
            meshing_chunks.insert(id, (entity, task));
            continue;
        };
        let data = bevy::tasks::block_on(task);
        to_populate.insert(entity, mesh_asset.add(data));
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

    for (entity, mesh) in to_populate.drain() {
        let Ok(id) = chunks.get(entity) else {
            error!("Entity{entity} dose not have a ChunkId");
            continue;
        };

        let chunk = loaded_chunks
            .get(id)
            .expect("chunk to load before populate");
        voxel_count.add(chunk.count());

        commands
            .entity(entity)
            .insert((Mesh3d(mesh), MeshMaterial3d(block_data.material())));
    }
}

// fn modidy_pipline(
//     renderGr
// ) {

// }
