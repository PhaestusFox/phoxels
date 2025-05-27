use bevy::{asset::RenderAssetUsages, render::mesh::Mesh};

use crate::core::BlockMeta;

use super::{ChunkData, utils::BlockIter};

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

pub async fn make_mesh(data: ChunkData) -> Mesh {
    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    for (x, y, z) in BlockIter::<{ super::CHUNK_SIZE.size() }>::new() {
        let block = data.block(x, y, z);
        if block == BlockMeta::EMPTY {
            continue;
        }
        let mut m_block = VertexSet::default();
        if data.block(x, y + 1, z).is_transparent() {
            m_block.add_face(&TOP_FACE);
        };
        if y == 0 || data.block(x, y - 1, z).is_transparent() {
            m_block.add_face(&BOTTOM_FACE);
        };
        if x == 0 || data.block(x - 1, y, z).is_transparent() {
            m_block.add_face(&LEFT_FACE);
        };
        if data.block(x + 1, y, z).is_transparent() {
            m_block.add_face(&RIGHT_FACE);
        };
        if z == 0 || data.block(x, y, z - 1).is_transparent() {
            m_block.add_face(&FRONT_FACE);
        };
        if data.block(x, y, z + 1).is_transparent() {
            m_block.add_face(&BACK_FACE);
        };

        indices.extend(m_block.indices.iter().map(|i| positions.len() as u32 + i));
        positions.extend(m_block.vertexs.iter().map(|p| {
            let p = p.to_pos();
            let x = p[0] + x as u32;
            let y = p[1] + y as u32;
            let z = p[2] + z as u32;
            x | y << 8 | z << 16 | (block.texture() as u32) << 24
        }));
    }
    mesh.insert_attribute(crate::simple_shader::BLOCK_DATA, positions);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    mesh
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
    fn to_pos(self) -> [u32; 3] {
        match self {
            Vertex::LeftBottomBack => [0, 0, 1],
            Vertex::RightBottomBack => [1, 0, 1],
            Vertex::RightTopBack => [1, 1, 1],
            Vertex::LeftTopBack => [0, 1, 1],
            Vertex::LeftBottomFront => [0, 0, 0],
            Vertex::RightBottomFront => [1, 0, 0],
            Vertex::RightTopFront => [1, 1, 0],
            Vertex::LeftTopFront => [0, 1, 0],
        }
    }
}
