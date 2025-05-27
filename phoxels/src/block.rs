#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlockMeta {
    pub texture: u8,
    pub is_solid: bool,
    pub not_transparent: bool,
}

impl BlockMeta {
    pub const EMPTY: Self = BlockMeta {
        texture: 0,
        is_solid: false,
        not_transparent: false,
    };
}

impl BlockMeta {
    pub fn is_solid(&self) -> bool {
        self.is_solid
    }

    pub fn is_transparent(&self) -> bool {
        !self.not_transparent
    }

    pub fn texture(&self) -> u8 {
        self.texture
    }
}

impl<T: Block> From<T> for BlockMeta {
    fn from(block: T) -> Self {
        BlockMeta {
            texture: block.texture(),
            is_solid: block.is_solid(),
            not_transparent: block.is_transparent(),
        }
    }
}

pub trait Block {
    fn is_solid(&self) -> bool;
    fn is_transparent(&self) -> bool;
    fn texture(&self) -> u8;
}
