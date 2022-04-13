use bevy::{
    math::{Vec2, Vec3},
    render::primitives::Aabb,
};
use rstar::{PointDistance, RTreeObject, AABB};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize, Debug)]
pub struct RoadSection {
    pub id: usize,
    pub points: Vec<Vec2>,
    #[serde(
        serialize_with = "serialize_aabb",
        deserialize_with = "deserialize_aabb"
    )]
    pub aabb: Aabb,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct RoadSpatialIndex {
    pub id: usize, // Points to a road in the RoadMap
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

#[derive(Serialize, Deserialize, Debug)]
pub struct JunctionSpatialIndex {
    pub junction_id: usize,
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
