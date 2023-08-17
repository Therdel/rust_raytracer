use std::borrow::Cow;
use std::io::Cursor;

use lib_raytracer::raytracing::{Light, Material, Sphere, Triangle};
use lib_raytracer::{gpu_types, Scene};
use lib_raytracer::scene_file::Parser;
use nalgebra_glm as glm;
use wasm_bindgen::prelude::*;
use wgpu::{BufferUsages, Device, Queue, BindGroupDescriptor, BindGroupEntry, BufferDescriptor, ShaderModuleDescriptor, CommandEncoderDescriptor};
use wgpu::util::{DeviceExt, BufferInitDescriptor};

use crate::color::ColorRgbaU8;
use crate::asset_store::AssetStore;

#[wasm_bindgen]
pub struct GpuRenderer {
    scene: Scene,

    device: Device,
    queue: Queue,
    compute_pipeline_and_buffers: ComputePipelineAndBuffers,
}

struct ComputePipelineAndBuffers {
    canvas_staging_buf: wgpu::Buffer,
    canvas_storage_buf: wgpu::Buffer,
    camera_uniform_buf: wgpu::Buffer,
    _background_uniform_buf: wgpu::Buffer,

    // scene buffers
    _lights_storage_buf: wgpu::Buffer,
    _materials_storage_buf: wgpu::Buffer,
    _spheres_storage_buf: wgpu::Buffer,
    _triangles_storage_buf: wgpu::Buffer,

    compute_pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
}

#[wasm_bindgen]
impl GpuRenderer {
    pub async fn new(canvas_width: usize, canvas_height: usize,
                     asset_store: AssetStore, scene_file_name: &str) -> GpuRenderer {
        let Some(scene_bytes): Option<Vec<u8>> = asset_store.get_scene_bytes(scene_file_name) else {
            panic!("Loading scene '{scene_file_name}' was undefined")
        };
        let mut scene = Parser {
            file_reader: Cursor::new(scene_bytes),
            mesh_loader: &asset_store,
        }.parse_json().unwrap();
        scene.resize_screen(canvas_width, canvas_height);

        let (device, queue) = Self::get_device_and_queue().await.unwrap();

        let compute_pipeline_and_buffers = Self::setup_pipeline_and_buffers(&device, &scene);

        GpuRenderer {
            scene,

            device,
            queue,
            compute_pipeline_and_buffers,
        }
    }

    fn wgsl_array_storage_buffer_from_primitives<Primitive, GpuPrimitive>(label: Option<&str>, device: &Device, primitives: &[Primitive]) -> wgpu::Buffer where
        for<'p> &'p Primitive: Into<GpuPrimitive>,
        GpuPrimitive: Sized + bytemuck::Pod {

        // TODO: Why can't this be const here?
        // const gpu_primitive_size: usize = std::mem::size_of::<GpuPrimitive>();
        let gpu_primitive_size: usize = std::mem::size_of::<GpuPrimitive>();
        let mut gpu_bytes = Vec::with_capacity(gpu_primitive_size + gpu_primitive_size * primitives.len());
    
        gpu_bytes.resize(gpu_primitive_size, 0);
        for primitive in primitives {
            let gpu_primitive: GpuPrimitive = primitive.into();
            let as_slice = std::slice::from_ref(&gpu_primitive);
            let byte_slice = bytemuck::cast_slice(as_slice);
            gpu_bytes.extend(byte_slice.iter());
        }
    
        device.create_buffer_init(&BufferInitDescriptor {
            label,
            contents: gpu_bytes.as_slice(),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        })
    }

    fn setup_pipeline_and_buffers(device: &Device, scene: &Scene) -> ComputePipelineAndBuffers {
        // Loads the shader from WGSL
        let cs_module = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let canvas_size = scene.camera().screen_dimensions.x as usize * scene.camera().screen_dimensions.y as usize * std::mem::size_of::<ColorRgbaU8>();

        // Instantiates buffer without data.
        // `usage` of buffer specifies how it can be used:
        //   `BufferUsages::MAP_READ` allows it to be read (outside the shader).
        //   `BufferUsages::COPY_DST` allows it to be the destination of the copy.
        let canvas_staging_buf = device.create_buffer(&BufferDescriptor {
            label: Some("canvas_staging"),
            size: canvas_size as _,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let canvas_storage_buf = device.create_buffer(&BufferDescriptor {
            label: Some("canvas_storage"),
            size: canvas_size as _,
            // contents: bytemuck::cast_slice(canvas),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let gpu_camera = gpu_types::Camera::from(scene);
        let camera_uniform_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("camera_uniform"),
            contents: bytemuck::cast_slice(std::slice::from_ref(&gpu_camera)),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let gpu_background = gpu_types::Background::from(&scene.background);
        let background_uniform_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("background_uniform"),
            contents: bytemuck::cast_slice(std::slice::from_ref(&gpu_background)),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let lights_storage_buf = Self::wgsl_array_storage_buffer_from_primitives::<Light, gpu_types::Light>(Some("lights_storage"), device, &scene.lights);

        let materials_storage_buf = Self::wgsl_array_storage_buffer_from_primitives::<Material, gpu_types::Material>(Some("materials_storage"), device, &scene.materials);

        let spheres_storage_buf = Self::wgsl_array_storage_buffer_from_primitives::<Sphere, gpu_types::Sphere>(Some("spheres_storage"), device, &scene.spheres);

        let triangles_storage_buf = Self::wgsl_array_storage_buffer_from_primitives::<Triangle, gpu_types::Triangle>(Some("triangles_storage"), device, &scene.triangles);

        // A bind group defines how buffers are accessed by shaders.
        // It is to WebGPU what a descriptor set is to Vulkan.
        // `binding` here refers to the `binding` of a buffer in the shader (`layout(set = 0, binding = 0) buffer`).
    
        // A pipeline specifies the operation of a shader

        // Instantiates the pipeline.
        // TODO: evaluate using wgpu::PipelineCompilationOptions::constants
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &cs_module,
            entry_point: None,
            // TODO: evaluate using a cache
            cache: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });

        // Instantiates the bind group, once again specifying the binding of buffers.
        let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: canvas_storage_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: camera_uniform_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: background_uniform_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: lights_storage_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: materials_storage_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: spheres_storage_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: triangles_storage_buf.as_entire_binding(),
                },
            ],
        });

        ComputePipelineAndBuffers {
            canvas_staging_buf,
            canvas_storage_buf,
            camera_uniform_buf,
            _background_uniform_buf: background_uniform_buf,

            _lights_storage_buf: lights_storage_buf,
            _materials_storage_buf: materials_storage_buf,
            _spheres_storage_buf: spheres_storage_buf,
            _triangles_storage_buf: triangles_storage_buf,
            compute_pipeline,
            bind_group,
        }
    }

    async fn get_device_and_queue() -> Result<(Device, Queue), String> {
        let instance = wgpu::Instance::default();
    
        // `request_adapter` instantiates the general connection to the GPU
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .map_err(|_| "Didn't get adapter")?;
    
        // skip this on LavaPipe temporarily
        if adapter.get_info().vendor == 0x10005 {
            return Err("Adapter was wrong: LavaPipe".to_string());
        }
        
        // `request_device` instantiates the feature specific connection to the GPU, defining some parameters,
        //  `features` being the available features.
        adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(|err| err.to_string())
    }

    pub async fn render(&mut self, canvas_u8: &mut [u8]) {
        let millisecons = js_sys::Date::now();
        self.scene.update_camera(|camera| {
            camera.position.x = ((millisecons/200.0).cos()*0.1) as f32;
            camera.position.z = ((millisecons/200.0).sin()*0.1) as f32;
        });
        let canvas_dimensions = self.scene.camera().screen_dimensions;

        let ComputePipelineAndBuffers {
            canvas_staging_buf,
            canvas_storage_buf,
            camera_uniform_buf,
            _background_uniform_buf,

            _lights_storage_buf,
            _materials_storage_buf,
            _spheres_storage_buf,
            _triangles_storage_buf,
            compute_pipeline,
            bind_group,
        } = &self.compute_pipeline_and_buffers;
        
        // A command encoder executes one or many pipelines.
        // It is to WebGPU what a command buffer is to Vulkan.
        let mut encoder =
            self.device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            cpass.set_pipeline(compute_pipeline);
            cpass.set_bind_group(0, bind_group, &[]);
            cpass.insert_debug_marker("compute canvas");
            cpass.dispatch_workgroups(canvas_dimensions.x, canvas_dimensions.y, 1); // Number of cells to run, the (x,y,z) size of item being processed
        }
        // Sets adds copy operation to command encoder.
        // Will copy data from storage / canvas buffer on GPU to staging buffer on CPU.
        let canvas_size = (canvas_dimensions.x * canvas_dimensions.y) as usize * std::mem::size_of::<ColorRgbaU8>();
        encoder.copy_buffer_to_buffer(canvas_storage_buf, 0, canvas_staging_buf, 0, canvas_size as _);
        let command_buffer = encoder.finish();

        let gpu_camera = gpu_types::Camera::from(&self.scene);
        self.queue.write_buffer(camera_uniform_buf, 0, bytemuck::cast_slice(std::slice::from_ref(&gpu_camera)));
        self.queue.submit(Some(command_buffer));
    
        // // Note that we're not calling `.await` here.
        let buffer_slice = canvas_staging_buf.slice(..);
        // // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
        
        // // Poll the device in a blocking manner so that our future resolves.
        // // In an actual application, `device.poll(...)` should
        // // be called in an event loop or on another thread.
        self.device.poll(wgpu::PollType::Wait).unwrap();
    
        // // Awaits until `buffer_future` can be read from
        if let Some(Ok(())) = receiver.receive().await {
            // Gets contents of buffer
            let data = buffer_slice.get_mapped_range();
            // Since contents are got in bytes
            canvas_u8.copy_from_slice(&data);
    
            // With the current interface, we have to make sure all mapped views are
            // dropped before we unmap the buffer.
            drop(data);
            canvas_staging_buf.unmap(); // Unmaps buffer from memory
                                           // If you are familiar with C++ these 2 lines can be thought of similarly to:
                                           //   delete myPointer;
                                           //   myPointer = NULL;
                                           // It effectively frees the memory
        } else {
            panic!("failed to run compute on gpu!")
        }
    }
    
    pub fn resize_screen(&mut self, width: usize, height: usize) {
        self.scene.resize_screen(width, height);

        self.compute_pipeline_and_buffers = Self::setup_pipeline_and_buffers(&self.device, &self.scene);
    }

    pub fn turn_camera(&mut self, drag_begin_x: f32, drag_begin_y: f32, drag_end_x: f32, drag_end_y: f32) {
        self.scene.turn_camera(&glm::vec2(drag_begin_x, drag_begin_y), &glm::vec2(drag_end_x, drag_end_y))
    }
}
