use bevy::math::{UVec3, Vec3};
use bevy::render::primitives::Aabb;
use bevy::{asset::RenderAssetUsages, render::mesh::Mesh};

use super::{CHUNK_SIZE, ChunkData};
use crate::core::{Block, BlockMeta};
use crate::utils::DynBlockIter;

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

pub fn make_mesh<T: Block>(data: ChunkData<T>) -> (Mesh, Aabb) {
    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    let mut min = data.size;
    let mut max = UVec3::ZERO;
    let mut positions = Vec::new();

    #[cfg(feature = "standerd_position")]
    let mut positions_old = Vec::new();
    let mut indices = Vec::new();
    let mut checked = bevy::platform::collections::HashMap::new();
    // let UVec3 { x, y, z } = data.size;
    for (x, y, z) in DynBlockIter::new(data.size) {
        let block = data.block_meta(x, y, z);
        if block == BlockMeta::EMPTY {
            continue;
        }
        let mut current: Face = checked.remove(&UVec3::new(x, y, z)).unwrap_or_default();
        if current.all() {
            continue; // All block already added
        }
        let mut m_block = VertexSet::default();
        let mut long_x = 0;
        let mut long_y: u32 = 0;
        let mut long_z: u32 = 0;
        let block = data.texture(x, y, z);
        if !current.top() {
            if data.block_meta(x, y + 1, z).is_transparent() {
                let mut x_run = 1;
                for x in (x + 1)..data.size.x {
                    if data.texture(x, y, z) != block {
                        break;
                    }
                    if !data.block_meta(x, y + 1, z).is_transparent() {
                        break;
                    }
                    let other = checked.entry(UVec3::new(x, y, z)).or_default();
                    if other.top() {
                        break; // Already checked
                    }
                    x_run += 1;
                    other.set_top();
                }
                let mut z_run = 1;
                'z_loop: for z in (z + 1)..data.size.z {
                    for x in x..(x + x_run) {
                        if data.texture(x, y, z) != block {
                            break 'z_loop;
                        }
                        if !data.block_meta(x, y + 1, z).is_transparent() {
                            break 'z_loop;
                        }
                        if checked.get(&UVec3::new(x, y, z)).is_some_and(|f| f.top()) {
                            break 'z_loop; // Already checked
                        }
                    }
                    for x in x..(x + x_run) {
                        checked
                            .entry(UVec3::new(x, y, z))
                            .or_insert_with(Face::from_top)
                            .set_top();
                    }
                    z_run += 1;
                }
                if x_run > 1 || z_run > 1 {
                    long_x = long_x.max(x_run);
                    // long_y = long_y.max(y_run);
                    long_z = long_z.max(z_run as u32);
                    m_block.add_run(&TOP_FACE, x_run as u8, 1, z_run);
                } else {
                    m_block.add_face(&TOP_FACE);
                }
            };
            current.set_top();
        }
        if !current.bottom() {
            if y == 0 || data.block_meta(x, y - 1, z).is_transparent() {
                let mut x_run = 1;
                for x in (x + 1)..data.size.x {
                    if data.texture(x, y, z) != block {
                        break;
                    }
                    if y != 0 && !data.block_meta(x, y - 1, z).is_transparent() {
                        break;
                    }
                    let other = checked.entry(UVec3::new(x, y, z)).or_default();
                    if other.bottom() {
                        break; // Already checked
                    }
                    x_run += 1;
                    other.set_bottom();
                }
                let mut z_run = 1;
                'z_look: for z in (z + 1)..data.size.z {
                    for x in x..(x + x_run) {
                        if data.texture(x, y, z) != block {
                            break 'z_look;
                        }
                        if y != 0 && !data.block_meta(x, y - 1, z).is_transparent() {
                            break 'z_look;
                        }
                        if checked
                            .get(&UVec3::new(x, y, z))
                            .is_some_and(|f| f.bottom())
                        {
                            break 'z_look; // Already checked
                        }
                    }
                    for x in x..(x + x_run) {
                        checked
                            .entry(UVec3::new(x, y, z))
                            .or_insert_with(Face::from_bottom)
                            .set_bottom();
                    }
                    z_run += 1;
                }
                if x_run > 1 || z_run > 1 {
                    long_x = long_x.max(x_run);
                    // long_y = long_y.max(y_run);
                    long_z = long_z.max(z_run as u32);
                    m_block.add_run(&BOTTOM_FACE, x_run as u8, 1, z_run);
                } else {
                    m_block.add_face(&BOTTOM_FACE);
                }
            };
            current.set_bottom();
        }
        if !current.left() {
            if x == 0 || data.block_meta(x - 1, y, z).is_transparent() {
                let mut z_run = 1;
                for nz in (z + 1)..data.size.z {
                    if data.texture(x, y, nz) != block {
                        break;
                    }
                    if x != 0 && !data.block_meta(x - 1, y, nz).is_transparent() {
                        break;
                    }
                    let other = checked.entry(UVec3::new(x, y, nz)).or_default();
                    if other.left() {
                        break; // Already checked
                    }
                    z_run += 1;
                    other.set_left();
                }
                let mut y_run = 1;
                'y_look: for ny in (y + 1)..data.size.y {
                    for nz in z..(z + z_run) {
                        if data.texture(x, ny, nz) != block {
                            break 'y_look;
                        }
                        if x != 0 && !data.block_meta(x - 1, ny, nz).is_transparent() {
                            break 'y_look;
                        }
                        if checked
                            .get(&UVec3::new(x, ny, nz))
                            .is_some_and(|f| f.left())
                        {
                            break 'y_look; // Already checked
                        }
                    }
                    for nz in z..(z + z_run) {
                        checked
                            .entry(UVec3::new(x, ny, nz))
                            .or_insert_with(Face::from_left)
                            .set_left();
                    }
                    y_run += 1;
                }
                if y_run > 1 || z_run > 1 {
                    // long_x = long_x.max(x_run);
                    long_y = long_y.max(y_run as u32);
                    long_z = long_z.max(z_run);
                    m_block.add_run(&LEFT_FACE, 1, y_run, z_run as u8);
                } else {
                    m_block.add_face(&LEFT_FACE);
                }
            };
            current.set_left();
        }
        if !current.right() {
            if data.block_meta(x + 1, y, z).is_transparent() {
                let mut z_run = 1;
                for nz in (z + 1)..data.size.z {
                    if data.texture(x, y, nz) != block {
                        break;
                    }
                    if !data.block_meta(x + 1, y, nz).is_transparent() {
                        break;
                    }
                    let other = checked.entry(UVec3::new(x, y, nz)).or_default();
                    if other.right() {
                        break; // Already checked
                    }
                    z_run += 1;
                    other.set_right();
                }
                let mut y_run = 1;
                'y_look: for ny in (y + 1)..data.size.y {
                    for nz in z..(z + z_run) {
                        if data.texture(x, ny, nz) != block {
                            break 'y_look;
                        }
                        if !data.block_meta(x + 1, ny, nz).is_transparent() {
                            break 'y_look;
                        }
                        if checked
                            .get(&UVec3::new(x, ny, nz))
                            .is_some_and(|f| f.right())
                        {
                            break 'y_look; // Already checked
                        }
                    }
                    for nz in z..(z + z_run) {
                        checked
                            .entry(UVec3::new(x, ny, nz))
                            .or_insert_with(Face::from_right)
                            .set_right();
                    }
                    y_run += 1;
                }
                if y_run > 1 || z_run > 1 {
                    // long_x = long_x.max(x_run);
                    long_y = long_y.max(y_run as u32);
                    long_z = long_z.max(z_run);
                    m_block.add_run(&RIGHT_FACE, 1, y_run, z_run as u8);
                } else {
                    m_block.add_face(&RIGHT_FACE);
                }
            }
            current.set_right();
        }
        if !current.front() {
            if z == 0 || data.block_meta(x, y, z - 1).is_transparent() {
                let mut x_run = 1;
                for nx in (x + 1)..data.size.x {
                    if data.texture(nx, y, z) != block {
                        break;
                    }
                    if z != 0 && !data.block_meta(nx, y, z - 1).is_transparent() {
                        break;
                    }
                    let other = checked.entry(UVec3::new(nx, y, z)).or_default();
                    if other.front() {
                        break; // Already checked
                    }
                    x_run += 1;
                    other.set_front();
                }
                let mut y_run = 1;
                'y_look: for ny in (y + 1)..data.size.y {
                    for nx in x..(x + x_run) {
                        if data.texture(nx, ny, z) != block {
                            break 'y_look;
                        }
                        if z != 0 && !data.block_meta(nx, ny, z - 1).is_transparent() {
                            break 'y_look;
                        }
                        if checked
                            .get(&UVec3::new(nx, ny, z))
                            .is_some_and(|f| f.front())
                        {
                            break 'y_look; // Already checked
                        }
                    }
                    for nx in x..(x + x_run) {
                        checked
                            .entry(UVec3::new(nx, ny, z))
                            .or_insert_with(Face::from_front)
                            .set_front();
                    }
                    y_run += 1;
                }
                if y_run > 1 || x_run > 1 {
                    long_x = long_x.max(x_run);
                    long_y = long_y.max(y_run as u32);
                    // long_z = long_z.max(z_run);
                    m_block.add_run(&FRONT_FACE, x_run as u8, y_run, 1);
                } else {
                    m_block.add_face(&FRONT_FACE);
                }
            };
            current.set_front();
        }
        if !current.back() {
            if data.block_meta(x, y, z + 1).is_transparent() {
                let mut x_run = 1;
                for nx in (x + 1)..data.size.x {
                    if data.texture(nx, y, z) != block {
                        break;
                    }
                    if !data.block_meta(nx, y, z + 1).is_transparent() {
                        break;
                    }
                    let other = checked.entry(UVec3::new(nx, y, z)).or_default();
                    if other.back() {
                        break; // Already checked
                    }
                    x_run += 1;
                    other.set_back();
                }
                let mut y_run = 1;
                'y_look: for ny in (y + 1)..data.size.y {
                    for nx in x..(x + x_run) {
                        if data.texture(nx, ny, z) != block {
                            break 'y_look;
                        }
                        if !data.block_meta(nx, ny, z + 1).is_transparent() {
                            break 'y_look;
                        }
                        if checked
                            .get(&UVec3::new(nx, ny, z))
                            .is_some_and(|f| f.back())
                        {
                            break 'y_look; // Already checked
                        }
                    }
                    for nx in x..(x + x_run) {
                        checked
                            .entry(UVec3::new(nx, ny, z))
                            .or_insert_with(Face::from_back)
                            .set_back();
                    }
                    y_run += 1;
                }
                if y_run > 1 || x_run > 1 {
                    long_x = long_x.max(x_run);
                    long_y = long_y.max(y_run as u32);
                    // long_z = long_z.max(z_run);
                    m_block.add_run(&BACK_FACE, x_run as u8, y_run, 1);
                } else {
                    m_block.add_face(&BACK_FACE);
                }
            };
            current.set_back();
        }
        let id = data.texture(x, y, z);
        min.x = x.min(min.x);
        min.y = y.min(min.y);
        min.z = z.min(min.z);
        max.x = (x + long_x).max(max.x);
        max.y = (y + long_y).max(max.y);
        max.z = (z + long_z).max(max.z);
        checked.insert(UVec3::new(x, y, z), current);
        indices.extend(m_block.indices.iter().map(|i| positions.len() as u32 + i));
        positions.extend(m_block.vertexs.iter().map(|p| {
            let p = p.0.to_pos(p.1, p.2, p.3);
            let x = p[0] + x;
            let y = p[1] + y;
            let z = p[2] + z;
            #[cfg(feature = "standerd_position")]
            positions_old.push([x as f32, y as f32, z as f32]);
            x | y << CHUNK_SIZE.bits_per_axis()
                | z << (CHUNK_SIZE.bits_per_axis() * 2)
                | id << (8 + (CHUNK_SIZE.bits_per_axis() * 2))
            // 9 bits left
        }));
    }
    mesh.insert_attribute(crate::simple_shader::BLOCK_DATA, positions);
    #[cfg(feature = "standerd_position")]
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions_old);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    (mesh, Aabb::from_min_max(min.as_vec3(), max.as_vec3()))
}

#[derive(Default)]
struct Face(u8);

impl std::fmt::Display for Face {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        if self.top() {
            s.push('T');
        }
        if self.bottom() {
            s.push('B');
        }
        if self.left() {
            s.push('L');
        }
        if self.right() {
            s.push('R');
        }
        if self.back() {
            s.push('K');
        }
        if self.front() {
            s.push('F');
        }
        write!(f, "Face({})", s)
    }
}

impl Face {
    fn top(&self) -> bool {
        self.0 & 0b0001 != 0
    }
    fn bottom(&self) -> bool {
        self.0 & 0b0010 != 0
    }
    fn left(&self) -> bool {
        self.0 & 0b0100 != 0
    }
    fn right(&self) -> bool {
        self.0 & 0b1000 != 0
    }
    fn back(&self) -> bool {
        self.0 & 0b0001_0000 != 0
    }
    fn front(&self) -> bool {
        self.0 & 0b0010_0000 != 0
    }
    fn set_top(&mut self) {
        self.0 |= 0b0001;
    }
    fn set_bottom(&mut self) {
        self.0 |= 0b0010;
    }
    fn set_left(&mut self) {
        self.0 |= 0b0100;
    }
    fn set_right(&mut self) {
        self.0 |= 0b1000;
    }
    fn set_back(&mut self) {
        self.0 |= 0b0001_0000;
    }
    fn set_front(&mut self) {
        self.0 |= 0b0010_0000;
    }
    fn all(&self) -> bool {
        self.0 == 0b0011_1111
    }

    fn from_top() -> Self {
        Face(0b0001)
    }
    fn from_bottom() -> Self {
        Face(0b0010)
    }
    fn from_left() -> Self {
        Face(0b0100)
    }
    fn from_right() -> Self {
        Face(0b1000)
    }
    fn from_back() -> Self {
        Face(0b0001_0000)
    }
    fn from_front() -> Self {
        Face(0b0010_0000)
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
    vertexs: Vec<(Vertex, u8, u8, u8)>,
    indices: Vec<u32>,
}

impl VertexSet {
    fn add_run(&mut self, face: &[Vertex; 4], x_run: u8, y_run: u8, z_run: u8) {
        let start = self.vertexs.len() as u32;
        for face in face {
            self.vertexs.push((*face, x_run, y_run, z_run));
        }
        for i in [0, 1, 2, 0, 2, 3] {
            self.indices.push(start + i);
        }
    }

    fn add_face(&mut self, face: &[Vertex; 4]) {
        for i in [0, 1, 2, 0, 2, 3] {
            let v = face[i];
            let i = self.index(v, 1, 1, 1);
            self.indices.push(i);
        }
    }

    fn index(&mut self, vertex: Vertex, x_run: u8, y_run: u8, z_run: u8) -> u32 {
        let update = || {
            let i = self.vertexs.len();
            self.vertexs.push((vertex, x_run, y_run, z_run));
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
    fn to_pos(self, x_run: u8, y_run: u8, z_run: u8) -> [u32; 3] {
        match self {
            Vertex::LeftBottomFront => [0, 0, 0],
            Vertex::RightTopBack => [x_run as u32, y_run as u32, z_run as u32],
            Vertex::RightTopFront => [x_run as u32, y_run as u32, 0],
            Vertex::LeftTopBack => [0, y_run as u32, z_run as u32],

            Vertex::LeftTopFront => [0, y_run as u32, 0],
            Vertex::RightBottomBack => [x_run as u32, 0, z_run as u32],
            Vertex::LeftBottomBack => [0, 0, z_run as u32],
            Vertex::RightBottomFront => [x_run as u32, 0, 0],
        }
    }
}
