mod resources;
mod systems;

pub use resources::{WorldConfig, WorldEntity, WorldEntitySelectionType};
pub use systems::convert;

use bevy::prelude::*;
use resources::WorldTracker;

pub struct WorldPlugin {
    pub config: WorldConfig,
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(WorldTracker::default())
            .insert_resource(self.config.clone())
            .add_systems(Startup, systems::init_materials)
            .add_systems(Startup, systems::init_road_map)
            .add_systems(Update, systems::mark_on_changed_preprocess)
            .add_systems(Update, systems::colour_system) // Used for drawing the layers
            .add_systems(Update, systems::visible_entities);
    }
}
