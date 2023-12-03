use glam::Vec3;

const NR_NODES: usize = 48; //needs to be at least 3
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

        // After 5 Newton iterations discretization error should dominate
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
                let u0 = other_position.length_recip();
                let u_bar = (self.pos.length_recip() - u0) / self.last_phi - self.last_phi / 2. * (-u0 + 1.5 * self.schwarz_r * u0 * u0);
                incoming_angle = self.calc_ray_angle(u_bar, 1. / u0);
            }
            // No euclidian geometry allowed!
            //let incoming_angle = Vec3::angle_between(other_position - self.pos, - self.pos) 
            //    * if self.less_than_180 {1.} else {-1.};
            return [self.pos.x, self.pos.y, self.pos.z, incoming_angle];
        }

        let u0 = other_position.length_recip();
        let u1 = self.pos.length_recip();

        // If the observer jumps by more than 0.5, we gotta reset the ray
        if (u0.recip() - self.u_ray[0].recip()).abs() > 0.5 {
            return self.reset_ray(other_position);
        }
        
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
        let mut thomas_c: [f32; NR_NODES - 2] = [0.; NR_NODES - 2]; //Last entry is a dummy
        let h = self.last_phi / (NR_NODES - 1) as f32;
        let scale =  1. / (h*h);
        for _k in 0..iterations {
            // index shifted to make the stencil more clear
            for i in 1..(NR_NODES - 1) {
                residual[i-1] = scale * (-self.u_ray[i-1] + 2. * self.u_ray[i] - self.u_ray[i+1]) - self.u_ray[i] //M_h * u_h
                    + 3. * self.schwarz_r / 2. * self.u_ray[i] * self.u_ray[i];
            }

            // Thomas algorithm to solve: (M_h + 3*R*diag(u_h)) * z_h = residual
            // Off-diagonals are all (-scale)
            let main_diag_inv = 1. / (2. * scale - 1. + 3. * self.schwarz_r * self.u_ray[1]);
            thomas_c[0] = (-scale) * main_diag_inv;
            residual[0] = residual[0] * main_diag_inv;
            for i in 1..(NR_NODES - 2) {
                let main_diag_inv = 1. / (2. * scale - 1. + 3. * self.schwarz_r * self.u_ray[i+1] + scale * thomas_c[i-1]);
                thomas_c[i] = (-scale) * main_diag_inv;
                residual[i] = (residual[i] + scale * residual[i-1]) * main_diag_inv;
            }

            // Back substitution and applying the correction to u_ray
            // Did put this into one to save a for loop
            self.u_ray[NR_NODES-2] -= residual[NR_NODES-3];
            for i in (0..(NR_NODES - 3)).rev() {
                residual[i] = residual[i] - thomas_c[i] * residual[i+1];
                self.u_ray[i+1] -= residual[i];
            }
        }

        //Time to calculate the angle
        let u_bar = (self.u_ray[1] - self.u_ray[0]) / h - h / 2. * (-self.u_ray[0] + 1.5 * self.schwarz_r * self.u_ray[0] * self.u_ray[0]); //Higher order scheme using u''
        let incoming_angle = self.calc_ray_angle(u_bar, u0.recip());
        return [self.pos.x, self.pos.y, self.pos.z, incoming_angle];
    }

    pub fn set_position(&mut self, new_pos: Vec3) {
        self.pos = new_pos;
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
            let intermediate = -(r * r * u_bar * u_bar) / ( 1. - self.schwarz_r / r) - 1.;
            if intermediate > 0. {
                theta = -std::f32::consts::FRAC_PI_2 + f32::atan(intermediate.recip().sqrt());
            }
            else {
                theta = 0.; // Should I come up with a better approach? This only affects points inside the event horizon...
            }
        }
        let special_angle_format = (std::f32::consts::FRAC_PI_2 - theta) * if self.less_than_180 {1.} else {-1.};
        return special_angle_format;
    }

    #[allow(dead_code)]
    pub fn print_ray_for_matlab(&self) -> String {
        let mut result = "polarplot( (0:".to_owned();
        result.push_str(&(NR_NODES-1).to_string());
        result.push_str(") *");
        result.push_str(&(self.last_phi / NR_NODES as f32).to_string());
        result.push_str(", [");
        result.push_str(&(1. / self.u_ray[0]).to_string());

        for i in 1..NR_NODES {
            result.push_str(", ");
            result.push_str(&(1. / self.u_ray[i]).to_string());
        }

        result.push_str("]);");

        return result;
    }
}