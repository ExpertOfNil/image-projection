use crate::{PhysicalKey, WindowEvent, KeyCode};

pub enum Movement {
    Forward,
    Left,
    Backward,
    Right,
}

pub struct Camera {
    pub eye: glam::Vec3,
    pub target: glam::Vec3,
    pub up: glam::Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    const SPEED: f32 = 0.2;
    pub fn build_view_projection_matrix(&self) -> glam::Mat4 {
        let view = glam::Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = glam::Mat4::perspective_rh(
            self.fovy,
            self.aspect,
            self.znear,
            self.zfar,
        );
        proj * view
    }

    pub fn update(&mut self, direction: Movement) {
        let fwd = self.target - self.eye;
        let fwd_norm = fwd.normalize();
        let fwd_mag = fwd.length();

        match direction {
            Movement::Forward => {
                //if fwd_mag > Self::SPEED {
                //    //self.eye += fwd_norm * Self::SPEED;
                //    self.eye = self.target - (fwd + self.up * Self::SPEED).normalize() * fwd_mag;
                //}
                self.eye = self.target - (fwd + self.up * Self::SPEED).normalize() * fwd_mag;
            }
            Movement::Backward => {
                //if fwd_mag > Self::SPEED {
                //    self.eye -= fwd_norm * Self::SPEED;
                //}
                self.eye = self.target - (fwd - self.up * Self::SPEED).normalize() * fwd_mag;
            }
            Movement::Right => {
                let right = fwd_norm.cross(self.up);
                self.eye = self.target - (fwd + right * Self::SPEED).normalize() * fwd_mag;
            }
            Movement::Left => {
                let right = fwd_norm.cross(self.up);
                self.eye = self.target - (fwd - right * Self::SPEED).normalize() * fwd_mag;
            }
        }
    }
}

pub struct CameraController {
    pub speed: f32,
    pub is_fwd_pressed: bool,
    pub is_bwd_pressed: bool,
    pub is_rt_pressed: bool,
    pub is_lt_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_fwd_pressed: false,
            is_bwd_pressed: false,
            is_rt_pressed: false,
            is_lt_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                let is_pressed = event.state.is_pressed();
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyE) => {
                        self.is_fwd_pressed = is_pressed;
                        true
                    }
                    PhysicalKey::Code(KeyCode::KeyD) => {
                        self.is_bwd_pressed = is_pressed;
                        true
                    }
                    PhysicalKey::Code(KeyCode::KeyF) => {
                        self.is_rt_pressed = is_pressed;
                        true
                    }
                    PhysicalKey::Code(KeyCode::KeyS) => {
                        self.is_lt_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        let fwd = camera.target - camera.eye;
        let fwd_norm = fwd.normalize();
        let fwd_mag = fwd.length();

        if self.is_fwd_pressed && fwd_mag > self.speed {
            //camera.eye += fwd_norm * self.speed;
            camera.eye = camera.target - (fwd - camera.up * self.speed).normalize() * fwd_mag;
        }
        if self.is_bwd_pressed && fwd_mag > self.speed {
            //camera.eye -= fwd_norm * self.speed;
            camera.eye = camera.target - (fwd + camera.up * self.speed).normalize() * fwd_mag;
        }

        let right = fwd_norm.cross(camera.up);

        let fwd = camera.target - camera.eye;
        let fwd_mag = fwd.length();

        if self.is_rt_pressed {
            camera.eye = camera.target - (fwd - right * self.speed).normalize() * fwd_mag;
        }
        if self.is_lt_pressed {
            camera.eye = camera.target - (fwd + right * self.speed).normalize() * fwd_mag;
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        let ident = glam::Mat4::default();
        let view_proj = [
            ident.x_axis.into(),
            ident.y_axis.into(),
            ident.z_axis.into(),
            ident.w_axis.into(),
        ];

        Self {
            view_proj,
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        let matrix = camera.build_view_projection_matrix();
        self.view_proj = [
            matrix.x_axis.into(),
            matrix.y_axis.into(),
            matrix.z_axis.into(),
            matrix.w_axis.into(),
        ];
    }
}
