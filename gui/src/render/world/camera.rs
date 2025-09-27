// render/orbit_camera.rs
use parry3d::na::{self, Vector3};
use winit::event::{MouseButton, MouseScrollDelta, WindowEvent};

pub struct OrbitCamera {
    pub target: na::Point3<f32>,
    pub radius: f32, // distance from target
    pub yaw: f32,    // radians, around Y
    pub pitch: f32,  // radians, up/down
    // controls
    rotate_sens: f32, // radians per pixel
    pan_sens: f32,    // world units per pixel (scaled by radius)
    zoom_sens: f32,   // scalar per wheel tick
    // state
    rmb_down: bool,
    mmb_down: bool,
    shift_down: bool,
    last_cursor: Option<(f64, f64)>,
}

impl OrbitCamera {
    pub fn new() -> Self {
        Self {
            target: na::Point3::new(0.0, 0.8, 0.0),
            radius: 6.0,
            yaw: 0.0,
            pitch: 0.25, // ~14°
            rotate_sens: 0.005,
            pan_sens: 0.0015,
            zoom_sens: 1.1,
            rmb_down: false,
            mmb_down: false,
            shift_down: false,
            last_cursor: None,
        }
    }

    pub fn view(&self) -> na::Isometry3<f32> {
        let dir = Self::spherical_dir(self.yaw, self.pitch);
        let eye = self.target - dir * self.radius;
        na::Isometry3::look_at_rh(&eye, &self.target, &na::Vector3::y())
    }

    pub fn proj(aspect: f32) -> na::Perspective3<f32> {
        na::Perspective3::new(aspect, (45.0f32).to_radians(), 0.1, 500.0)
    }

    fn spherical_dir(yaw: f32, pitch: f32) -> na::Vector3<f32> {
        let cp = pitch.clamp(-1.5533, 1.5533); // ~±89°
        let cy = yaw.cos();
        let sy = yaw.sin();
        let cpv = cp.cos();
        let sp = cp.sin();
        // right-handed: +Z forward; we want camera looking towards target
        Vector3::new(cy * cpv, sp, sy * cpv)
    }

    pub fn handle_event(&mut self, event: &WindowEvent, imgui_wants_mouse: bool) {
        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                let down = *state == winit::event::ElementState::Pressed;
                match button {
                    MouseButton::Right => self.rmb_down = down,
                    MouseButton::Middle => self.mmb_down = down,
                    _ => {}
                }
            }
            // WindowEvent::KeyboardInput { input, .. } => {
            //     if let Some(vk) = input.virtual_keycode {
            //         if vk == winit::event::VirtualKeyCode::LShift
            //             || vk == winit::event::VirtualKeyCode::RShift
            //         {
            //             self.shift_down = input.state == winit::event::ElementState::Pressed;
            //         }
            //     }
            // }
            WindowEvent::CursorMoved { position, .. } => {
                if imgui_wants_mouse {
                    self.last_cursor = Some((position.x, position.y));
                    return;
                }
                if let Some((lx, ly)) = self.last_cursor {
                    let dx = (position.x - lx) as f32;
                    let dy = (position.y - ly) as f32;

                    // PAN when MMB or Shift+RMB
                    if self.mmb_down || (self.rmb_down && self.shift_down) {
                        // pan along camera right/up
                        let dir = Self::spherical_dir(self.yaw, self.pitch);
                        let right = na::Vector3::new(dir.z, 0.0, -dir.x).normalize(); // Y-up right
                        let up = na::Vector3::y();
                        let scale = self.radius * self.pan_sens;
                        self.target += right * dx * scale;
                        self.target += up * dy * scale;
                    }
                    // ORBIT when RMB (without Shift)
                    else if self.rmb_down {
                        self.yaw += dx * self.rotate_sens;
                        self.pitch -= dy * self.rotate_sens;
                        self.pitch = self.pitch.clamp(-1.53, 1.53);
                    }
                }
                self.last_cursor = Some((position.x, position.y));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if imgui_wants_mouse {
                    return;
                }
                match delta {
                    MouseScrollDelta::LineDelta(_, y) => {
                        self.radius = (self.radius / self.zoom_sens.powf(*y)).clamp(0.5, 200.0);
                    }
                    MouseScrollDelta::PixelDelta(p) => {
                        let y = p.y as f32 / 60.0;
                        self.radius = (self.radius / self.zoom_sens.powf(y)).clamp(0.5, 200.0);
                    }
                }
            }
            WindowEvent::Focused(false) => {
                self.rmb_down = false;
                self.mmb_down = false;
            }
            _ => {}
        }
    }
}
