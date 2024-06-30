use bevy::{
    math::{Vec2, Vec3},
    render::primitives::Aabb,
};
use rstar::{PointDistance, RTreeObject, AABB};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{JunctionId, RoadId};

/// A road section, with id, points and bounding box
#[derive(Serialize, Deserialize, Debug)]
pub struct RoadSection {
    pub id: RoadId,
    pub points: Vec<Vec2>,
    #[serde(
        serialize_with = "serialize_aabb",
        deserialize_with = "deserialize_aabb"
    )]
    pub aabb: Aabb,
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
    pub aabb: Aabb,
}

impl RTreeObject for RoadSpatialIndex {
    type Envelope = rstar::AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let min = self.aabb.min();
        let max = self.aabb.max();
        rstar::AABB::from_corners([min.x, min.y], [max.x, max.y])
    }
}

fn deserialize_aabb<'de, D>(deserializer: D) -> Result<Aabb, D::Error>
where
    D: Deserializer<'de>,
{
    let (min, max): (Vec3, Vec3) = Deserialize::deserialize(deserializer)?;

    Ok(Aabb::from_min_max(min, max))
}

fn serialize_aabb<S>(aabb: &Aabb, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let min = aabb.min();
    let max = aabb.max();

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
            aabb: Aabb::from_min_max(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
        }
        .envelope();

        let road2 = RoadSpatialIndex {
            id: RoadId(1),
            aabb: Aabb::from_min_max(Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0)),
        }
        .envelope();

        let road3 = RoadSpatialIndex {
            id: RoadId(2),
            aabb: Aabb::from_min_max(Vec3::new(2.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0)),
        }
        .envelope();

        assert_eq!(road, rstar::AABB::from_corners([0.0, 0.0], [1.0, 1.0]));
        assert_eq!(road2, rstar::AABB::from_corners([1.0, 1.0], [2.0, 2.0]));
        assert_eq!(road3, rstar::AABB::from_corners([2.0, 2.0], [3.0, 3.0]));
    }

    #[test]
    fn serialize_road_section() {
        let road_section = RoadSection {
            id: RoadId(0),
            points: vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)],
            aabb: Aabb::from_min_max(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
        };

        let serialized = serde_json::to_string(&road_section).unwrap();

        assert_eq!(
            serialized,
            "{\"id\":0,\"points\":[[0.0,0.0],[1.0,1.0]],\"aabb\":[[0.0,0.0,0.0],[1.0,1.0,1.0]]}"
        );
    }

    #[test]
    fn deserialize_road_section() {
        let serialized =
            "{\"id\":0,\"points\":[[0.0,0.0],[1.0,1.0]],\"aabb\":[[0.0,0.0,0.0],[1.0,1.0,1.0]]}";
        let road_section: RoadSection = serde_json::from_str(serialized).unwrap();

        assert_eq!(road_section.id, RoadId(0));
        assert_eq!(
            road_section.points,
            vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)]
        );
        assert_eq!(
            road_section.aabb,
            Aabb::from_min_max(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0))
        );
    }
}
