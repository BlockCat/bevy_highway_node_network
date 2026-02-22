pub use road_map::*;
use rusqlite::types::FromSql;
use serde::{Deserialize, Serialize};
pub use spatial::*;

// mod road_data;
mod road_map;
mod spatial;

pub type AABB = rstar::AABB<[f32; 2]>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JunctionId(usize);

impl JunctionId {
    pub fn num(&self) -> usize {
        self.0
    }
}

impl From<usize> for JunctionId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl FromSql for JunctionId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let id = usize::column_result(value)?;
        Ok(JunctionId(id))
    }
}

//
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoadId(usize);

impl RoadId {
    pub fn num(&self) -> usize {
        self.0
    }
}

impl From<usize> for RoadId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl FromSql for RoadId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let id = usize::column_result(value)?;
        Ok(RoadId(id))
    }
}
