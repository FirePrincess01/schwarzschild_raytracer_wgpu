use glam::{Vec3, Vec4};

const NR_NODES: usize = 30; //needs to be at least 3
const SMALLEST_ANGLE: f32 = 0.05;

pub struct RayConnector {
    schwarz_r: f32,
    pos: Vec3,
    last_phi: f32,
    less_than_180: bool,
    needs_reset: bool,
    u_ray: [f32; NR_NODES],
}

impl RayConnector {
    pub fn new(schwarz_r: f32, pos: Vec3, less_than_180: bool) -> Self {
        Self {
            schwarz_r,
            pos,
            last_phi: 1.,
            less_than_180,
            needs_reset: true,
            u_ray: [1.; NR_NODES],
        }
    }

    pub fn reset_ray(&mut self, other_position: Vec3) -> [f32; 4] {
        self.needs_reset = false;
        let u0 = 1. / other_position.length();
        let u1 = 1. / self.pos.length();
        self.last_phi = self.pos.angle_between(other_position);
        if !self.less_than_180 {
            self.last_phi = std::f32::consts::TAU - self.last_phi;
        }

        // A robust and fast initial guess
        for i in 0..NR_NODES {
            let weight = i as f32 / (NR_NODES as f32 - 1.);
            self.u_ray[i] = u0 * (1. - weight) + u1 * weight;
        }

        return self.update_ray(other_position, 5);
    }

    // Updates the ray and calculates the current incoming angle for other_position
    // The output is packed as [current_position, incoming_angle]
    pub fn update_ray(&mut self, other_position: Vec3, iterations: usize) -> [f32; 4] {
        if self.needs_reset {
            return self.reset_ray(other_position);
        }
        
        self.last_phi = self.pos.angle_between(other_position);
        if !self.less_than_180 {
            self.last_phi = std::f32::consts::TAU - self.last_phi;
        }

        // If the angle is too small, the ray will follow a mostly straight path
        // Furthermore calculation would be unstable
        // Because u_ray isnt updated, it will need a reset later on
        if self.last_phi < SMALLEST_ANGLE {
            self.needs_reset = true;
            let incoming_angle: f32;
            if self.last_phi == 0. {
                incoming_angle = if other_position.length() > self.pos.length() {0.} else {std::f32::consts::PI};
            }
            else {
                let u_bar = (1. / self.pos.length() - 1. / other_position.length()) / self.last_phi;
                incoming_angle = self.calc_ray_angle(u_bar, other_position.length());
            }
            // No euclidian geometry allowed!
            //let incoming_angle = Vec3::angle_between(other_position - self.pos, - self.pos) 
            //    * if self.less_than_180 {1.} else {-1.};
            return [self.pos.x, self.pos.y, self.pos.z, incoming_angle];
        }

        let u0 = 1. / other_position.length();
        let u1 = 1. / self.pos.length();
        
        // Need to update the amount both points moved
        let u0_delta = u0 - self.u_ray[0];
        let u1_delta = u1 - self.u_ray[NR_NODES-1];
        for i in 0..NR_NODES {
            let weight = i as f32 / (NR_NODES as f32 - 1.); // is compiler optimization enough here?
            self.u_ray[i] += u0_delta * (1. - weight) + u1_delta * weight;
        }

        // Newton iterations
        // M_h = Stiffness Matrix - I      for u'' + u
        // Solving (M_h + 3*R*diag(u_h))^-1 * (M_h * u_h + 3R/2 u_h^2) with fixed boundaries
        let mut residual: [f32; NR_NODES - 2] = [0.; NR_NODES - 2]; //Corresponds to u_ray without boundary
        let mut thomas_c: [f32; NR_NODES - 2] = [0.; NR_NODES - 2];
        let scale =  (NR_NODES * NR_NODES) as f32 / (self.last_phi * self.last_phi);
        for _k in 0..iterations {
            // index shifted to make the stencil more clear
            for i in 1..(NR_NODES - 1) {
                residual[i-1] = scale * (-self.u_ray[i-1] + 2. * self.u_ray[i] - self.u_ray[i+1]) - self.u_ray[i] //M_h * u_h
                    + 3. * self.schwarz_r / 2. * self.u_ray[i] * self.u_ray[i];
            }

            // Thomas algorithm to solve: (M_h + 3*R*diag(u_h)) * z_h = residual
            // Off-diagonals are all (-scale)
            let main_diag_inv = 1. / (2. * scale - 1. + self.u_ray[1]);
            thomas_c[0] = (-scale) * main_diag_inv;
            residual[0] = residual[0] * main_diag_inv;
            for i in 1..(NR_NODES - 2) {
                let main_diag_inv = 1. / (2. * scale - 1. + self.u_ray[i+1] + scale * thomas_c[i-1]);
                thomas_c[i] = (-scale) * main_diag_inv;
                residual[i] = (residual[i] + scale * residual[i-1]) * main_diag_inv;
            }

            // Back substitution and applying the correction to u_ray
            // Did put this into one to save a for loop
            self.u_ray[NR_NODES-1] -= residual[NR_NODES-3];
            for i in (0..(NR_NODES - 3)).rev() {
                residual[i] = residual[i] - thomas_c[i] * residual[i+1];
                self.u_ray[i+1] -= residual[i];
            }
        }

        //Time to calculate the angle
        let u_bar = (self.u_ray[1] - self.u_ray[0]) * NR_NODES as f32 / self.last_phi;
        let incoming_angle = self.calc_ray_angle(u_bar, other_position.length());
        return [self.pos.x, self.pos.y, self.pos.z, incoming_angle];
    }

    // Calculates the angle perceived by the frozen observer at radius r
    // between a ray with inverse derivitive u_bar and the the center of the black hole
    // negative angles represent rays traveling the long way around the black hole
    fn calc_ray_angle(&self, u_bar: f32, r: f32) -> f32 {
        let theta: f32;
        if r > self.schwarz_r {
            theta = u_bar.signum() * f32::acos(f32::sqrt(1. / (1. + (r * r * u_bar * u_bar) / ( 1. - self.schwarz_r / r))));
        }
        else {
            theta = todo!();
        }
        let special_angle_format = (std::f32::consts::FRAC_PI_2 - theta) * if self.less_than_180 {1.} else {-1.};
        return special_angle_format;
    }
}