pub const HALF_BLOCK_SIZE: f32 = 0.25;

#[derive(Clone, Copy, std::cmp::PartialEq)]
pub enum BlockType {
    AIR = 0,
    STONE = 1,
}

#[derive(Clone, Copy)]
pub struct Block {
    pub is_active: bool,
    pub block_type: BlockType,
}

impl Block {
    pub fn new() -> Self {
        Block {
            is_active: false,
            block_type: BlockType::STONE,
        }
    }

    #[allow(dead_code)]
    pub fn from(b_type: BlockType) -> Self {
        Block {
            is_active: false,
            block_type: b_type,
        }
    }
}