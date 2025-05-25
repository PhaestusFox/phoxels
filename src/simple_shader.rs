use bevy::{
    prelude::*,
    render::{
        mesh::{MeshVertexAttribute, VertexFormat},
        render_resource::AsBindGroup,
    },
};

const ATTRIBUTE_BLEND_COLOR: MeshVertexAttribute =
    MeshVertexAttribute::new("BlendColor", 988540917, VertexFormat::Float32x4);

pub struct VoxelShaderPlugin;

impl Plugin for VoxelShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<VoxelMaterial>::default());
    }
}

#[derive(Default, Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct VoxelMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    pub base_color_texture: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
}

impl Material for VoxelMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/voxel.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    // fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
    //     "shaders/voxel.wgsl".into()
    // }

    // fn prepass_vertex_shader() -> bevy::render::render_resource::ShaderRef {
    //     bevy::render::render_resource::ShaderRef::Default
    // }

    // fn specialize(
    //     _pipeline: &bevy::pbr::MaterialPipeline<Self>,
    //     descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
    //     layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
    //     _key: bevy::pbr::MaterialPipelineKey<Self>,
    // ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
    //     let vertex_layout = layout.0.get_layout(&[
    //         Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
    //         Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
    //         // ATTRIBUTE_BLEND_COLOR.at_shader_location(1),
    //     ])?;
    //     descriptor.vertex.buffers = vec![vertex_layout];
    //     Ok(())
    // }
}
