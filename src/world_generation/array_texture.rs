use bevy::{
    pbr::{MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline},
    prelude::*,
    render::{
        mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef, VertexFormat},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
            VertexAttribute,
        },
    },
};

pub const ATTRIBUTE_TEXTURE_ID: MeshVertexAttribute =
    MeshVertexAttribute::new("TextureId", 988543481, VertexFormat::Uint32);

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ArrayTextureMaterial {
    #[texture(100, dimension = "2d_array")]
    #[sampler(101)]
    pub array_texture: Handle<Image>,
}

impl MaterialExtension for ArrayTextureMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/array_texture.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/array_texture.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(index) = layout
            .0
            .attribute_ids()
            .iter()
            .position(|id| *id == ATTRIBUTE_TEXTURE_ID.id)
        {
            let layout_attribute = &layout.0.layout().attributes[index];
            descriptor.vertex.buffers[0]
                .attributes
                .push(VertexAttribute {
                    format: layout_attribute.format,
                    offset: layout_attribute.offset,
                    shader_location: 8,
                });
        }
        Ok(())
    }
}
