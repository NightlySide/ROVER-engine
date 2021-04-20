use crate::render::vertex::ColorVertex;
use super::block::{Block, HALF_BLOCK_SIZE, BlockType};
use super::noise;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 32;

pub struct Chunk {
    pub width: usize,
    pub height: usize,
    pub blocks: [[[Block; CHUNK_WIDTH]; CHUNK_HEIGHT]; CHUNK_WIDTH],
}

impl Chunk {
    pub fn new() -> Self {
        let noise_gen = noise::NoiseGenerator::from_seed(1337);
        let mut blocks = [[[Block::new(); CHUNK_WIDTH]; CHUNK_HEIGHT]; CHUNK_WIDTH];
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                let noise_value = noise_gen.get(x as f64 / 16.0, z as f64 / 16.0) * CHUNK_HEIGHT as f64;
                for y in 0..CHUNK_HEIGHT {
                    blocks[x][y][z].block_type = if y as f64 > noise_value {
                        BlockType::AIR
                    } else { BlockType::STONE };
                }
            }
        }
        Chunk {
            width: CHUNK_WIDTH,
            height: CHUNK_HEIGHT,
            blocks,
        }
    }

    pub fn create_mesh(&self) -> (Vec<ColorVertex>, Vec<u16>) {
        let mut vertices: Vec<ColorVertex> = vec![];
        let mut indices: Vec<u16> = vec![];
        
        for y in 0..CHUNK_HEIGHT {
            for x in 0..CHUNK_WIDTH {
                for z in 0..CHUNK_WIDTH {
                    if self.blocks[x][y][z].block_type != BlockType::AIR {
                        let (mut v_cube, mut i_cube) = self.create_cube(vertices.len(), x, y, z);
                        vertices.append(&mut v_cube);
                        indices.append(&mut i_cube);
                    }
                }
            }
        }

        println!("Sending to GPU: {} vertices and {} indices", vertices.len(), indices.len());

        (vertices, indices)
    }

    fn create_cube(&self, idx_offset: usize, x: usize, y: usize, z: usize) -> (Vec<ColorVertex>, Vec<u16>) {
        //println!("Block: x: {} y: {} z: {}", x, y, z);
        let color: f32 = idx_offset as f32 / (CHUNK_HEIGHT * CHUNK_WIDTH * 36 * 8) as f32;
    
        let px = x as f32 * 2.0 * HALF_BLOCK_SIZE;
        let py = y as f32 * 2.0 * HALF_BLOCK_SIZE;
        let pz = z as f32 * 2.0 * HALF_BLOCK_SIZE;

        let v_cube = vec![
            // front
            ColorVertex { position: [px-HALF_BLOCK_SIZE, py-HALF_BLOCK_SIZE, pz-HALF_BLOCK_SIZE], color: [color, 0.0, 0.0] },
            ColorVertex { position: [px+HALF_BLOCK_SIZE, py-HALF_BLOCK_SIZE, pz-HALF_BLOCK_SIZE], color: [color, 0.0, 0.0] },
            ColorVertex { position: [px+HALF_BLOCK_SIZE, py+HALF_BLOCK_SIZE, pz-HALF_BLOCK_SIZE], color: [color, 0.0, 0.0] },
            ColorVertex { position: [px-HALF_BLOCK_SIZE, py+HALF_BLOCK_SIZE, pz-HALF_BLOCK_SIZE], color: [color, 0.0, 0.0] },
            // Back
            ColorVertex { position: [px+HALF_BLOCK_SIZE, py-HALF_BLOCK_SIZE, pz+HALF_BLOCK_SIZE], color: [0.0, 0.0, color] },
            ColorVertex { position: [px-HALF_BLOCK_SIZE, py-HALF_BLOCK_SIZE, pz+HALF_BLOCK_SIZE], color: [0.0, 0.0, color] },
            ColorVertex { position: [px-HALF_BLOCK_SIZE, py+HALF_BLOCK_SIZE, pz+HALF_BLOCK_SIZE], color: [0.0, 0.0, color] },
            ColorVertex { position: [px+HALF_BLOCK_SIZE, py+HALF_BLOCK_SIZE, pz+HALF_BLOCK_SIZE], color: [0.0, 0.0, color] },
        ];
    
        let mut i_cube: Vec<u16> = vec![];
        // culling
        if z == CHUNK_WIDTH - 1 || (z < CHUNK_WIDTH - 1 && self.blocks[x][y][z+1].block_type == BlockType::AIR) {
            i_cube.append(&mut add_face_indices(&Faces::BACK, idx_offset));
        }
        if z == 0 || (z > 0 && self.blocks[x][y][z-1].block_type == BlockType::AIR) {
            i_cube.append(&mut add_face_indices(&Faces::FRONT, idx_offset));
        }
        if x == CHUNK_WIDTH - 1 || (x < CHUNK_WIDTH - 1 && self.blocks[x+1][y][z].block_type == BlockType::AIR) {
            i_cube.append(&mut add_face_indices(&Faces::RIGHT, idx_offset));
        }
        if x == 0 || (x > 0 && self.blocks[x-1][y][z].block_type == BlockType::AIR) {
            i_cube.append(&mut add_face_indices(&Faces::LEFT, idx_offset));
        }
        if y == CHUNK_HEIGHT - 1 || (y < CHUNK_HEIGHT - 1 && self.blocks[x][y+1][z].block_type == BlockType::AIR) {
            i_cube.append(&mut add_face_indices(&Faces::TOP, idx_offset));
        }
        if y == 0 || (y > 0 && self.blocks[x][y-1][z].block_type == BlockType::AIR) {
            i_cube.append(&mut add_face_indices(&Faces::BOTTOM, idx_offset));
        }

        // Extreme culling
        if i_cube.len() == 0 {
            return (vec![], vec![]);
        }
    
        //println!("IDX: {:?}", i_cube);
    
        (v_cube, i_cube)
    }
}

#[derive(Clone, Copy)]
enum Faces {
    FRONT = 0,
    BACK = 1,
    TOP = 2,
    BOTTOM = 3,
    LEFT = 4,
    RIGHT = 5,
}

const BASE_INDICES: [[u16; 6]; 6] = [
    [0,1,3,  3,1,2], // Front
    [4,5,7,  7,5,6], // Back
    [3,2,6,  6,2,7], // Top
    [5,4,0,  0,4,1], // Bottom
    [5,0,6,  6,0,3], // Left
    [1,4,2,  2,4,7], // Right
];

fn add_face_indices(face: &Faces, idx_offset: usize) -> Vec<u16> {
    let mut res: Vec<u16> = vec![];
    for i in 0..6 {
        res.push(BASE_INDICES[*face as usize][i] + idx_offset as u16);
    }
    res
}