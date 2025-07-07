use bevy::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use crate::debug_tools::debug_resource::OpentaleDebugResource;

pub struct OpentaleDebugPlugin;

impl Plugin for OpentaleDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OpentaleDebugResource>()
            .register_type::<OpentaleDebugResource>()
            .add_plugins(ResourceInspectorPlugin::<OpentaleDebugResource>::default());
    }
}
