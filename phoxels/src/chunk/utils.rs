pub struct BlockIter<const CHUNK_SIZE: usize> {
    x: usize,
    y: usize,
    z: usize,
}

impl<const CHUNK_SIZE: usize> BlockIter<CHUNK_SIZE> {
    pub fn new() -> BlockIter<CHUNK_SIZE> {
        BlockIter { x: 0, y: 0, z: 0 }
    }
}

impl<const CHUNK_SIZE: usize> Iterator for BlockIter<CHUNK_SIZE> {
    type Item = (usize, usize, usize);
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
