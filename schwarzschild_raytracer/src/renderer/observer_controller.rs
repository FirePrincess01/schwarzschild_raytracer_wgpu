//! Tracks key and mouse inputs to move the camera and position of the observer

use glam::DVec3;
use winit::event::*;
use winit::dpi::PhysicalPosition;
use instant::Duration;

use crate::simulation::observer::Observer;

#[derive(Debug)]
pub struct ObserverController {
    amount_left: f64,
    amount_right: f64,
    amount_forward: f64,
    amount_backward: f64,
    amount_up: f64,
    amount_down: f64,
    rotate_horizontal: f64,
    rotate_vertical: f64,
    scroll: f64,
    speed: f64,
    sensitivity: f64,
    _sensitivity_scroll: f64,
}

impl ObserverController {
    pub fn new(speed: f64, sensitivity: f64, _sensitivity_scroll: f64) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
            _sensitivity_scroll,

        }
    }

    pub fn process_keyboard(&mut self, key: winit::keyboard::KeyCode, state: ElementState) -> bool{
        let amount = if state == ElementState::Pressed { 1.0 } else { 0.0 };
        match key {
            winit::keyboard::KeyCode::KeyW | winit::keyboard::KeyCode::ArrowUp => {
                self.amount_forward = amount;
                true
            }
            winit::keyboard::KeyCode::KeyS | winit::keyboard::KeyCode::ArrowDown => {
                self.amount_backward = amount;
                true
            }
            winit::keyboard::KeyCode::KeyA | winit::keyboard::KeyCode::ArrowLeft => {
                self.amount_left = amount;
                true
            }
            winit::keyboard::KeyCode::KeyD | winit::keyboard::KeyCode::ArrowRight => {
                self.amount_right = amount;
                true
            }
            winit::keyboard::KeyCode::Space => {
                self.amount_up = amount;
                true
            }
            winit::keyboard::KeyCode::ShiftLeft => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx;
        self.rotate_vertical = mouse_dy;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => *scroll as f64 * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition {
                y: scroll,
                ..
            }) => *scroll, 
        };
    }

    pub fn update_observer(&mut self, observer: &mut Observer, dt: Duration) {
        let dt = dt.as_secs_f64();

        // Move
        let mut direction = DVec3::ZERO;
        direction.x += self.amount_forward - self.amount_backward;
        direction.y += self.amount_left - self.amount_right;
        direction.z += self.amount_up - self.amount_down;
        observer.update_position(direction * self.speed * dt, dt);

        self.scroll = 0.0; //not used right now

        // Rotate and reset rotation input
        observer.move_camera(self.rotate_horizontal * self.sensitivity, self.rotate_vertical * self.sensitivity);
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
    }

    // pub fn compute_rotation_delta(&mut self, dt: Duration) -> f64 {
    //     let dt = dt.as_secs_f64();
    //     let mut delta = 0.;

    //     delta += (self.amount_forward - self.amount_backward) * dt;
    //     delta += self.scroll;
    //     self.scroll = 0.0;
    //     delta += self.rotate_vertical;
    //     self.rotate_horizontal = 0.0;
    //     self.rotate_vertical = 0.0;
    //     return delta;
    // }
}

 

 