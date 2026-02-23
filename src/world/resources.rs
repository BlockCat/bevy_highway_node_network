use bevy::prelude::*;
use bevy_polyline::prelude::PolylineMaterial;
use bevy_shapefile::RoadId;
use std::collections::HashMap;

#[derive(Debug, Clone, Resource)]
pub struct WorldConfig {
    pub road_map_path: String,
    pub geopackage_path: String,
    pub directed_graph_path: String,

    pub selected_colour: Color,
    pub normal_colour: Color,
}

#[derive(Resource)]
pub struct LoadedMaterials {
    pub normal_material: Handle<PolylineMaterial>,
    pub selected_material: Handle<PolylineMaterial>,

    pub outgoing_material: Handle<PolylineMaterial>,
    pub incoming_material: Handle<PolylineMaterial>,
    pub route_material: Handle<PolylineMaterial>,
}

#[derive(Debug, Clone, Component)]
pub struct WorldEntity {
    pub id: RoadId,
    pub selected: WorldEntitySelectionType,
}

#[derive(Debug, Clone)]
pub enum WorldEntitySelectionType {
    NotSelected,
    BaseSelected,
    BiDirection,
    Outgoing,
    Incoming,
    Route,
}

#[derive(Debug, Default, Resource)]
pub struct WorldTracker {
    pub map: HashMap<RoadId, Entity>,
}

impl WorldTracker {
    pub fn track(&mut self, id: RoadId, entity: Entity) {
        self.map.insert(id, entity);
    }
    pub fn remove(&mut self, id: RoadId) {
        self.map.remove(&id);
    }
}
