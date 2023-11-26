use glam::*;
use super::{orbit::Orbit, polar_transformations::{look_to_vec_mat, polar2_to_carthesic}};

const SAFE_FRAC_PI_2: f64 = std::f64::consts::FRAC_PI_2 - 0.0001;

#[allow(dead_code)]
#[derive(PartialEq)]
enum ObserverState {
    Unmoving,   // No movement relative to the black hole
    FrozenFall, // Only movement aberration according to a straight fall, user choses position
    Orbiting,      // Simulated movement on an orbit
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformationPipeline{
    pub display_to_movement: [f32; 16],
    pub movement_to_central: [f32; 16],
    pub central_to_uv: [f32; 16],
    pub psi_factor_and_position: [f32; 4],
}

#[allow(dead_code)]
impl TransformationPipeline {
    pub fn new() -> Self {
        Self {
            display_to_movement: [0.; 16],
            movement_to_central: [0.; 16],
            central_to_uv: [0.; 16],
            psi_factor_and_position: [0.; 4],
        }
        
    }
}

pub struct Observer{
    schwarz_r: f64,
    position: DVec3,    // Carthesic coordinates
    //velocity: DVec3,    //lets try to have this dependent
    camera: DVec2,
    orbit: Option<Orbit>,
    state: ObserverState,

    time_step: f64,
    energy: f64,

    mouse_sensitivity: f64,

    // sub-matrices needed to assemble the first transformation
    // the camera transformation is left out, so looking around is possible even when singular
    fov_scaling: DMat3,     //constant
    standard_to_movement: DMat3,    

    //Further rotates towards the center of the black holes
    movement_to_central: DMat3,
    //Rotates on the texture sphere towards the texture coordinates
    central_to_uv: DMat3,
    psi: f64,
}

#[allow(dead_code)]
impl Observer {
    pub fn new(schwarz_r: f64, fov: f64, width: f64, height: f64) -> Self {
        let screen_ratio = width / height;
        let position = dvec3(25., 0., 0.);
        let camera = dvec2(std::f64::consts::PI, 0.);
        Self {
            schwarz_r,
            position,
            camera,
            orbit: None,
            state: ObserverState::FrozenFall,
            time_step: 1./60.,
            energy: 1.,
            mouse_sensitivity: fov / height,
            fov_scaling: DMat3::from_diagonal(DVec3::new((fov/2.).tan(), (fov/2.).tan() * screen_ratio, 1.)),
            standard_to_movement: DMat3::IDENTITY,
            movement_to_central: DMat3::IDENTITY,
            central_to_uv: DMat3::IDENTITY,
            psi: 1.,
        }
    }

    fn h_r(&self) -> f64 {
        return 1. - self.schwarz_r / self.position.length();
    }

    pub fn get_radial_position(&self) -> f64 {
        return self.position.length();
    }

    // Maybe add advance_some_steps

    // Updates the position with either user commands or simulated trajectory
    // desired_direction is in terms of (forward, left, up)
    pub fn update_position(&mut self, mut desired_direction: DVec3) {
        match self.state {
            ObserverState::Unmoving | ObserverState::FrozenFall => {
                let movement_step = 0.051;
                desired_direction = movement_step * DMat3::from_rotation_z(-self.camera.x) * desired_direction;
                self.position += desired_direction;
            },
            ObserverState::Orbiting => {
                self.orbit.as_mut().unwrap().do_step(self.time_step);
                self.position = self.orbit.as_mut().unwrap().get_position();
                // match mutable is not available
                //match &self.orbit {
                //     Some(orbit) => {
                //         orbit.do_step(self.time_step);
                //         self.position = orbit.get_position();
                //     },
                //     None => {},    //This should never happen
                // }
            },
        }
    }

    // Returns the momentary velocity in t,r,phi
    pub fn velocity(&mut self) -> DVec3 {
        match self.state {
            ObserverState::Unmoving => self.unmoving_velocity(),
            ObserverState::FrozenFall => self.frozen_fall_velocity(),
            ObserverState::Orbiting => {
                match &self.orbit {
                    Some(orbit) => orbit.get_velocity(),
                    None => self.unmoving_velocity(), //This should never happen
                }
            }
        }
    }

    pub fn unmoving_velocity(&self) -> DVec3 {
        let mut velocity = DVec3::ZERO;
        if self.position.length() > self.schwarz_r {
            velocity.x = 1. / self.h_r().sqrt();
            velocity.y = 0.;
        }
        // Inside the event horizon case
        else {
            velocity.x = 0.;
            velocity.y = - f64::sqrt(-self.h_r());
        }
        return velocity;
    }

    pub fn frozen_fall_velocity(&self) -> DVec3 {
        if self.energy.powi(2) < self.h_r() {
            return self.unmoving_velocity();
        }
        return dvec3(self.energy / self.h_r(), (self.energy.powi(2) - self.h_r()).sqrt(), 0.);
    }

    pub fn start_orbit(&mut self, rotation: f64) {
        let direction = dvec3( -self.position.y, self.position.x , 0.);
        self.orbit = Orbit::new(self.schwarz_r, self.position, direction, rotation);
        if self.orbit.is_some() {
            self.state = ObserverState::Orbiting;
            //Maybe include a log message about the orbit
        }
    }

    // Enters the frozen falling mode
    // Could include starting height for the fall, never used it though
    pub fn start_frozen_fall(&mut self) {
        self.state = ObserverState::FrozenFall;
    }

    // Enters the unmoving mode
    // This mode makes no physical sense within the event horizon
    pub fn start_unmoving(&mut self) {
        self.state = ObserverState::Unmoving;
    }

    //TODO
    pub fn is_singular(&self) -> bool {
        // We are close to the event horizon
        if (self.position.length() - self.schwarz_r).abs() < 1e-10 as f64 {
            return true;
        }
        return match self.state {
            ObserverState::Unmoving | ObserverState::FrozenFall => self.position.length() < 1e-10 as f64,
            ObserverState::Orbiting => match &self.orbit {
                Some(orbit) => orbit.is_singular(),
                None => true, //should never happen
            },
        }
    }

    pub fn calc_transformation_pipeline(&mut self) -> TransformationPipeline {
        let r = self.position.length();

        //can only update position related values if we are not singular
        if !self.is_singular() {
            let vel = self.velocity();
            
            if r > self.schwarz_r {
                self.psi = vel.x * vel.x * self.h_r();
            }
            else {
                self.psi = -vel.y * vel.y / self.h_r();
            }
            if self.psi - 1. < 1e-10 as f64 {
                self.psi = 1.;
            }

            let standard_to_central = look_to_vec_mat(-self.position).transpose();
            if self.state == ObserverState::Orbiting && !self.orbit.as_ref().unwrap().is_central_fall() {
                let mut tilt_angle = 0.;
                let mut plane_angle1 = 0.;
                let mut plane_angle2 = 0.;

                match &self.orbit {
                    Some(orbit) => {
                        tilt_angle = orbit.current_tilt_angle();
                        plane_angle1 = f64::acos(-vel.x * vel.y * (r-self.schwarz_r).signum() / 
                            ((1. + r * r * vel.z * vel.z) * self.psi * (self.psi - 1.)).sqrt());
                        if r > self.schwarz_r {
                            plane_angle2 = (-vel.y / (self.h_r() * (self.psi - 1.)).sqrt()).acos();
                        }
                        else {
                            plane_angle2 = (-vel.x * (-self.h_r() / (self.psi - 1.)).sqrt()).acos();
                        }
                    }
                    None => {},
                };
                let orbit_plane_tilt = DMat3::from_rotation_z(-tilt_angle);
                let tilted_center_to_movement1 = DMat3::from_rotation_x(-plane_angle1);
                let movement2_to_tilted_center = DMat3::from_rotation_x(plane_angle2);

                
                
                self.standard_to_movement = tilted_center_to_movement1 * orbit_plane_tilt * standard_to_central;
                self.movement_to_central = orbit_plane_tilt.transpose() * movement2_to_tilted_center;
            }
            else {
                self.standard_to_movement = standard_to_central;
                self.movement_to_central = DMat3::IDENTITY;
            }
            // Need to invert y, because geodesics mirror the coordinates
            self.central_to_uv = look_to_vec_mat(self.position) * DMat3::from_diagonal(dvec3(1., -1., 1.));
            
        }
        //Allows the camera to update even if singular
        let camera_to_standard = look_to_vec_mat(polar2_to_carthesic(self.camera));

        return TransformationPipeline{
            display_to_movement: Mat4::from_mat3((self.standard_to_movement * camera_to_standard * self.fov_scaling).as_mat3()).to_cols_array(),
            movement_to_central: Mat4::from_mat3(self.movement_to_central.as_mat3()).to_cols_array(),
            central_to_uv: Mat4::from_mat3(self.central_to_uv.as_mat3()).to_cols_array(),
            psi_factor_and_position: [((self.psi - 1.) / self.psi).sqrt() as f32, self.position.x as f32, self.position.y as f32, self.position.z as f32],
        }
    }

    pub fn update_screen_format(&mut self, width: f64, height: f64) {
        // Update screen scaling matrix
        let screen_ratio = width / height;
        let fov_half_tan = self.fov_scaling.x_axis.x;
        self.fov_scaling = DMat3::from_diagonal(DVec3::new(fov_half_tan, fov_half_tan * screen_ratio, 1.));

        // Update mouse sensitivity

    }

    pub fn move_camera(&mut self, horizontal_pixels: f64, vertical_pixels: f64) {
        let delta_phi = horizontal_pixels * self.mouse_sensitivity;
        let delta_theta = vertical_pixels * self.mouse_sensitivity;

        self.camera.x += delta_phi;
        self.camera.y += delta_theta;
        if self.camera.y < -SAFE_FRAC_PI_2 {
            self.camera.y = -SAFE_FRAC_PI_2;
        }
        else if self.camera.y > SAFE_FRAC_PI_2 {
            self.camera.y = SAFE_FRAC_PI_2;
        }
    }

    pub fn get_schwarz_r(&self) -> f64 {
        return self.schwarz_r;
    }

    pub fn reset_to_start(&mut self) {
        self.position = dvec3(25., 0., 0.);
        self.camera = dvec2(std::f64::consts::PI, 0.);
        self.start_frozen_fall();
    }
}

