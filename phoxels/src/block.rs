/// BlockMeta holds info about a block that is used when generating meshes & coliders;
/// bit 0: solid
/// bit 1: opaque
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlockMeta(pub(crate) u8);

impl BlockMeta {
    pub const EMPTY: BlockMeta = BlockMeta(0);
}

impl BlockMeta {
    pub fn is_solid(&self) -> bool {
        self.0 & 0b0000_0001 != 0
    }

    pub fn is_transparent(&self) -> bool {
        self.0 & 0b0000_0010 == 0
    }
}

impl<T: Block> From<T> for BlockMeta {
    fn from(block: T) -> Self {
        let mut meta = BlockMeta::EMPTY;
        if block.is_solid() {
            meta.0 |= 0b0000_0001; // Set solid bit
        }
        if !block.is_transparent() {
            meta.0 |= 0b0000_0010; // Set opaque bit
        }
        meta
    }
}

pub trait Block: Copy {
    fn is_solid(&self) -> bool;
    fn is_transparent(&self) -> bool;
    fn id(&self) -> u8;
}

/// A BlockId is a simple wrapper around a u8 that represents the ID of a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlockId(pub u8);

impl PartialEq<u8> for BlockId {
    fn eq(&self, other: &u8) -> bool {
        self.0 == *other
    }
}

#[test]
fn empty_meta() {
    let air = BlockMeta::EMPTY;
    assert!(air.is_transparent());
    assert!(!air.is_solid());
}
