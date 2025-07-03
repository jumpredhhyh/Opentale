use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    usize,
};

use bevy::{
    log::error,
    math::IVec3,
    render::render_resource::{ShaderSize, ShaderType},
};

use super::{BlockType, CHUNK_SIZE};

#[derive(Debug, Clone, ShaderType, Default, Copy)]
pub struct Vec4<T: ShaderSize> {
    one: T,
    two: T,
    three: T,
    four: T,
}

impl<T: ShaderSize> Index<usize> for Vec4<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.one,
            1 => &self.two,
            2 => &self.three,
            3 => &self.four,
            _ => panic!("Outisde of Range!"),
        }
    }
}

impl<T: ShaderSize> IndexMut<usize> for Vec4<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.one,
            1 => &mut self.two,
            2 => &mut self.three,
            3 => &mut self.four,
            _ => panic!("Outisde of Range!"),
        }
    }
}

pub type VoxelArray = [BlockType; (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2)];
pub type VoxelPalette = [Vec4<u32>; 128];

pub struct VoxelData {
    pub array: VoxelArray,
}

impl Default for VoxelData {
    fn default() -> Self {
        Self {
            array: [BlockType::Air; (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2)],
        }
    }
}

impl VoxelData {
    pub fn is_air<T: Into<IVec3>>(&self, position: T) -> bool {
        let index = Self::position_to_indexes(position);
        self.array[index] == BlockType::Air
    }

    pub fn get_block<T: Into<IVec3>>(&self, position: T) -> BlockType {
        let index = Self::position_to_indexes(position);
        self.array[index]
    }

    pub fn set_block<T: Into<IVec3>>(&mut self, position: T, block: BlockType) {
        let index = Self::position_to_indexes(position);
        self.array[index] = block;
    }

    fn position_to_indexes<T: Into<IVec3>>(position: T) -> usize {
        let position: IVec3 = position.into();
        let index = position.x as usize
            + (position.y as usize * (CHUNK_SIZE + 2))
            + (position.z as usize * (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2));
        index
    }
}
