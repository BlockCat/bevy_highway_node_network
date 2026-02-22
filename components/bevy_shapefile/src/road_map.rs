use crate::{spatial::RoadSection, JunctionId, RoadId};
use bevy::{math::bounding::Aabb2d, prelude::Resource};
use rstar::{RStarInsertionStrategy, RTreeParams};

use rusqlite::{CachedStatement, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, ops::Deref};

/// Load shapefile.
/// This shapefile is used vor visualization of road data.
/// It loads all the road sections, and puts it in spatial data structures.
// #[derive(Serialize, Deserialize, Debug, Resource)]
// pub struct RoadMap {
//     roads: HashMap<RoadId, RoadSection>,
//     junction_spatial: rstar::RTree<JunctionSpatialIndex, Params>,
//     road_spatial: rstar::RTree<RoadSpatialIndex, Params>,
// }

#[derive(Debug)]
struct ResourceConnection(Connection);

unsafe impl Sync for ResourceConnection {}
unsafe impl Send for ResourceConnection {}

impl Deref for ResourceConnection {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Resource)]
pub struct RoadMap {
    connection: ResourceConnection,
}

impl RoadMap {
    pub fn new(connection: &str) -> Self {
        let connection = Connection::open(connection).expect("Could not open connection");
        unsafe {
            connection.load_extension_enable().unwrap();
            connection
                .load_extension("mod_spatialite", None::<&str>)
                .unwrap();
        }

        // connection
        //     .execute(
        //         "SELECT CreateMissingSystemTables(1);",
        //         [],
        //     )
        //     .unwrap();

        Self {
            connection: ResourceConnection(connection),
        }
    }

    pub fn road_length(&self, road_id: RoadId) -> f32 {
        // https://www.gaia-gis.it/gaia-sins/spatialite-sql-4.2.1.html#length_cvt
        let mut statement = self
            .connection
            .prepare("SELECT ST_Length(geom) FROM Wegvakken WHERE id = ?")
            .expect("Could not prepare statement");

        statement
            .query_row([road_id.num() as i64], |row| row.get(0))
            .expect("Could not query length")
    }

    pub fn load_roads_in_aabb2d(&self, aabb: &Aabb2d) -> HashSet<RoadId> {
        let min_x = aabb.min.x as f64;
        let min_y = aabb.min.y as f64;
        let max_x = aabb.max.x as f64;
        let max_y = aabb.max.y as f64;

        let mut statement = self
            .connection
            .prepare(
                "SELECT id FROM rtree_Wegvakken_geom WHERE minx >= ? AND miny >= ? AND maxx <= ? AND maxy <= ?",
            )
            .expect("Could not prepare statement");

        statement
            .query_map([min_x, min_y, max_x, max_y], |row| {
                let id: i64 = row.get(0)?;
                Ok(RoadId::from(id as usize))
            })
            .expect("Could not query roads")
            .map(|x| x.unwrap())
            .collect()
    }

    pub fn nearest_junction(&self, x: f32, y: f32) -> Option<JunctionId> {
        // let result = self
        //     .connection
        //     .query_one("
        //     SELECT

        //     ", [x, y], |row| row.get::<usize, JunctionId>(0))
        //     .optional()
        //     .unwrap();

        // result
        None
    }
    pub fn get_section(&self, road_id: &RoadId) -> Option<RoadSection> {
        self.connection
            .query_one(
                "SELECT geom FROM Wegvakken WHERE id = ?",
                [road_id.num() as i64],
                |row| {
                    let section: RoadSection = row.get(0)?;
                    Ok(section)
                },
            )
            .ok()
    }

    pub fn roads<'a>(&'a self) -> Vec<(RoadId, RoadSection)> {
        let mut statement: CachedStatement<'a> = self
            .connection
            .prepare_cached("SELECT id, geom FROM Wegvakken")
            .expect("Could not prepare statement");

        statement
            .query_map((), |row| {
                let id: RoadId = row.get(0)?;
                let section: RoadSection = row.get(1)?;
                Ok((id, section))
            })
            .unwrap()
            .map(|f| f.unwrap())
            .collect()
    }
}

// impl RoadMap {
//     pub fn road_length(&self, road_id: RoadId) -> f32 {
//         let section = &self.roads[&road_id];

//         section
//             .points
//             .windows(2)
//             .map(|points| points[0].distance(points[1]))
//             .sum()
//     }

//     pub fn roads(&self) -> &HashMap<RoadId, RoadSection> {
//         &self.roads
//     }

//     pub fn get_section(&self, road_id: &RoadId) -> Option<&RoadSection> {
//         self.roads.get(road_id)
//     }

//     pub fn load_roads_in_aabb2d(&self, aabb: &Aabb2d) -> HashSet<RoadId> {
//         let aabb = rstar::AABB::from_corners([aabb.min.x, aabb.min.y], [aabb.max.x, aabb.max.y]);

//         self.road_spatial
//             .locate_in_envelope_intersecting(&aabb)
//             .map(|x| x.id)
//             .collect()
//     }

//     pub fn nearest_junction(&self, x: f32, y: f32) -> Option<JunctionId> {
//         self.junction_spatial
//             .nearest_neighbor(&[x, y])
//             .map(|x| x.junction_id)
//     }

//     // fn road_length2(&self, road_id: RoadId) -> f32 {
//     // https://www.gaia-gis.it/gaia-sins/spatialite-sql-4.2.1.html#length_cvt
//     // SELECT ST_Length(geom) FROM Wegvakken WHERE id = ?;
//     // }
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct Params;

impl RTreeParams for Params {
    const MIN_SIZE: usize = 2;
    const MAX_SIZE: usize = 40;
    const REINSERTION_COUNT: usize = 1;
    type DefaultInsertionStrategy = RStarInsertionStrategy;
}

// impl RoadMap {
//     pub fn write<P: AsRef<Path>>(&self, path: P) {
//         let file = File::create(path).expect("Could not create file");
//         bincode::serialize_into(file, self).expect("Could not write");
//     }

//     pub fn read<P: AsRef<Path>>(path: P) -> Self {
//         let file = File::open(path).expect("Could not open file");
//         bincode::deserialize_from(file).expect("Could not deserialize")
//     }

//     /// Load data from a shapefile
//     pub fn from_shapefile<P: AsRef<Path>>(path: P) -> Result<Self, ShapeError> {
//         println!("Start read of road data");
//         let roads =
//             shapefile::read_as::<P, Polyline, Record>(path).map_err(|x| ShapeError::Shape(x))?;

//         println!("Loading junction data");
//         let junctions = load_junctions(&roads);

//         println!("Loading roads");
//         let roads = load_road_sections(roads);
//         println!("Finish loading roads");

//         println!("Creating spatial data");
//         let spatial_indeces = roads
//             .iter()
//             .map(|(id, section)| RoadSpatialIndex {
//                 id: *id,
//                 aabb: section.aabb.clone(),
//             })
//             .collect();

//         let junction_indeces = junctions
//             .into_iter()
//             .map(|x| JunctionSpatialIndex {
//                 junction_id: x.0,
//                 location: x.1,
//             })
//             .collect::<Vec<_>>();

//         println!("Inserting spatial indices");

//         let road_spatial: RTree<RoadSpatialIndex, Params> =
//             RTree::bulk_load_with_params(spatial_indeces);

//         let junction_spatial = RTree::bulk_load_with_params(junction_indeces);

//         println!("Created tree");

//         Ok(RoadMap {
//             roads,
//             road_spatial,
//             junction_spatial,
//         })
//     }
// }

// fn get_usize(record: &Record, name: &str) -> Option<usize> {
//     let value = record.get(name).unwrap();

//     if let FieldValue::Numeric(x) = value {
//         return x.map(|x| x as usize);
//     }
//     unreachable!();
// }

// /// Load junction point data
// fn load_junctions(roads: &Vec<(GenericPolyline<Point>, Record)>) -> HashMap<JunctionId, Vec2> {
//     roads
//         .par_iter()
//         .flat_map_iter(|(line, record)| {
//             let junction_start = get_usize(record, "JTE_ID_BEG").unwrap();
//             let junction_end = get_usize(record, "JTE_ID_END").unwrap();

//             let start = line.part(0).and_then(|p| p.first()).unwrap();
//             let end = line.part(0).and_then(|p| p.last()).unwrap();

//             [
//                 (
//                     JunctionId::from(junction_start),
//                     Vec2::new(start.x as f32, start.y as f32),
//                 ),
//                 (
//                     JunctionId::from(junction_end),
//                     Vec2::new(end.x as f32, end.y as f32),
//                 ),
//             ]
//         })
//         .collect::<HashMap<_, _>>()
// }

// /// Load road sections
// fn load_road_sections(
//     roads: Vec<(GenericPolyline<Point>, Record)>,
// ) -> HashMap<RoadId, RoadSection> {
//     let roads = roads
//         .into_par_iter()
//         .enumerate()
//         .map(|(id, (line, _))| {
//             assert!(line.parts().len() == 1);

//             let points = line
//                 .part(0)
//                 .expect("Could not get ?")
//                 .iter()
//                 .map(|point| Vec2::new(point.x as f32, point.y as f32))
//                 .collect::<Vec<_>>();

//             let bbox = line.bbox();
//             let aabb = Aabb2d {
//                 min: Vec2::new(bbox.x_range()[0] as f32, bbox.y_range()[0] as f32),
//                 max: Vec2::new(bbox.x_range()[1] as f32, bbox.y_range()[1] as f32),
//             };
//             let id = RoadId::from(id);

//             (id, RoadSection { points, aabb })
//         })
//         .collect::<HashMap<_, _>>();
//     roads
// }
