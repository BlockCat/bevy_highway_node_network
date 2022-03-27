use bevy::math::{Vec2, Vec3};

pub struct RijkDriehoekCoordinate(pub f32, pub f32);

impl From<RijkDriehoekCoordinate> for Vec2 {
    fn from(rdc: RijkDriehoekCoordinate) -> Self {
        Vec2::new(rdc.0, rdc.1)
    }
}

impl From<RijkDriehoekCoordinate> for Vec3 {
    fn from(rdc: RijkDriehoekCoordinate) -> Self {
        Vec3::new(rdc.0, rdc.1, 0.0)
    }
}

impl From<RijkDriehoekCoordinate> for WGS84 {
    fn from(rdc: RijkDriehoekCoordinate) -> Self {
        // Amersfoort
        let reference_rd_x = 155000.0;
        let reference_rd_y = 463000.0;

        let dx = (rdc.0 - reference_rd_x) * 0.00001;
        let dy = (rdc.1 - reference_rd_y) * 0.00001;

        let reference_wgs84_x = 52.15517;
        let reference_wgs84_y = 5.387206;

        let sum_n = (3235.65389 * dy)
            + (-32.58297 * dx.powi(2))
            + (-0.2475 * dy.powi(2))
            + (-0.84978 * dx.powi(2) * dy)
            + (-0.0655 * dy.powi(3))
            + (-0.01709 * dx.powi(2) * dy.powi(2))
            + (-0.00738 * dx)
            + (0.0053 * dx.powi(4))
            + (-0.00039 * dx.powi(2) * dy.powi(3))
            + (0.00033 * dx.powi(4) * dy)
            + (-0.00012 * dx * dy);

        let sum_e = (5260.52916 * dx)
            + (105.94684 * dx * dy)
            + (2.45656 * dx * dy.powi(2))
            + (-0.81885 * dx.powi(3))
            + (0.05594 * dx * dy.powi(3))
            + (-0.05607 * dx.powi(3) * dy)
            + (0.01199 * dy)
            + (-0.00256 * dx.powi(3) * dy.powi(2))
            + (0.00128 * dx * dy.powi(4))
            + (0.00022 * dy.powi(2))
            + (-0.00022 * dx.powi(2))
            + (0.00026 * dx.powi(5));

        let longitude = reference_wgs84_y + (sum_e / 3600.0);
        let latitude = reference_wgs84_x + (sum_n / 3600.0);

        Self {
            longitude,
            latitude,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WGS84 {
    pub longitude: f32,
    pub latitude: f32,
}

impl From<WGS84> for RijkDriehoekCoordinate {
    fn from(ltlt: WGS84) -> Self {
        let reference_rd_x = 155000.0;
        let reference_rd_y = 463000.0;

        let reference_wgs84_x = 52.15517;
        let reference_wgs84_y = 5.387206;

        let d_lattitude = 0.36 * (ltlt.latitude - reference_wgs84_x);
        let d_longitude = 0.36 * (ltlt.longitude - reference_wgs84_y);

        // let r_matrix = [
        //     [0.0, -0.705, 0.0, 0.0],
        //     [190094.945, -11832.228, -114.221, 0.0],
        //     [0.0; 4],
        //     [-32.391, -2.340, 0.0, 0.0],
        //     [0.0; 4],
        // ];

        let gen = |p, q| d_lattitude.powi(p) * d_longitude.powi(q);

        let x_transformations = [
            190094.945, -114.221, -11832.228, -0.705, -32.391, -2.340, -0.008, -0.608, 0.148,
        ];

        let x_values = [
            gen(0, 1),
            gen(1, 1),
            gen(2, 1),
            gen(0, 3),
            gen(1, 0),
            gen(3, 1),
            gen(0, 2),
            gen(1, 3),
            gen(2, 3),
        ];

        let y_transformations = [
            0.433, 3638.893, 0.092, 309056.544, 73.077, -157.984, 59.788, -6.439, -0.032, -0.054,
        ];

        let y_values = [
            gen(0, 1),
            gen(0, 2),
            gen(0, 4),
            gen(1, 0),
            gen(2, 0),
            gen(1, 2),
            gen(3, 0),
            gen(2, 2),
            gen(1, 1),
            gen(1, 4),
        ];

        let calc_latt: f32 = x_transformations
            .into_iter()
            .zip(x_values.into_iter())
            .map(|(a, b)| a * b)
            .sum();
        let calc_long: f32 = y_transformations
            .into_iter()
            .zip(y_values.into_iter())
            .map(|(a, b)| a * b)
            .sum();

        let rd_x_coordinate = reference_rd_x + calc_latt;
        let rd_y_coordinate = reference_rd_y + calc_long;

        return RijkDriehoekCoordinate(rd_x_coordinate, rd_y_coordinate);
    }
}

impl From<WGS84> for Vec2 {
    fn from(wgs: WGS84) -> Self {
        Vec2::new(wgs.latitude, wgs.longitude)
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use super::{RijkDriehoekCoordinate, WGS84};

    #[test]
    fn long_lat_to_rijk() {
        let rijk = RijkDriehoekCoordinate(136239.0, 456184.0);
        let ll = WGS84 {
            longitude: 5.113435,
            latitude: 52.093597,
        };

        let rijk_c = RijkDriehoekCoordinate::from(ll);

        let a = Vec2::from(rijk);
        let b = Vec2::from(rijk_c);

        assert!(dbg!(a.distance(b)) <= 30.0);
    }

    #[test]
    fn rijk_to_long_lat() {
        let rijk = RijkDriehoekCoordinate(136239.0, 456184.0);
        let ll = WGS84 {
            longitude: 5.113435,
            latitude: 52.093597,
        };

        let ll_c = WGS84::from(rijk);

        let a = Vec2::from(ll);
        let b = Vec2::from(ll_c);

        // println!("Expected: {:?} -> Got: {:?}", ll, ll_c);

        assert!(dbg!(a.distance(b)) <= 0.00001);
    }
}
