use winit::{
    dpi::PhysicalPosition,
    event::MouseScrollDelta,
    keyboard::{KeyCode, PhysicalKey},
};

pub enum Movement {
    Forward,
    Left,
    Backward,
    Right,
}

pub struct Projection {
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Projection {
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn matrix(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

pub struct Camera {
    pub position: glam::Vec3,
    pitch: f32,
    yaw: f32,
}

impl Camera {
    pub fn new(position: glam::Vec3, pitch: f32, yaw: f32) -> Self {
        Self {
            position,
            pitch,
            yaw,
        }
    }

    pub fn matrix(&self) -> glam::Mat4 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let dir = glam::Vec3::new(cos_pitch * cos_yaw, cos_pitch * sin_yaw, sin_pitch);
        glam::Mat4::look_to_rh(self.position, dir, glam::Vec3::Z)
    }
}

pub struct CameraController {
    left: f32,
    right: f32,
    forward: f32,
    backward: f32,
    up: f32,
    down: f32,
    rotate_h: f32,
    rotate_v: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            left: 0.0,
            right: 0.0,
            forward: 0.0,
            backward: 0.0,
            up: 0.0,
            down: 0.0,
            rotate_h: 0.0,
            rotate_v: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: PhysicalKey, pressed: bool) -> bool {
        let amount = if pressed { 1.0 } else { 0.0 };
        match key {
            PhysicalKey::Code(KeyCode::KeyE) | PhysicalKey::Code(KeyCode::ArrowUp) => {
                self.forward = amount;
                true
            }
            PhysicalKey::Code(KeyCode::KeyD) | PhysicalKey::Code(KeyCode::ArrowDown) => {
                self.backward = amount;
                true
            }
            PhysicalKey::Code(KeyCode::KeyS) | PhysicalKey::Code(KeyCode::ArrowLeft) => {
                self.left = amount;
                true
            }
            PhysicalKey::Code(KeyCode::KeyF) | PhysicalKey::Code(KeyCode::ArrowRight) => {
                self.right = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_h = -mouse_dx as f32;
        self.rotate_v = -mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: instant::Duration) {
        let dt = dt.as_secs_f32();
        // Move forward/backward and left/right
        let (sin_yaw, cos_yaw) = camera.yaw.sin_cos();
        let forward = glam::Vec3::new(cos_yaw, sin_yaw, 0.0).normalize();
        let right = glam::Vec3::new(-sin_yaw, cos_yaw, 0.0).normalize();
        camera.position += forward * (self.forward - self.backward) * self.speed * dt;
        camera.position += right * (self.right - self.left) * self.speed * dt;

        // Move in/out (zoom)
        let (sin_pitch, cos_pitch) = camera.pitch.sin_cos();
        let scrollward =
            glam::Vec3::new(cos_pitch * cos_yaw, cos_pitch * sin_yaw, sin_pitch).normalize();
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Move up/down
        camera.position.z += (self.up - self.down) * self.speed * dt;

        // Rotate
        camera.yaw += self.rotate_h * self.sensitivity * dt;
        self.rotate_h = 0.0;
        camera.pitch += self.rotate_v * self.sensitivity * dt;
        self.rotate_v = 0.0;

        // Limit rotation up/down
        if camera.pitch < -std::f32::consts::FRAC_PI_2 {
            camera.pitch = -std::f32::consts::FRAC_PI_2;
        } else if camera.pitch > std::f32::consts::FRAC_PI_2 {
            camera.pitch = std::f32::consts::FRAC_PI_2;
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl Default for CameraUniform {
    fn default() -> Self {
        let ident = glam::Mat4::default();
        let view_proj = [
            ident.x_axis.into(),
            ident.y_axis.into(),
            ident.z_axis.into(),
            ident.w_axis.into(),
        ];

        Self { view_proj }
    }
}

impl CameraUniform {
    pub fn new(view_matrix: glam::Mat4, proj_matrix: glam::Mat4) -> Self {
        let mat = proj_matrix * view_matrix;
        let view_proj = [
            mat.x_axis.into(),
            mat.y_axis.into(),
            mat.z_axis.into(),
            mat.w_axis.into(),
        ];

        Self { view_proj }
    }

    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        let matrix = projection.matrix() * camera.matrix();
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
}

impl Projector {
    pub fn new() -> Self {
        let sensor_size = 24_f32;
        let focal_length = 50_f32;
        let fovy = 2.0 * ((sensor_size / focal_length) * 0.5).atan();
        Self {
            view: glam::Mat4::default(),
            aspect: 16.0 / 9.0,
            fovy,
            znear: 0.1,
            zfar: 100.0,
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
        proj * self.view.inverse()
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
