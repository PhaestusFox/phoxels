use phoxels::prelude::*;

use criterion::{Criterion, criterion_group, criterion_main};

fn get_empty() -> ChunkData {
    ChunkData::empty()
}

fn get_solid(block: impl Block) -> ChunkData {
    ChunkData::solid(block)
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("gen_empty", |b| {
        b.iter_batched(
            get_empty,
            phoxels::dev::make_mesh,
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("gen_sold", |b| {
        b.iter_batched(
            || get_solid(BlockType::Stone),
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
