use bevy::math::{bounding::Aabb2d, Vec2};
use rstar::{PointDistance, RTreeObject, AABB};
use rusqlite::types::FromSql;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{JunctionId, RoadId};

/// A road section, with id, points and bounding box
#[derive(Serialize, Deserialize, Debug)]
pub struct RoadSection {
    pub points: Vec<Vec2>,
    #[serde(
        serialize_with = "serialize_aabb",
        deserialize_with = "deserialize_aabb"
    )]
    pub aabb: Aabb2d,
}

// https://www.gaia-gis.it/gaia-sins/BLOB-Geometry.html
impl FromSql for RoadSection {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let mut blob = value.as_blob()?.iter();
        let _ = blob.by_ref().take(1);
        let is_big_endian = match blob.next().unwrap() {
            0 => false,
            1 => true,
            _ => return Err(rusqlite::types::FromSqlError::InvalidType),
        };
        let _ = blob.by_ref().take(4);

        let minx = read_f64(blob.by_ref(), is_big_endian)?;
        let miny = read_f64(blob.by_ref(), is_big_endian)?;
        let maxx = read_f64(blob.by_ref(), is_big_endian)?;
        let maxy = read_f64(blob.by_ref(), is_big_endian)?;

        let metadata_end_byte = *blob.next().unwrap();

        if metadata_end_byte != 0x7C {
            return Err(rusqlite::types::FromSqlError::InvalidType);
        }

        let class_type = read_u32(blob.by_ref(), is_big_endian)?;
        if class_type != 2 {
            return Err(rusqlite::types::FromSqlError::InvalidType);
        }

        let point_count = read_u32(blob.by_ref(), is_big_endian)?;
        let mut points = Vec::with_capacity(point_count as usize);

        for _ in 0..point_count {
            let x = read_f64(blob.by_ref(), is_big_endian)?;
            let y = read_f64(blob.by_ref(), is_big_endian)?;
            points.push(Vec2::new(x as f32, y as f32));
        }

        let end_byte = *blob.next().unwrap();
        if end_byte != 0xFE {
            return Err(rusqlite::types::FromSqlError::InvalidType);
        }

        Ok(RoadSection {
            points,
            aabb: Aabb2d {
                min: Vec2::new(minx as f32, miny as f32),
                max: Vec2::new(maxx as f32, maxy as f32),
            },
        })
    }
}

fn read_f64<'a, I>(iter: &mut I, is_big_endian: bool) -> rusqlite::types::FromSqlResult<f64>
where
    I: Iterator<Item = &'a u8>,
{
    let bytes: [u8; 8] = [
        *iter
            .next()
            .ok_or(rusqlite::types::FromSqlError::InvalidType)?,
        *iter
            .next()
            .ok_or(rusqlite::types::FromSqlError::InvalidType)?,
        *iter
            .next()
            .ok_or(rusqlite::types::FromSqlError::InvalidType)?,
        *iter
            .next()
            .ok_or(rusqlite::types::FromSqlError::InvalidType)?,
        *iter
            .next()
            .ok_or(rusqlite::types::FromSqlError::InvalidType)?,
        *iter
            .next()
            .ok_or(rusqlite::types::FromSqlError::InvalidType)?,
        *iter
            .next()
            .ok_or(rusqlite::types::FromSqlError::InvalidType)?,
        *iter
            .next()
            .ok_or(rusqlite::types::FromSqlError::InvalidType)?,
    ];

    let value = if is_big_endian {
        f64::from_be_bytes(bytes)
    } else {
        f64::from_le_bytes(bytes)
    };

    Ok(value)
}

fn read_u32<'a, I>(iter: &mut I, is_big_endian: bool) -> rusqlite::types::FromSqlResult<u32>
where
    I: Iterator<Item = &'a u8>,
{
    let bytes: [u8; 4] = [
        *iter
            .next()
            .ok_or(rusqlite::types::FromSqlError::InvalidType)?,
        *iter
            .next()
            .ok_or(rusqlite::types::FromSqlError::InvalidType)?,
        *iter
            .next()
            .ok_or(rusqlite::types::FromSqlError::InvalidType)?,
        *iter
            .next()
            .ok_or(rusqlite::types::FromSqlError::InvalidType)?,
    ];
    let value = if is_big_endian {
        u32::from_be_bytes(bytes)
    } else {
        u32::from_le_bytes(bytes)
    };
    Ok(value)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JunctionSpatialIndex {
    pub junction_id: JunctionId,
    pub location: Vec2,
}

impl RTreeObject for JunctionSpatialIndex {
    type Envelope = AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.location.x, self.location.y])
    }
}

impl PointDistance for JunctionSpatialIndex {
    fn distance_2(
        &self,
        point: &<Self::Envelope as rstar::Envelope>::Point,
    ) -> <<Self::Envelope as rstar::Envelope>::Point as rstar::Point>::Scalar {
        self.location
            .distance_squared(Vec2::new(point[0], point[1]))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoadSpatialIndex {
    pub id: RoadId, // Points to a road in the RoadMap
    #[serde(
        serialize_with = "serialize_aabb",
        deserialize_with = "deserialize_aabb"
    )]
    pub aabb: Aabb2d,
}

impl RTreeObject for RoadSpatialIndex {
    type Envelope = rstar::AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let min = self.aabb.min;
        let max = self.aabb.max;
        rstar::AABB::from_corners([min.x, min.y], [max.x, max.y])
    }
}

fn deserialize_aabb<'de, D>(deserializer: D) -> Result<Aabb2d, D::Error>
where
    D: Deserializer<'de>,
{
    let (min, max): (Vec2, Vec2) = Deserialize::deserialize(deserializer)?;

    Ok(Aabb2d { min, max })
}

fn serialize_aabb<S>(aabb: &Aabb2d, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let min = aabb.min;
    let max = aabb.max;

    (min, max).serialize(serializer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstar::RTree;

    #[test]
    fn test_junction_spatial_index() {
        let junction = JunctionSpatialIndex {
            junction_id: JunctionId(0),
            location: Vec2::new(0.0, 0.0),
        };

        let junction2 = JunctionSpatialIndex {
            junction_id: JunctionId(1),
            location: Vec2::new(1.0, 1.0),
        };

        let junction3 = JunctionSpatialIndex {
            junction_id: JunctionId(2),
            location: Vec2::new(2.0, 2.0),
        };

        let mut rtree = RTree::new();
        rtree.insert(junction);
        rtree.insert(junction2);
        rtree.insert(junction3);

        let nearest = rtree.nearest_neighbor(&[0.1, 0.1]);
        assert_eq!(nearest.unwrap().junction_id, JunctionId(0));
    }

    #[test]
    fn test_road_spatial_index() {
        let road = RoadSpatialIndex {
            id: RoadId(0),
            aabb: Aabb2d {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(1.0, 1.0),
            },
        }
        .envelope();

        let road2 = RoadSpatialIndex {
            id: RoadId(1),
            aabb: Aabb2d {
                min: Vec2::new(1.0, 1.0),
                max: Vec2::new(2.0, 2.0),
            },
        }
        .envelope();

        let road3 = RoadSpatialIndex {
            id: RoadId(2),
            aabb: Aabb2d {
                min: Vec2::new(2.0, 2.0),
                max: Vec2::new(3.0, 3.0),
            },
        }
        .envelope();

        assert_eq!(road, rstar::AABB::from_corners([0.0, 0.0], [1.0, 1.0]));
        assert_eq!(road2, rstar::AABB::from_corners([1.0, 1.0], [2.0, 2.0]));
        assert_eq!(road3, rstar::AABB::from_corners([2.0, 2.0], [3.0, 3.0]));
    }

    #[test]
    fn serialize_road_section() {
        let road_section = RoadSection {
            points: vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)],
            aabb: Aabb2d {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(1.0, 1.0),
            },
        };

        let serialized = serde_json::to_string(&road_section).unwrap();

        assert_eq!(
            serialized,
            "{\"points\":[[0.0,0.0],[1.0,1.0]],\"aabb\":[[0.0,0.0,0.0],[1.0,1.0,1.0]]}"
        );
    }

    #[test]
    fn deserialize_road_section() {
        let serialized =
            "{\"points\":[[0.0,0.0],[1.0,1.0]],\"aabb\":[[0.0,0.0,0.0],[1.0,1.0,1.0]]}";
        let road_section: RoadSection = serde_json::from_str(serialized).unwrap();

        assert_eq!(
            road_section.points,
            vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)]
        );
        assert_eq!(
            road_section.aabb,
            Aabb2d {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(1.0, 1.0)
            }
        );
    }
}
