use glam::{Vec3, DVec3};
use crate::simulation::{ray_connector::RayConnector, orbit::Orbit};
use super::vertex::Vertex;



pub struct PointCloud {
    points: Vec<RayConnector>,
    points_farside: Vec<RayConnector>,
    orbits: Vec<Orbit>,
    vertices: Vec<Vertex>,
    vertices_farside: Vec<Vertex>,
    schwarz_r: f32,
    has_farside: bool,
    has_orbits: bool,
    rng: fastrand::Rng,
}

impl PointCloud {
    pub fn new(model_vertices: &[Vec3], schwarz_r: f32, observer_pos: Vec3, activate_farside: bool, activate_orbits: bool) -> Self { 
        let size = model_vertices.len();
        let mut points: Vec<RayConnector> = Vec::new();
        let mut points_farside: Vec<RayConnector> = Vec::new();
        let mut orbits: Vec<Orbit> = Vec::new();
        points.reserve(size);
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut vertices_farside: Vec<Vertex> = Vec::new();
        vertices.reserve(size);

        if activate_farside {
            points_farside.reserve(size);
            vertices_farside.reserve(size);
        }

        if activate_orbits {
            orbits.reserve(size);
        }
        let mut rng = fastrand::Rng::new();


        for i in 0..size {
            points.push(RayConnector::new(schwarz_r, model_vertices[i], true));
            vertices.push(Vertex{
                position: [model_vertices[i].x, model_vertices[i].y, model_vertices[i].z, points[i].reset_ray(observer_pos)]});
            if activate_farside {
                points_farside.push(RayConnector::new(schwarz_r, model_vertices[i], false));
                vertices_farside.push(Vertex{position: [model_vertices[i].x, model_vertices[i].y, model_vertices[i].z, points_farside[i].reset_ray(observer_pos)]});
            }
            if activate_orbits {
                let pos = model_vertices[i].as_dvec3();
                orbits.push(Orbit::new(schwarz_r as f64, pos, DVec3::new(-pos.y, pos.x, 0.), 18. + 2. * rng.f64()).unwrap());
            }
        }

        Self {
            points,
            points_farside,
            orbits,
            vertices,
            vertices_farside,
            schwarz_r,
            has_farside: activate_farside,
            has_orbits: activate_orbits,
            rng,
        } 
    }

    #[allow(dead_code)]
    pub fn new_spiral(schwarz_r: f32, observer_pos: Vec3, activate_farside: bool) -> Self {
        const NR_POINTS: usize = 10000;
        let mut points: Vec<Vec3> = Vec::new();
        points.reserve(NR_POINTS);

        for i in 0..NR_POINTS {
            let t = i as f32 / NR_POINTS as f32 * (std::f32::consts::TAU + 0.05);
            let r = 16. + 2. * t;
            let pos = Vec3::new(-r * (10. * t).cos(), -r * (10. * t).sin(), 0.001);
            points.push(pos);
        }

        return Self::new(&points, schwarz_r, observer_pos, activate_farside, false)
    }

    #[allow(dead_code)]
    pub fn new_accretion_disk(schwarz_r: f32, observer_pos: Vec3, activate_farside: bool) -> Self {
        const NR_POINTS: usize = 5000;
        let mut points: Vec<Vec3> = Vec::new();
        points.reserve(NR_POINTS);
        let mut rng = fastrand::Rng::new();

        for _ in 0..NR_POINTS {
            let r = 2. * schwarz_r as f64 + 2. * schwarz_r as f64 * rng.f64();
            let phi = rng.f64() * std::f64::consts::TAU;
            let theta = 0.2 * (rng.f64() - 0.5);
            let pos = crate::simulation::polar_transformations::polar_to_carthesic(DVec3::new(r, phi, theta));
            points.push(pos.as_vec3());
        }

        return Self::new(&points, schwarz_r, observer_pos, activate_farside, true)
    }

    #[allow(dead_code)]
    pub fn new_heart(schwarz_r: f32, observer_pos: Vec3, activate_farside: bool) -> Self {
        const NR_POINTS: usize = 400;
        let mut points: Vec<Vec3> = Vec::new();
        points.reserve(NR_POINTS);

        for i in 0..NR_POINTS {
            let t = i as f32 / NR_POINTS as f32 * std::f32::consts::TAU;
            let pos = Vec3::new(11., 16. * t.sin().powi(3),
               13. * t.cos() - 5. * (2. * t).cos() - 2. *  (3. * t).cos() - (4. * t).cos());
            points.push(pos);
        }

        return Self::new(&points, schwarz_r, observer_pos, activate_farside, false)
    }

    pub fn update(&mut self, observer_pos: Vec3, dt: instant::Duration) {
        for i in 0..(self.points.len()) {
            if self.has_orbits {
                self.orbits[i].do_step(dt.as_secs_f64());
                let orbit_pos = self.orbits[i].get_position().as_vec3();

                if self.orbits[i].is_singular() || orbit_pos.length_squared() <= self.schwarz_r * self.schwarz_r {
                    let r = 16. + 10. * self.rng.f64();
                    let phi = self.rng.f64() * std::f64::consts::TAU;
                    let theta = 0.2 * (self.rng.f64() - 0.5);
                    let pos = crate::simulation::polar_transformations::polar_to_carthesic(DVec3::new(r, phi, theta));
                    self.orbits[i] = Orbit::new(self.schwarz_r as f64, pos, DVec3::new(-pos.y, pos.x, 0.), 18. + 2. * self.rng.f64()).unwrap();
                    self.points[i].set_position(self.orbits[i].get_position().as_vec3());
                    self.points[i].reset_ray(observer_pos);
                    if self.has_farside {
                        self.points_farside[i].set_position(self.orbits[i].get_position().as_vec3());
                        self.points_farside[i].reset_ray(observer_pos);
                    }
                }

                self.points[i].set_position(orbit_pos);
                self.vertices[i].position[0..3].copy_from_slice(& [orbit_pos.x, orbit_pos.y, orbit_pos.z]);
                if self.has_farside {
                    self.points_farside[i].set_position(orbit_pos);
                    self.vertices_farside[i].position[0..3].copy_from_slice(& [orbit_pos.x, orbit_pos.y, orbit_pos.z]);
                }
            }

            self.vertices[i].position[3] = self.points[i].update_ray(observer_pos, 1);
            if self.has_farside {
                self.vertices_farside[i].position[3] = self.points_farside[i].update_ray(observer_pos, 1);
            }
        }
    }

    pub fn get_vertices(&self) -> &[Vertex] {
        return &self.vertices;
    }

    pub fn get_vertices_farside(&self) -> &[Vertex] {
        return &self.vertices_farside;
    }

    pub fn has_farside(&self) -> bool {
        self.has_farside
    }
}