use crate::model;
use crate::{KeyCode, PhysicalKey, WindowEvent};

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
    //pub view: glam::Mat4,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    const SPEED: f32 = 0.2;
    pub fn view_matrix(&self) -> glam::Mat4 {
        glam::Mat4::look_at_rh(self.eye, self.target, self.up)
        //self.view
    }
    pub fn build_view_projection_matrix(&self) -> glam::Mat4 {
        let view = glam::Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = glam::Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);
        proj * view
        //proj * self.view
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

        Self { view_proj }
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

pub struct Projector {
    pub view: glam::Mat4,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub material: model::Material,
}

impl Projector {
    pub fn new(material: model::Material) -> Self {
        let sensor_size = 24_f32;
        let focal_length = 50_f32;
        let fovy = 2.0 * ((sensor_size / focal_length) * 0.5).atan();
        Self {
            view: glam::Mat4::default(),
            aspect: 16.0/9.0,
            fovy,
            znear: 0.1,
            zfar: 100.0,
            material,
        }
    }

    pub fn with_fovy(self, fovy: f32) -> Self {
        let mut this = self;
        this.fovy = fovy;
        this
    }

    pub fn with_eye_target_up(self, eye: glam::Vec3, target: glam::Vec3, up: glam::Vec3) -> Self {
        let mut this = self;
        this.view = glam::Mat4::look_to_rh(eye, target, up);
        this
    }

    pub fn with_rotation_translation(self, rot: glam::Quat, pos: glam::Vec3) -> Self {
        let view = glam::Mat4::from_rotation_translation(rot, pos);
        let fwd = (view * glam::Vec4::new(0.0, 0.0, -1.0, 1.0)).normalize();
        let fwd = glam::vec3(fwd.x, fwd.y, fwd.z);
        let eye = view * glam::Vec4::new(0.0, 0.0, 0.0, 1.0);
        let eye = glam::vec3(eye.x, eye.y, eye.z);
        let mut this = self;
        this.view = glam::Mat4::look_to_rh(eye, fwd, glam::Vec3::Z);
        this
    }

    pub fn with_view(self, view: glam::Mat4) -> Self {
        let mut this = self;
        this.view = view;
        this
    }

    pub fn rotation(&self) -> glam::Quat {
        glam::Quat::from_mat4(&self.view)
    }

    pub fn position(&self) -> glam::Vec3 {
        glam::Vec3 {
            x: self.view.w_axis.x,
            y: self.view.w_axis.y,
            z: self.view.w_axis.z,
        }
    }

    pub fn build_view_projection_matrix(&self) -> glam::Mat4 {
        let proj = glam::Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);
        proj * self.view
    }
}

pub struct EulerDegreesXYZ(pub [f32; 3]);

impl From<EulerDegreesXYZ> for glam::Quat {
    fn from(value: EulerDegreesXYZ) -> glam::Quat {
        let [x, y, z] = value.0;
        glam::Quat::from_euler(
            glam::EulerRot::XYZ,
            x.to_radians(),
            y.to_radians(),
            z.to_radians(),
        )
    }
}

impl From<&Projector> for CameraUniform {
    fn from(value: &Projector) -> CameraUniform {
        let matrix = value.build_view_projection_matrix();
        CameraUniform {
            view_proj: [
                matrix.x_axis.into(),
                matrix.y_axis.into(),
                matrix.z_axis.into(),
                matrix.w_axis.into(),
            ],
        }
    }
}
