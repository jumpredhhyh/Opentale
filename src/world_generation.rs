pub mod chunk_generation;
pub mod chunk_loading;
pub mod foliage_generation;
pub mod generation_options;
pub mod texture_loading;
pub mod voxel_world;

use crate::world_generation::chunk_generation::ChunkGenerationPlugin;
use crate::world_generation::texture_loading::texture_loading;
use bevy::app::{App, Startup};
use bevy::prelude::Plugin;

pub struct WorldGenerationPlugin;

impl Plugin for WorldGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, texture_loading)
            .add_plugins(ChunkGenerationPlugin);
    }
}
