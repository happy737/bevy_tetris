use bevy::{
    asset::RenderAssetUsages, pbr::{MaterialPipeline, MaterialPipelineKey}, prelude::*, render::{
        mesh::{MeshVertexBufferLayoutRef, PrimitiveTopology}, 
        render_resource::{AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError}
    }
};

const SHADER_ASSET_PATH: &str = "shaders/line_material.wgsl";

pub struct LineStuffPlugin;

impl Plugin for LineStuffPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<LineMaterial>::default());
    }
}

/// A mesh, made entirely of lines instead of triangles. Two consecutive indices define a single line. <br>
/// The material responsible for lines is [LineMaterial]. <br>
/// To use this mesh, make sure to add the [LineStuffPlugin] plugin. 
#[derive(Debug, Clone)]
pub struct LineListIndex {
    pub points: Vec<Vec3>,
    pub indices: Vec<u32>,
}

impl LineListIndex {
    /// Returns an object shaped like a cube. 
    pub fn cube() -> Self {
        Self {
            points: vec![
                Vec3::new(-1.0, -1.0, -1.0),
                Vec3::new(-1.0, -1.0, 1.0),
                Vec3::new(1.0, -1.0, 1.0),
                Vec3::new(1.0, -1.0, -1.0),

                Vec3::new(-1.0, 1.0, -1.0),
                Vec3::new(-1.0, 1.0, 1.0),
                Vec3::new(1.0, 1.0, 1.0),
                Vec3::new(1.0, 1.0, -1.0),
            ], 
            indices: vec![0,1, 1,2, 2,3, 3,0,  0,4, 1,5, 2,6, 3,7,  4,5, 5,6, 6,7, 7,4,],
        }
    }
}

impl From<LineListIndex> for Mesh {
    fn from(value: LineListIndex) -> Self {
        Mesh::new(
            PrimitiveTopology::LineList,
            RenderAssetUsages::RENDER_WORLD
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, value.points)
        .with_inserted_indices(bevy::render::mesh::Indices::U32(value.indices))
    }
}

/// The material for a line object. Contains simply a color. 
#[derive(Asset, TypePath, Default, AsBindGroup, Debug, Clone)]
pub struct LineMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<LineMaterial>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the important part to tell bevy to render this material as a line between vertices
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}