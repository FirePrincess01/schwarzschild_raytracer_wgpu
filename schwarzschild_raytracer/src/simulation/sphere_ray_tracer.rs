use std::f64::consts::{PI, FRAC_PI_2};



pub struct SphereRayTracer {
    sphere_r: f64,
    schwarz_r: f64,
    max_iter: u32,
    default_step: f64,
    nr_nodes: usize,  //this should be an even number
    interpolation_grid: Vec<f32>,
}

impl SphereRayTracer {
    const NO_VALUE: f64 = 10.;    //roughly five rotations

    pub fn new(sphere_r: f64, schwarz_r: f64, max_iter: u32, default_step: f64, nr_nodes_half: usize) -> Self { 
        Self { 
            sphere_r, 
            schwarz_r, 
            max_iter, 
            default_step, 
            nr_nodes: nr_nodes_half * 2,
            interpolation_grid: vec![Self::NO_VALUE as f32; nr_nodes_half * 2],
        }
    }

    pub fn solve_ray_fan(&mut self, r: f64) -> &Vec<f32> {
        
        for i in 0..self.nr_nodes {
            let theta = FRAC_PI_2 - PI * (i as f64) / (self.nr_nodes as f64 - 1.);
            let rotation = r * theta.cos();
            let r_falling:bool;
            let energy: f64;
            if r < self.schwarz_r {
                r_falling = false;
                energy = f64::sin(-theta) * (-1. + self.schwarz_r / r).sqrt();
            }
            else {
                r_falling = theta > 0.;
                energy = (1. - self.schwarz_r / r).sqrt();
            }

            // transforming the traveled angle into theta from polar coordinates
            self.interpolation_grid[i] = (FRAC_PI_2 - self.solve_geodesic(r, energy, rotation, r_falling)) as f32;
        }

        return &self.interpolation_grid;
    }

    fn solve_geodesic(&mut self, r: f64, energy: f64, rotation: f64, r_falling: bool) -> f64 {
        let b = rotation / energy;
        let outside = r > self.schwarz_r;
        let sphere_outside = self.sphere_r > self.schwarz_r;
        let inside_sphere = r < self.sphere_r;

        //looking straight in or out
        if rotation < 1e-10 as f64 {
            if inside_sphere {
                if outside {
                    if r_falling {
                        if self.schwarz_r == 0. {
                            return PI;
                        } 
                        else { 
                            return SphereRayTracer::NO_VALUE; 
                        }
                    }
                    else {
                        return 0.;
                    }
                }
                else {
                    if sphere_outside {
                        if energy > 0. {
                            return 0.;
                        }
                        else {
                            return SphereRayTracer::NO_VALUE;
                        }
                    }
                    else {
                        return 0.;
                    }
                }
            }
            else {
                if sphere_outside && r_falling {
                    return 0.;
                }
                else {
                    return SphereRayTracer::NO_VALUE;
                }
            }
        }

        //Energy requirement to leave the 3R/2 barrier
        let barrier_3r_2 = (self.schwarz_r > 0.) && 1. / (b * b) < 4. / (27. * self.schwarz_r * self.schwarz_r);
        //Wether r and sphere_r are on different sides of the 3R/2 barrier
        let r3_2 = 3. * self.schwarz_r / 2.;
        let different_sides_3r_2 = ((r < r3_2) ^ (self.sphere_r < r3_2)) && (r - r3_2).abs() > (1e-10 as f64);

        //Some cases where the geodesic won't hit the surface
        if (inside_sphere && !sphere_outside) ||
            (!outside && sphere_outside && energy < 0.) ||
            (barrier_3r_2 && different_sides_3r_2) ||
            (r < r3_2 && inside_sphere && r_falling) ||
            (r > r3_2 && !inside_sphere && ! r_falling) {
            return SphereRayTracer::NO_VALUE;
        }

        //After preliminary checks, starting the RK4 scheme
        let mut u_k = 1. / r;
        let mut u_bar_k = if r_falling {1.} else {-1.} * f64::sqrt(1. / (b * b) - (1. - self.schwarz_r / r) / (r * r));
        let mut angle = 0.;
        let mut iteration = 0;

        let bound = 0.9 * f64::min(u_k, 1. / f64::max(self.sphere_r, r3_2));
        let step = self.default_step;
        let final_newton_refinements: u32 = 3;
        let step_half = step / 2.;
        let sphere_u = 1. / self.sphere_r;
        let schwarz_u = 1. / self.schwarz_r;

        while (!(self.schwarz_r != 0. && u_k > schwarz_u && u_bar_k > 0.) // not inside BH and falling
            && iteration < self.max_iter && u_k > 0. ) {

            let mut a_u = u_k + step_half * u_bar_k;
            let mut a_u_bar = u_bar_k + step_half * (-u_k + r3_2 * u_k*u_k);
            let mut b_u = u_k + step_half * a_u_bar;
            let mut b_u_bar = u_bar_k + step_half * (-a_u + r3_2 * a_u*a_u);
            let mut c_u = u_k + step * b_u_bar;
            let mut c_u_bar = u_bar_k + step * (-b_u + r3_2 * b_u*b_u);

            let next_u = u_k + step * (u_bar_k + 2. * a_u_bar + 2. * b_u_bar + c_u_bar) / 6.;
			let next_u_bar = u_bar_k + step * ((-u_k + r3_2 * u_k*u_k) +
				2. * (-a_u + r3_2 * a_u*a_u) + 2. * (-b_u + r3_2 * b_u*b_u) + (-c_u + r3_2 * c_u*c_u)) / 6.;

            //check if the ray has passed through the surface, then do some newton to find the precise cut.
			//The Newton method works with the function of one RK4 step from the previous position
            if (next_u > sphere_u)^(u_k > sphere_u) {
                let mut newton_u;
                let mut newton_u_bar;
                let mut newton_step;
                //Start at side with larger slope
                if u_bar_k.abs() > next_u_bar.abs() {
                    newton_step = 0.;
                    newton_u = u_k;
                    newton_u_bar = u_bar_k
                }
                else {
                    newton_step = step;
                    newton_u = next_u;
                    newton_u_bar = next_u_bar;
                }

                for _ in 0..final_newton_refinements {
                    newton_step -= (newton_u - sphere_u) / newton_u_bar;
                    let newton_step_half = newton_step / 2.;

                    a_u = u_k + newton_step_half * u_bar_k;
                    a_u_bar = u_bar_k + newton_step_half * (-u_k + r3_2 * u_k*u_k);
                    b_u = u_k + newton_step_half * a_u_bar;
                    b_u_bar = u_bar_k + newton_step_half * (-a_u + r3_2 * a_u*a_u);
                    c_u = u_k + newton_step * b_u_bar;
                    c_u_bar = u_bar_k + newton_step * (-b_u + r3_2 * b_u*b_u);

                    newton_u = u_k + newton_step * (u_bar_k + 2. * a_u_bar + 2. * b_u_bar + c_u_bar) / 6.;
                    newton_u_bar = u_bar_k + newton_step * ((-u_k + r3_2 * u_k*u_k) +
                        2. * (-a_u + r3_2 * a_u*a_u) + 2. * (-b_u + r3_2 * b_u*b_u) + (-c_u + r3_2 * c_u*c_u)) / 6.;
                }
                return angle + newton_step;
            }

            if next_u < bound {
                return SphereRayTracer::NO_VALUE;
            }
            u_k = next_u;
            u_bar_k = next_u_bar;
            iteration += 1;
            angle += step;
        }
        return SphereRayTracer::NO_VALUE;
    }

    
}

