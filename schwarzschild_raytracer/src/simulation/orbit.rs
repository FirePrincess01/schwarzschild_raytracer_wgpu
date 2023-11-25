use glam::*;
    use super::polar_transformations::*;

pub struct Orbit{
    schwarz_r: f64,
    start_phi: f64,
    tilt_angle: f64,
    orbit_angle: f64,
    plane_tilt_mat: DMat3,
    energy: f64,
    rotation: f64, //rotational momentum
    r: f64,
    // ODE variables
    u: f64,
    u_bar: f64,
    last_r: f64, //for central falling case
    has_hit_singularity: bool,
}

impl Orbit {
    pub fn new(schwarz_r: f64, position: 
        DVec3, desired_direction: DVec3, mut rotation: f64) -> Option<Self> {
        let r = position.length();
        //Cant start orbits within the Black Hole
        if r <= schwarz_r{
            return None;
        }
        // Central falling case
        if rotation < schwarz_r * 1e-5 as f64 {
            rotation = 0.;
        }

        let u = 1./r;
        let energy = ((1. - schwarz_r / r) * (1. + rotation * rotation / (r * r))).sqrt();
        let plane_normal = position.cross(desired_direction);
        let mut tilt_angle = plane_normal.angle_between(DVec3::Z);

        let mut start_phi;
        let mut orbit_angle;
        let mut plane_tilt_mat;
        let pos_phi = f64::atan2(position.y, position.x);

        if tilt_angle < 1e-10 as f64 || std::f64::consts::PI - tilt_angle < 1e-10 as f64 {
            tilt_angle = 0.;
            start_phi = 0.;
            orbit_angle = pos_phi;
            plane_tilt_mat = DMat3::IDENTITY;
        }
        else {
            let horizontal_cut = DVec3::Z.cross(plane_normal);
            orbit_angle = horizontal_cut.angle_between(position);
            //In case the angle is supposed to be negative.
            if position.z < 0. {
                orbit_angle = std::f64::consts::TAU - orbit_angle;
            }
            start_phi = f64::atan2(horizontal_cut.y, horizontal_cut.x);
            plane_tilt_mat = DMat3::from_rotation_x(tilt_angle);
        }

        Some(Self{
            schwarz_r,
            start_phi,
            tilt_angle,
            orbit_angle,
            plane_tilt_mat,
            energy,
            rotation,
            r,
            u,
            u_bar: 0.,
            last_r: r,
            has_hit_singularity: false,
        })
    }

    pub fn do_step(&mut self, time_step: f64) {
        // The singularity is the end of time
        if self.has_hit_singularity{
            return;
        }

        //Maybe rework this, Verlet doesnt work with dynamic time steps
        if self.rotation == 0. {
            let next_r = 2. * self.r - self.last_r - time_step * time_step * self.schwarz_r / (2. * self.r * self.r);
            if next_r < 0. {
                self.has_hit_singularity = true;
            }
            else {
                self.last_r = self.r;
                self.r = next_r;
            }
            return;
        }

        let l = self.rotation;
        let u = self.u;
        let u_bar = self.u_bar;

		//Time stepping variables
        let mut delta_phi: f64;
        let mut next_u: f64;

        //Approximate the angle step according to the time step
		//These are iterated updates, because phi and u are interdependent
		//Errors in deltaPhi only influence the simulation speed, whereas errors in u can cause orbit decay.
        delta_phi = time_step * l * u * u / 2.;
		next_u = u + delta_phi * u_bar;
		delta_phi = time_step * l / 4. * (u * u + next_u * next_u);
		next_u = u + delta_phi * u_bar;
		delta_phi = time_step * l / 4. * (u * u + next_u * next_u);
		next_u = u + delta_phi * u_bar;
		delta_phi = time_step * l / 4. * (u * u + next_u * next_u);

        //Break down into smaller steps if the step size is too large
		let mut step_fragments:u32 = (1 + ((delta_phi * 100.).floor() as u32)).min(1000);

        for i in 0..step_fragments {
            self.do_angle_step(delta_phi / step_fragments as f64);
            if self.has_hit_singularity {
                return;
            }
        }
    }

    pub fn do_angle_step(&mut self, delta_phi: f64) {   
        let l = self.rotation;
        let u = self.u;
        let u_bar = self.u_bar;
        let schwarz_r = self.schwarz_r;

        //Runge Kutta 4 scheme
        let a_u = u + delta_phi / 2. * u_bar;
        let a_u_bar = u_bar + delta_phi / 2. * (schwarz_r * (1. / (2. * l *l) + 3. / 2. * u * u) - u);
        let b_u = u + delta_phi / 2. * a_u_bar;
        let b_u_bar = u_bar + delta_phi / 2. * (schwarz_r * (1. / (2. * l *l) + 3. / 2. * a_u * a_u) - a_u);
        let c_u = u + delta_phi * b_u_bar;
        let c_u_bar = u_bar + delta_phi * (schwarz_r * (1. / (2. * l *l) + 3. / 2. * b_u * b_u) - b_u);
        let next_u = u + delta_phi * (u_bar / 6. + a_u_bar / 3. + b_u_bar / 3. + c_u_bar / 6.);
        let next_u_bar = u_bar + delta_phi * (schwarz_r / (2. * l*l) +
            3. * schwarz_r / 2. * (u * u / 6. + a_u * a_u / 3. + b_u * b_u / 3. + c_u * c_u / 6.)
            - (u + 2. * a_u + 2. * b_u + c_u) / 6.);

        
        self.u = next_u;
        self.u_bar = next_u_bar;
        if self.u.is_infinite() || self.u > 1e5{
            self.has_hit_singularity = true;
        }
        else {
            self.r = 1. / self.u;
            self.orbit_angle += delta_phi;
        }
        
    }

    fn h_r(&self) -> f64 {
        return 1. - self.schwarz_r / self.r;
    }

    // Returns the current position in carthesic coordinates
    pub fn get_position(&self) -> DVec3 {
        let mut polar_pos = DVec3::ZERO;
        polar_pos.x = self.r;
        polar_pos.y = self.orbit_angle;
        polar_pos.z = 0.;
        polar_pos = trans_polar_vec(polar_pos, self.plane_tilt_mat);
        polar_pos.y += self.start_phi;
        return polar_to_carthesic(polar_pos);
    }

    // Returns the spectator in terms of (t,r,phi)
    pub fn get_velocity(&self) -> DVec3 {
        let mut spectator = DVec3::ZERO;
        let falling: f64;
        if self.rotation == 0. {
            falling = -(self.r - self.last_r).signum();
        }
        else {
            falling = self.u_bar.signum();
        }
        spectator.x = self.energy / self.h_r();
        spectator.y = - falling * f64::sqrt(self.energy * self.energy - self.h_r() * (1. + self.rotation * self.rotation / (self.r * self.r)));
        spectator.z = self.rotation / (self.r * self.r);
        return spectator;
    }

    pub fn current_tilt_angle(&self) -> f64 {
        return self.tilt_angle * (self.get_position().y - self.start_phi).cos();
    }

    /// Calculates wether the Orbit is stable, instable (falls into Black hole), or on an escape trajectory.
    /// Todo: use enum as return value
    pub fn is_stable(&mut self) -> bool {
        if self.rotation.powi(2) < 3. * self.schwarz_r.powi(2) {
            return false; //instable
        }
        let r1 = self.rotation.powi(2) / self.schwarz_r * (1. - f64::sqrt(1. - 3. * self.schwarz_r.powi(2) / self.rotation.powi(2)));
        if self.r < r1 {
            return false; //"instable";
        }
        if self.energy > 1. {
            return false;//"escaping";
        }
        if self.energy.powi(2) - (1. - self.schwarz_r / r1) * (self.rotation.powi(2) / r1.powi(2) + 1.) < 0. {
            return true; //"stable";
        }
        return false; //"instable";
    }

    pub fn is_singular(&self) -> bool {
        return self.has_hit_singularity;
    }

    pub fn is_central_fall(&self) -> bool {
        return self.rotation == 0.;
    }
}
