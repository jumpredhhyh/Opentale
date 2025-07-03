use bevy::prelude::*;

#[derive(Resource)]
pub struct TextureAtlasHandle {
    pub handle: Handle<Image>,
}

pub fn texture_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_atlas = asset_server.load("atlas.png");

    commands.insert_resource(TextureAtlasHandle {
        handle: texture_atlas,
    });
}
