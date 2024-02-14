use crate::{model::{ModelVertex, self}, texture};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimpleVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl SimpleVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SimpleVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct Cube(pub model::Mesh);

impl Cube {
    pub fn new(name: &str, device: &wgpu::Device) -> Self {
        #[rustfmt::skip]
        let scale = 2.0;
        let vertices = [
            SimpleVertex { position: [-0.5 * scale,  0.5 * scale,  0.5 * scale], tex_coords: [-1.0, -1.0] },
            SimpleVertex { position: [-0.5 * scale, -0.5 * scale,  0.5 * scale], tex_coords: [-1.0, -1.0] },
            SimpleVertex { position: [ 0.5 * scale, -0.5 * scale,  0.5 * scale], tex_coords: [-1.0, -1.0] },
            SimpleVertex { position: [ 0.5 * scale,  0.5 * scale,  0.5 * scale], tex_coords: [-1.0, -1.0] },

            SimpleVertex { position: [ 0.5 * scale, -0.5 * scale, -0.5 * scale], tex_coords: [-1.0, -1.0] },
            SimpleVertex { position: [ 0.5 * scale,  0.5 * scale, -0.5 * scale], tex_coords: [-1.0, -1.0] },
            SimpleVertex { position: [-0.5 * scale, -0.5 * scale, -0.5 * scale], tex_coords: [-1.0, -1.0] },
            SimpleVertex { position: [-0.5 * scale,  0.5 * scale, -0.5 * scale], tex_coords: [-1.0, -1.0] },
        ];


        #[rustfmt::skip]
        let indices = [
            0, 1, 2,
            2, 3, 0,
            2, 4, 3,
            3, 4, 5,
            4, 6, 5,
            6, 7, 5,
            6, 4, 1,
            4, 2, 1,
            1, 7, 6,
            0, 7, 1,
            0, 3, 7,
            7, 3, 5,
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Vertex Buffer", name)),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", name)),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mesh = model::Mesh {
            name: format!("{:?} mesh", name),
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
            material: 0,
        };

        Self(mesh)
    }
}

impl From<Cube> for model::Mesh {
    fn from(value: Cube) -> Self {
        value.0
    }
}

pub struct Plane(pub model::Mesh);

impl Plane {
    pub fn new(name: &str, device: &wgpu::Device) -> Self {
        //let scale = 10.0;
        //#[rustfmt::skip]
        //let vertices = [
        //    SimpleVertex { position: [-2.0 * scale, -1.0, -2.0 * scale], tex_coords: [-1.0, -1.0] },
        //    SimpleVertex { position: [-2.0 * scale, -1.0,  2.0 * scale], tex_coords: [-1.0, -1.0] },
        //    SimpleVertex { position: [ 2.0 * scale, -1.0,  2.0 * scale], tex_coords: [-1.0, -1.0] },
        //    SimpleVertex { position: [ 2.0 * scale, -1.0, -2.0 * scale], tex_coords: [-1.0, -1.0] },
        //];

        //#[rustfmt::skip]
        //let indices = [
        //    0, 1, 2,
        //    2, 3, 0,
        //];

        #[rustfmt::skip]
        let vertices = [
            ModelVertex { position: [1.0, 0.0, 0.0], tex_coords: [-1.0, -1.0], normal: [0.0, 0.0, 1.0]},
            ModelVertex { position: [0.0, 1.0, 0.0], tex_coords: [-1.0, -1.0], normal: [0.0, 0.0, 1.0]},
            ModelVertex { position: [0.0, 0.0, 0.0], tex_coords: [-1.0, -1.0], normal: [0.0, 0.0, 1.0]},
            ModelVertex { position: [1.0, 1.0, 0.0], tex_coords: [-1.0, -1.0], normal: [0.0, 0.0, 1.0]},
        ];

        #[rustfmt::skip]
        let indices = [
            0, 1, 2,
            0, 3, 1,
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Vertex Buffer", name)),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", name)),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mesh = model::Mesh {
            name: format!("{:?} mesh", name),
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
            material: 0,
        };

        Self(mesh)
    }
}

impl From<Plane> for model::Mesh {
    fn from(value: Plane) -> Self {
        value.0
    }
}

pub struct Billboard(pub model::Mesh);

impl Billboard {
    pub fn new(name: &str, device: &wgpu::Device) -> Self {
        #[rustfmt::skip]
        let vertices = [
            SimpleVertex { position: [-3.0,  4.0,  4.0], tex_coords: [0.0, 1.0] },
            SimpleVertex { position: [-3.0, -4.0,  4.0], tex_coords: [0.0, 0.0] },
            SimpleVertex { position: [-3.0, -4.0, -4.0], tex_coords: [1.0, 0.0] },
            SimpleVertex { position: [-3.0,  4.0, -4.0], tex_coords: [1.0, 1.0] },
        ];

        #[rustfmt::skip]
        let indices = [
            0, 1, 2,
            2, 3, 0,
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Vertex Buffer", name)),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", name)),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mesh = model::Mesh {
            name: format!("{:?} mesh", name),
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
            material: 0,
        };

        Self(mesh)
    }
}

impl From<Billboard> for model::Mesh {
    fn from(value: Billboard) -> Self {
        value.0
    }
}
