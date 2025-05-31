use bevy::math::UVec3;

use crate::core::CHUNK_SIZE;

pub struct BlockIter<const X: u32, const Y: u32, const Z: u32> {
    x: u32,
    y: u32,
    z: u32,
}

impl<const X: u32, const Y: u32, const Z: u32> BlockIter<X, Y, Z> {
    pub fn new() -> BlockIter<X, Y, Z> {
        BlockIter { x: 0, y: 0, z: 0 }
    }
}

impl<const X: u32, const Y: u32, const Z: u32> Default for BlockIter<X, Y, Z> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const X: u32, const Y: u32, const Z: u32> Iterator for BlockIter<X, Y, Z> {
    type Item = (u32, u32, u32);
    fn next(&mut self) -> Option<Self::Item> {
        let out = if self.y >= Y {
            return None;
        } else {
            (self.x, self.y, self.z)
        };
        self.x += 1;
        if self.x >= X {
            self.x -= X;
            self.z += 1;
        }
        if self.z >= Z {
            self.z -= Z;
            self.y += 1
        }
        Some(out)
    }
}

pub struct DynBlockIter {
    size: UVec3,
    x: u32,
    y: u32,
    z: u32,
}

impl DynBlockIter {
    pub fn new(size: UVec3) -> DynBlockIter {
        DynBlockIter {
            size,
            x: 0,
            y: 0,
            z: 0,
        }
    }
}

impl Default for DynBlockIter {
    fn default() -> Self {
        Self::new(UVec3::splat(CHUNK_SIZE.size()))
    }
}

impl Iterator for DynBlockIter {
    type Item = (u32, u32, u32);
    fn next(&mut self) -> Option<Self::Item> {
        let out = if self.y >= self.size.y {
            return None;
        } else {
            (self.x, self.y, self.z)
        };
        self.x += 1;
        if self.x >= self.size.x {
            self.x -= self.size.x;
            self.z += 1;
        }
        if self.z >= self.size.z {
            self.z -= self.size.z;
            self.y += 1
        }
        Some(out)
    }
}
