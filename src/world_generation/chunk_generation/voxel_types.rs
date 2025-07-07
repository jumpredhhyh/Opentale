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

pub const CHUNK_LENGTH: usize = (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RunLength(pub u32);

pub type VoxelArray = Vec<(BlockType, RunLength)>;
pub type VoxelPalette = [Vec4<u32>; 128];

pub struct VoxelData {
    pub array: VoxelArray,
}

impl Default for VoxelData {
    fn default() -> Self {
        Self {
            array: vec![(BlockType::Air, RunLength(CHUNK_LENGTH as u32))],
        }
    }
}

impl VoxelData {
    pub fn is_air<T: Into<IVec3>>(&self, position: T) -> bool {
        self.get_block(position) == BlockType::Air
    }

    pub fn is_air_amortized<T: Into<IVec3>>(
        &self, 
        start: (usize, RunLength), 
        position: T
    ) -> (bool, (usize, RunLength)) {
        let (block, next) = self.get_block_amortized(start, position);
        let is_air = block == BlockType::Air;
        return (is_air, next);
    }

    pub fn get_block<T: Into<IVec3>>(&self, position: T) -> BlockType {
        let index = Self::position_to_indexes(position);

        let mut count = 0;

        for (block_type, RunLength(run_len)) in &self.array {
            if count + (*run_len as usize) <= index {
                count += *run_len as usize;
                continue;
            }
            return *block_type;
        }

        panic!("{index} was out of bounds");
    }

    //TODO: we can turn this into an iterator which would be more safe and robust

    /// We return the index in voxel data, so we can avoid redudant checks.
    /// This should go from quadratic to linear amortized.
    /// Further benchmarks are needed. 
    pub fn get_block_amortized<T: Into<IVec3>>(
        &self, 
        (start_index, RunLength(start_count)): (usize, RunLength), 
        position: T
    ) -> (BlockType, (usize, RunLength)) {
        let index = Self::position_to_indexes(position);

        let mut count = start_count;

        for (
            i, (block_type, RunLength(run_len))
        ) in self.array[start_index..].iter().enumerate() {
            if ((count + run_len) as usize) <= index {
                count += run_len;
                continue;
            }
            return (*block_type, (start_index + i, RunLength(count)));
        }

        panic!("{index} was out of bounds");
    }

    pub fn set_block<T: Into<IVec3> + Clone>(&mut self, position: T, block: BlockType) {
        let index = Self::position_to_indexes(position.clone());

        let mut count = 0;

        let mut i = 0;
        let len = self.array.len();
        while i < len {
            let (curr_block, RunLength(run_len)) = self.array[i];

            if count + (run_len as usize) <= index {
                count += run_len as usize;
                i += 1;
                continue;
            }

            if curr_block == block {
                return;
            }

            let (pre_len, post_len);
            pre_len = index - count;
            post_len = (count + run_len as usize - 1) - index;

            match (pre_len, post_len) {
                (0, 0) => {
                    let is_next_run_same = self.array.get(i + 1)
                        .is_some_and(|(next_block_type, _)| 
                            *next_block_type == block
                        );
                    let is_prev_run_same = i > 0 && self.array.get(i - 1)
                        .is_some_and(|(prev_block_type, _)|
                            *prev_block_type == block
                        );

                    if is_next_run_same && is_prev_run_same {
                        let (_, RunLength(next_run_len)) = 
                            self.array[i + 1];
                        let (_, RunLength(ref mut prev_run_len)) = 
                            self.array[i - 1];

                        *prev_run_len += 1 + next_run_len;

                        // We remove the run we were looking at and
                        // the next run because the prev run absorbs them
                        self.array.remove(i);
                        self.array.remove(i);
                    } else if is_next_run_same {
                        let (_, RunLength(ref mut next_run_len)) = 
                            self.array[i + 1];

                        *next_run_len += 1;

                        // We remove the run we were looking at because
                        // the next run absorbs it
                        self.array.remove(i);
                    } else if is_prev_run_same {
                        let (_, RunLength(ref mut prev_run_len)) = 
                            self.array[i - 1];

                        *prev_run_len += 1;

                        // We remove the run we were looking at because
                        // the prev run absorbs it
                        self.array.remove(i);
                    } else {
                        let (ref mut block_type, _) = self.array[i];
                        *block_type = block;
                    }
                }
                (0, _) => {
                    let is_prev_run_same = i > 0 && self.array.get(i - 1)
                        .is_some_and(|(prev_block_type, _)| 
                            *prev_block_type == block
                        );

                    if is_prev_run_same {
                        let (_, RunLength(ref mut prev_run_len)) = 
                            self.array[i - 1];

                        *prev_run_len += 1;

                        let (_, RunLength(ref mut curr_run_len)) = 
                            self.array[i];

                        *curr_run_len -= 1;
                    } else {
                        let (_, RunLength(ref mut curr_run_len)) = self.array[i];
                        *curr_run_len -= 1;

                        self.array.insert(i, (block, RunLength(1)));
                    }
                }
                (_, 0) => {
                    let is_next_run_same = self.array.get(i + 1)
                        .is_some_and(|(next_block_type, _)| 
                            *next_block_type == block
                        );
                        
                    if is_next_run_same {
                        let (_, RunLength(ref mut next_run_len)) = 
                            self.array[i + 1];

                        *next_run_len += 1;

                        let (_, RunLength(ref mut curr_run_len)) = 
                            self.array[i];

                        *curr_run_len -= 1;
                    } else {
                        let (_, RunLength(ref mut curr_run_len)) = 
                            self.array[i];

                        *curr_run_len -= 1;

                        self.array.insert(i + 1, (block, RunLength(1)));
                    }
                }
                (_, _) => {
                    let (curr_block_type, RunLength(ref mut curr_run_len)) = 
                        self.array[i];

                    *curr_run_len = pre_len as u32;

                    self.array.insert(i + 1, (
                        block, 
                        RunLength(1)
                    ));
                    self.array.insert(i + 2, (
                        curr_block_type, 
                        RunLength(post_len as u32)
                    ));
                }
            }

            return;
        }

        panic!("{index} was out of bounds at position {:?}", position.into());
    }

    /// We return the index in voxel data, so we can avoid redudant checks.
    /// This should go from quadratic to linear amortized.
    /// Further benchmarks are needed. 
    /// 
    /// We assume that the next block that is being set
    /// will be after this block, meaning if we set a block
    /// in position 68, we assume the next block will be
    /// set in positions 69 and greater
    pub fn set_block_amortized<T: Into<IVec3>>(
        &mut self, 
        (start_index, RunLength(start_count)): (usize, RunLength),
        position: T, block: BlockType
    ) -> (usize, RunLength) {
        let index = Self::position_to_indexes(position);

        if index < start_count as usize {
            panic!("index {index} should be greater than starting count {start_count}");
        }

        let mut count = start_count;

        let mut i = start_index;
        let len = self.array.len();
        while i < len {
            let (curr_block, RunLength(run_len)) = self.array[i];

            if ((count + run_len) as usize) <= index {
                count += run_len;
                i += 1;
                continue;
            }

            if curr_block == block {
                return (i, RunLength(count));
            }

            let (pre_len, post_len);
            pre_len = index - count as usize;
            post_len = ((count + run_len) as usize - 1) - index;
            let next_index;

            match (pre_len, post_len) {
                (0, 0) => {
                    let is_next_run_same = self.array.get(i + 1)
                        .is_some_and(|(next_block_type, _)| 
                            *next_block_type == block
                        );
                    let is_prev_run_same = i > 0 && self.array.get(i - 1)
                        .is_some_and(|(prev_block_type, _)|
                            *prev_block_type == block
                        );

                    if is_next_run_same && is_prev_run_same {
                        let (_, RunLength(next_run_len)) = 
                            self.array[i + 1];
                        let (_, RunLength(ref mut prev_run_len)) = 
                            self.array[i - 1];

                        count -= *prev_run_len;
                        *prev_run_len += 1 + next_run_len;

                        // We remove the run we were looking at and
                        // the next run because the prev run absorbs them
                        self.array.remove(i);
                        self.array.remove(i);

                        // We have to backtrack a bit, because the next run
                        // is absorbed by the prev run, and the next thing
                        // to check is the next run
                        next_index = i - 1;
                    } else if is_next_run_same {
                        let (_, RunLength(ref mut next_run_len)) = 
                            self.array[i + 1];

                        *next_run_len += 1;

                        // We remove the run we were looking at because
                        // the next run absorbs it
                        self.array.remove(i);

                        next_index = i;
                    } else if is_prev_run_same {
                        let (_, RunLength(ref mut prev_run_len)) = 
                            self.array[i - 1];

                        *prev_run_len += 1;

                        // We remove the run we were looking at because
                        // the prev run absorbs it
                        self.array.remove(i);

                        next_index = i;
                        count += 1;
                    } else {
                        let (ref mut block_type, _) = self.array[i];
                        *block_type = block;

                        next_index = i + 1;
                    }
                }
                (0, _) => {
                    let is_prev_run_same = i > 0 && self.array.get(i - 1)
                        .is_some_and(|(prev_block_type, _)| 
                            *prev_block_type == block
                        );

                    if is_prev_run_same {
                        let (_, RunLength(ref mut prev_run_len)) = 
                            self.array[i - 1];

                        *prev_run_len += 1;

                        let (_, RunLength(ref mut curr_run_len)) = 
                            self.array[i];

                        *curr_run_len -= 1;

                        next_index = i;
                        count += 1;
                    } else {
                        let (_, RunLength(ref mut curr_run_len)) = self.array[i];

                        *curr_run_len -= 1;

                        self.array.insert(i, (block, RunLength(1)));

                        next_index = i + 1;
                        count += 1;
                    }
                }
                (_, 0) => {
                    let is_next_run_same = self.array.get(i + 1)
                        .is_some_and(|(next_block_type, _)| 
                            *next_block_type == block
                        );
                        
                    if is_next_run_same {
                        let (_, RunLength(ref mut next_run_len)) = 
                            self.array[i + 1];

                        *next_run_len += 1;

                        let (_, RunLength(ref mut curr_run_len)) = 
                            self.array[i];

                        *curr_run_len = pre_len as u32;

                        next_index = i + 1;
                        count += pre_len as u32;
                    } else {
                        let (_, RunLength(ref mut curr_run_len)) = 
                            self.array[i];

                        *curr_run_len -= 1;

                        self.array.insert(i + 1, (block, RunLength(1)));

                        next_index = i + 2;
                        count += pre_len as u32 + 1;
                    }
                }
                (_, _) => {
                    let (curr_block_type, RunLength(ref mut curr_run_len)) = 
                        self.array[i];

                    *curr_run_len = pre_len as u32;

                    self.array.insert(i + 1, (
                        block, 
                        RunLength(1)
                    ));
                    self.array.insert(i + 2, (
                        curr_block_type, 
                        RunLength(post_len as u32)
                    ));

                    next_index = i + 2;
                    count += pre_len as u32 + 1;
                }
            }

            return (next_index, RunLength(count));
        }

        panic!("{index} was out of bounds");
    }

    fn position_to_indexes<T: Into<IVec3>>(position: T) -> usize {
        let position: IVec3 = position.into();
        let index = position.x as usize
            + (position.y as usize * (CHUNK_SIZE + 2))
            + (position.z as usize * (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2));
        index
    }
}

#[cfg(test)]
mod test {
    use bevy::math::IVec3;

    use crate::world_generation::chunk_generation::{voxel_types::{RunLength, VoxelData, CHUNK_LENGTH}, BlockType, CHUNK_SIZE};

    #[test]
    fn test_get_block_1() {
        let voxel_data = VoxelData {
            array: vec![
                (BlockType::Stone, RunLength(6)),
                (BlockType::Path, RunLength(6)),
                (BlockType::Stone, RunLength(6)),
            ]
        };

        let blocks = [
            BlockType::Stone,
            BlockType::Stone,
            BlockType::Path,
            BlockType::Stone,
            BlockType::Stone
        ];

        let positions = [
            IVec3::new(0, 0, 0),
            IVec3::new(4, 0, 0),
            IVec3::new(8, 0, 0),
            IVec3::new(12, 0, 0),
            IVec3::new(16, 0, 0),
        ];

        let indices = [0, 4, 8, 12, 16];

        for (
            test_block, 
            (position, index)
        ) in blocks.into_iter().zip(
            positions.into_iter().zip(indices)
        ) {
            assert_eq!(
                VoxelData::position_to_indexes(position), index,
                "checking if position {position} to index is {index}"
            );

            let block = voxel_data.get_block(position);
            assert_eq!(
                block, test_block, 
                "comparing {block:?} at position {position} to {test_block:?}"
            );
        }
    }

    #[test]
    fn test_get_block_2() {
        let mut voxel_data = VoxelData::default();

        let blocks = [
            BlockType::Air, 
            BlockType::Stone, 
            BlockType::Path, 
            BlockType::Snow
        ].iter().cycle();

        let positions = (0..CHUNK_LENGTH).map(|e|
            IVec3::new(
                (e % (CHUNK_SIZE + 2)) as i32, 
                ((e / (CHUNK_SIZE + 2)) % (CHUNK_SIZE + 2)) as i32,
                (e / ((CHUNK_SIZE + 2) * (CHUNK_SIZE + 2))) as i32,
            )
        );

        let mut start = (0, RunLength(0));
        for (block, position) in blocks.zip(positions) {
            start = voxel_data.set_block_amortized(start, position, *block);
        }

        
    }

    #[test]
    fn test_get_block_amortized() {
        let voxel_data = VoxelData {
            array: vec![
                (BlockType::Stone, RunLength(6)),
                (BlockType::Path, RunLength(6)),
                (BlockType::Stone, RunLength(6)),
            ]
        };
        
        let test_blocks = [
            [BlockType::Stone; 6],
            [BlockType::Path; 6],
            [BlockType::Stone; 6]
        ].concat();
        
        let positions = (0..18).map(|x| IVec3::new(x, 0, 0));

        let mut start = (0, RunLength(0));
        for (test_block, position) in test_blocks.into_iter().zip(positions) {
            let (got_block, new_start) = voxel_data.get_block_amortized(
                start, position
            );

            assert_eq!(
                got_block, test_block, 
                "getting block amortized at position {position}"
            );

            start = new_start;
        }
    }

    #[test]
    fn test_set_block_1() {
        let mut voxel_data = VoxelData {
            array: vec![
                (BlockType::Stone, RunLength(6)),
                (BlockType::Path, RunLength(6)),
                (BlockType::Stone, RunLength(5)),
                (BlockType::Snow, RunLength(1)),
            ]
        };

        let blocks_to_place = [
            BlockType::Stone,
            BlockType::Snow,
            BlockType::Snow,
            BlockType::Path,
            BlockType::Snow
        ];

        let positions = [
            IVec3::new(0, 0, 0),
            IVec3::new(4, 0, 0),
            IVec3::new(8, 0, 0),
            IVec3::new(12, 0, 0),
            IVec3::new(16, 0, 0),
        ];

        let indices = [0, 4, 8, 12, 16];

        for (
            block_to_place, 
            (position, index)
        ) in blocks_to_place.into_iter().zip(
            positions.into_iter().zip(indices)
        ) {
            assert_eq!(
                VoxelData::position_to_indexes(position), index,
                "checking if position {position} to index is {index}"
            );

            voxel_data.set_block(position, block_to_place);
        }

        assert_eq!(
            voxel_data.array, vec![
                (BlockType::Stone, RunLength(4)),
                (BlockType::Snow, RunLength(1)),
                (BlockType::Stone, RunLength(1)),
                (BlockType::Path, RunLength(2)),
                (BlockType::Snow, RunLength(1)),
                (BlockType::Path, RunLength(4)),
                (BlockType::Stone, RunLength(3)),
                (BlockType::Snow, RunLength(2)),
            ]
        );
    }

    #[test]
    fn test_set_block_2() {
        let mut voxel_data = VoxelData {
            array: vec![
                (BlockType::Stone, RunLength(5)), // Setting Middle
                (BlockType::Path,  RunLength(5)), // Setting Beginning Diff Prev
                (BlockType::Stone, RunLength(5)), // Setting Beginning Same Prev
                (BlockType::Path,  RunLength(5)), // Setting End Diff Next
                (BlockType::Stone, RunLength(5)), // Setting End Same Next
                (BlockType::Path,  RunLength(2)),
                (BlockType::Stone, RunLength(1)), // Setting Single Diff Prev + Diff Next
                (BlockType::Path,  RunLength(2)),
                (BlockType::Stone, RunLength(1)), // Setting Single Diff Prev + Same Next
                (BlockType::Snow,  RunLength(2)), 
                (BlockType::Stone, RunLength(1)), // Setting Single Same Prev + Diff Next
                (BlockType::Path,  RunLength(2)),
                (BlockType::Stone, RunLength(1)), // Setting Single Same Prev + Same Next
                (BlockType::Path,  RunLength(2)),
            ]
        };

        let blocks_to_place = [
            BlockType::Snow, // Setting Middle
            BlockType::Snow, // Setting Beginning Diff Prev
            BlockType::Path, // Setting Beginning Same Prev
            BlockType::Snow, // Setting End Diff Next
            BlockType::Path, // Setting End Same Next
            BlockType::Snow, // Setting Single Diff Prev + Diff Next
            BlockType::Snow, // Setting Single Diff Prev + Same Next
            BlockType::Snow, // Setting Single Same Prev + Diff Next
            BlockType::Path, // Setting Single Same Prev + Same Next
        ];

        let positions = [
            IVec3::new(2, 0, 0),
            IVec3::new(5, 0, 0),
            IVec3::new(10, 0, 0),
            IVec3::new(19, 0, 0),
            IVec3::new(24, 0, 0),
            IVec3::new(27, 0, 0),
            IVec3::new(30, 0, 0),
            IVec3::new(33, 0, 0),
            IVec3::new(36, 0, 0),
        ];

        let indices = [2, 5, 10, 19, 24, 27, 30, 33, 36];

        for (
            block_to_place, 
            (position, index)
        ) in blocks_to_place.into_iter().zip(
            positions.into_iter().zip(indices)
        ) {
            assert_eq!(
                VoxelData::position_to_indexes(position), index,
                "checking if position {position} to index is {index}"
            );

            voxel_data.set_block(position, block_to_place);
        }

        assert_eq!(
            voxel_data.array, vec![
                (BlockType::Stone, RunLength(2)),
                (BlockType::Snow,  RunLength(1)),
                (BlockType::Stone, RunLength(2)),
                (BlockType::Snow,  RunLength(1)),
                (BlockType::Path,  RunLength(5)),
                (BlockType::Stone, RunLength(4)),
                (BlockType::Path,  RunLength(4)),
                (BlockType::Snow,  RunLength(1)),
                (BlockType::Stone, RunLength(4)),
                (BlockType::Path,  RunLength(3)),
                (BlockType::Snow,  RunLength(1)),
                (BlockType::Path,  RunLength(2)),
                (BlockType::Snow,  RunLength(4)),
                (BlockType::Path,  RunLength(5)),
            ]
        );
    }

    #[test]
    fn test_set_block_3() {
        let mut voxel_data = VoxelData::default();

        let blocks = [
            BlockType::Air, 
            BlockType::Stone, 
            BlockType::Path, 
            BlockType::Snow
        ].iter().cycle();

        let positions = (0..CHUNK_LENGTH).map(|e|
            IVec3::new(
                (e % (CHUNK_SIZE + 2)) as i32, 
                ((e / (CHUNK_SIZE + 2)) % (CHUNK_SIZE + 2)) as i32,
                (e / ((CHUNK_SIZE + 2) * (CHUNK_SIZE + 2))) as i32,
            )
        );

        let mut start = (0, RunLength(0));
        for (block, position) in blocks.zip(positions) {
            start = voxel_data.set_block_amortized(start, position, *block);
        }

        let chunk_length = voxel_data.array
            .iter()
            .map(|(_, RunLength(run_len))| run_len)
            .sum::<u32>() as usize;
        let num_runs = voxel_data.array.len();

        assert_eq!(chunk_length, CHUNK_LENGTH);
        assert_eq!(num_runs, CHUNK_LENGTH);

        //println!("final chunk data: {:?}", voxel_data.array);
    }

    #[test]
    fn test_set_block_amortized() {
        let mut voxel_data = VoxelData {
            array: vec![
                (BlockType::Stone, RunLength(6)),
                (BlockType::Path, RunLength(6)),
                (BlockType::Stone, RunLength(5)),
                (BlockType::Snow, RunLength(1)),
            ]
        };

        let blocks_to_place = vec![
            vec![BlockType::Stone; 4],
            vec![BlockType::Snow; 1],
            vec![BlockType::Stone; 1],
            vec![BlockType::Path; 2],
            vec![BlockType::Snow; 1],
            vec![BlockType::Path; 4],
            vec![BlockType::Stone; 3],
            vec![BlockType::Snow; 2],
        ].into_iter().flatten();

        let positions = (0..18).map(|x| IVec3::new(x, 0, 0));

        let mut start = (0, RunLength(0));
        for (block_to_place, position) in blocks_to_place.zip(positions) {
            start = voxel_data.set_block_amortized(
                start, position, block_to_place
            );
        }

        assert_eq!(
            voxel_data.array, vec![
                (BlockType::Stone, RunLength(4)),
                (BlockType::Snow, RunLength(1)),
                (BlockType::Stone, RunLength(1)),
                (BlockType::Path, RunLength(2)),
                (BlockType::Snow, RunLength(1)),
                (BlockType::Path, RunLength(4)),
                (BlockType::Stone, RunLength(3)),
                (BlockType::Snow, RunLength(2)),
            ]
        );
    }
}