use std::marker::PhantomData;

use bevy::{
    app::Update,
    ecs::{
        query::Added,
        removal_detection::RemovedComponents,
        system::{Query, ResMut},
    },
    log::warn,
    prelude::{App, Plugin, Resource},
    render::mesh::Mesh3d,
};

use crate::core::{Block, ChunkData};

#[derive(Default)]
pub struct PhoxelDiagnostics<T: Block>(PhantomData<T>);

impl<T: Block> Plugin for PhoxelDiagnostics<T> {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelCount>()
            .add_systems(Update, update_on_mesh::<T>);
    }
}

#[derive(Resource, Default)]
pub struct VoxelCount {
    pub loaded: usize,
    pub meshed: usize,
}

fn update_on_mesh<T: Block>(
    mut diagnostics: ResMut<VoxelCount>,
    mesh_added: Query<&ChunkData<T>, Added<Mesh3d>>,
    all: Query<&ChunkData<T>>,
    mut removed: RemovedComponents<Mesh3d>,
) {
    for chunk in mesh_added.iter() {
        diagnostics.meshed += chunk.voxel_count();
    }

    for e in removed.read() {
        if let Ok(chunk) = all.get(e) {
            diagnostics.meshed -= chunk.voxel_count();
            #[cfg(feature = "log")]
            warn!(
                "Chunk({:?}) had its mesh removed, but was not its self removed this is bad practice",
                e
            );
        }
    }
}
