use criterion::{Criterion, criterion_group, criterion_main};
use phoxels::core::CHUNK_SIZE;
use phoxels::prelude::*;
use rand::{Rng, SeedableRng, rngs::StdRng};
fn get_empty() -> ChunkData {
    ChunkData::empty()
}

fn get_solid() -> ChunkData {
    ChunkData::solid(BlockType::Stone)
}

fn get_mostly_empty() -> ChunkData {
    let mut noise = StdRng::from_seed([0; 32]);
    let mut chunk = ChunkData::empty();
    for x in 0..CHUNK_SIZE.size() {
        for y in 0..CHUNK_SIZE.size() {
            for z in 0..CHUNK_SIZE.size() {
                if noise.gen_bool(0.1) {
                    chunk.set_block(x, y, z, BlockType::Stone);
                }
            }
        }
    }
    chunk
}

fn get_mostly_full() -> ChunkData {
    let mut noise = StdRng::from_seed([0; 32]);
    let mut chunk = ChunkData::solid(BlockType::Stone);
    for x in 0..CHUNK_SIZE.size() {
        for y in 0..CHUNK_SIZE.size() {
            for z in 0..CHUNK_SIZE.size() {
                if noise.gen_bool(0.1) {
                    chunk.set_block(x, y, z, BlockType::Air);
                }
            }
        }
    }
    chunk
}

fn get_worst_case() -> ChunkData {
    let mut chunk = ChunkData::empty();
    for x in 0..CHUNK_SIZE.size() {
        for y in 0..CHUNK_SIZE.size() {
            for z in 0..CHUNK_SIZE.size() {
                if (x + y + z) % 2 == 0 {
                    chunk.set_block(x, y, z, BlockType::Stone);
                } else {
                    chunk.set_block(x, y, z, BlockType::Air);
                }
            }
        }
    }
    chunk
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("gen_empty", |b| {
        b.iter_batched(
            get_empty,
            phoxels::dev::make_mesh,
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("gen_solid", |b| {
        b.iter_batched(
            get_solid,
            phoxels::dev::make_mesh,
            criterion::BatchSize::SmallInput,
        )
    });
    c.bench_function("gen_10_full", |b| {
        b.iter_batched(
            get_mostly_full,
            phoxels::dev::make_mesh,
            criterion::BatchSize::SmallInput,
        )
    });
    c.bench_function("gen_10_empty", |b| {
        b.iter_batched(
            get_mostly_empty,
            phoxels::dev::make_mesh,
            criterion::BatchSize::SmallInput,
        )
    });
    c.bench_function("worst case", |b| {
        b.iter_batched(
            get_worst_case,
            phoxels::dev::make_mesh,
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

enum BlockType {
    Air,
    Stone,
    Dirt,
    Grass,
}

impl Block for BlockType {
    fn id(&self) -> u8 {
        match self {
            BlockType::Air => 0,
            BlockType::Stone => 1,
            BlockType::Dirt => 2,
            BlockType::Grass => 3,
        }
    }

    fn is_solid(&self) -> bool {
        match self {
            BlockType::Air => false,
            _ => true,
        }
    }

    fn is_transparent(&self) -> bool {
        match self {
            BlockType::Air => true,
            _ => false,
        }
    }
}
