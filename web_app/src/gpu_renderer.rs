use std::borrow::Cow;

use lib_raytracer::raytracing::transform::matrix;
use nalgebra_glm as glm;
use wasm_bindgen::prelude::wasm_bindgen;

use lib_raytracer::raytracing::{Screen, Camera};
use wgpu::{BufferUsages, Device, Queue};
use wgpu::util::DeviceExt;

use crate::color::ColorRgbaU8;

#[wasm_bindgen]
pub struct GpuRenderer {
    camera: Camera,
    screen: Screen,
    screen_to_world: glm::Mat4,

    device: Device,
    queue: Queue,
    compute_pipeline_and_buffers: ComputePipelineAndBuffers,
}

struct ComputePipelineAndBuffers {
    canvas_staging_buf: wgpu::Buffer,
    canvas_storage_buf: wgpu::Buffer,
    screen_dimensions_uniform_buf: wgpu::Buffer,
    screen_to_world_uniform_buf: wgpu::Buffer,
    compute_pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
}

#[wasm_bindgen]
impl GpuRenderer {
    pub async fn new(canvas_width: usize, canvas_height: usize) -> GpuRenderer {
        let camera = Camera {
            position: glm::vec3(0.0, 0.0, 0.0),
            orientation: glm::vec3(0.0, 0.0, 0.0),
            y_fov_degrees: 90.0,
            z_near: 0.1,
            z_far: 25.0,
        };

        let screen = Screen {
            pixel_width: canvas_width,
            pixel_height: canvas_height,
            background: glm::vec3(1., 1., 1.),
        };

        let screen_to_world = matrix::screen_to_world(&camera, &screen);

        let (device, queue) = Self::get_device_and_queue().await.unwrap();

        let compute_pipeline_and_buffers = Self::setup_compute_pipeline(&device, &screen, &screen_to_world);

        GpuRenderer {
            camera,
            screen,
            screen_to_world,

            device,
            queue,
            compute_pipeline_and_buffers,
        }
    }

    fn setup_compute_pipeline(device: &Device, screen: &Screen, screen_to_world: &glm::Mat4) -> ComputePipelineAndBuffers {
        // Loads the shader from WGSL
        let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });
    
        let canvas_size = screen.pixel_width * screen.pixel_height * std::mem::size_of::<ColorRgbaU8>();
        
        // Instantiates buffer without data.
        // `usage` of buffer specifies how it can be used:
        //   `BufferUsages::MAP_READ` allows it to be read (outside the shader).
        //   `BufferUsages::COPY_DST` allows it to be the destination of the copy.
        let canvas_staging_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("canvas_staging"),
            size: canvas_size as _,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let canvas_storage_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("canvas_storage"),
            size: canvas_size as _,
            // contents: bytemuck::cast_slice(canvas),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
    
        let canvas_dimensions = glm::vec2(
            screen.pixel_width as u32,
            screen.pixel_height as u32
        );
        let screen_dimensions_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("screen_dimensions_uniform"),
            contents: bytemuck::cast_slice(canvas_dimensions.as_slice()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
    
        let screen_to_world_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mat_screen_to_world_uniform"),
            contents: bytemuck::cast_slice(screen_to_world.as_slice()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
    
        // A bind group defines how buffers are accessed by shaders.
        // It is to WebGPU what a descriptor set is to Vulkan.
        // `binding` here refers to the `binding` of a buffer in the shader (`layout(set = 0, binding = 0) buffer`).
    
        // A pipeline specifies the operation of a shader
    
        // Instantiates the pipeline.
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &cs_module,
            entry_point: "main",
        });
    
        // Instantiates the bind group, once again specifying the binding of buffers.
        let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: canvas_storage_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: screen_dimensions_uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: screen_to_world_uniform_buf.as_entire_binding(),
                },
            ],
        });

        ComputePipelineAndBuffers {
            canvas_staging_buf,
            canvas_storage_buf,
            screen_dimensions_uniform_buf,
            screen_to_world_uniform_buf,
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
            .ok_or("Didn't get adapter")?;
    
        // skip this on LavaPipe temporarily
        if adapter.get_info().vendor == 0x10005 {
            return Err("Adapter was wrong: LavaPipe".to_string());
        }
        
        // `request_device` instantiates the feature specific connection to the GPU, defining some parameters,
        //  `features` being the available features.
        adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await
            .map_err(|err| err.to_string())
    }

    pub async fn render(&self, canvas_u8: &mut [u8]) {
        // TODO: Put background into: SolidColor, NormalColor, EnvironmentTexture
        let canvas_dimensions = glm::vec2(
            self.screen.pixel_width,
            self.screen.pixel_height,
        );

        let ComputePipelineAndBuffers {
            canvas_staging_buf,
            canvas_storage_buf,
            screen_dimensions_uniform_buf,
            screen_to_world_uniform_buf,
            compute_pipeline,
            bind_group,
        } = &self.compute_pipeline_and_buffers;
        
        // A command encoder executes one or many pipelines.
        // It is to WebGPU what a command buffer is to Vulkan.
        let mut encoder =
            self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(compute_pipeline);
            cpass.set_bind_group(0, bind_group, &[]);
            cpass.insert_debug_marker("compute canvas");
            cpass.dispatch_workgroups(canvas_dimensions.x as _, canvas_dimensions.y as _, 1); // Number of cells to run, the (x,y,z) size of item being processed
        }
        // Sets adds copy operation to command encoder.
        // Will copy data from storage / canvas buffer on GPU to staging buffer on CPU.
        let canvas_size = canvas_dimensions.x * canvas_dimensions.y * std::mem::size_of::<ColorRgbaU8>();
        encoder.copy_buffer_to_buffer(canvas_storage_buf, 0, canvas_staging_buf, 0, canvas_size as _);
        let command_buffer = encoder.finish();

        self.queue.write_buffer(screen_to_world_uniform_buf, 0, bytemuck::cast_slice(self.screen_to_world.as_slice()));
        self.queue.write_buffer(screen_dimensions_uniform_buf, 0, bytemuck::cast_slice(canvas_dimensions.as_slice()));
        self.queue.submit(Some(command_buffer));
    
        // // Note that we're not calling `.await` here.
        let buffer_slice = canvas_staging_buf.slice(..);
        // // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
        
        // // Poll the device in a blocking manner so that our future resolves.
        // // In an actual application, `device.poll(...)` should
        // // be called in an event loop or on another thread.
        self.device.poll(wgpu::Maintain::Wait);
    
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
        self.screen.pixel_width = width;
        self.screen.pixel_height = height;

        self.screen_to_world = matrix::screen_to_world(&self.camera, &self.screen);

        self.compute_pipeline_and_buffers = Self::setup_compute_pipeline(&self.device, &self.screen, &self.screen_to_world);
    }

    pub fn turn_camera(&mut self, drag_begin_x: f32, drag_begin_y: f32, drag_end_x: f32, drag_end_y: f32) {
        let begin: glm::Vec2 = glm::vec2(drag_begin_x, drag_begin_y);
        let end: glm::Vec2 = glm::vec2(drag_end_x, drag_end_y);
        let radians = |degrees: f32| degrees * (glm::pi::<f32>() / 180.0);

        // pixel to degrees mapping
        let y_fov_degrees = self.camera.y_fov_degrees;
        let degrees_per_pixel = y_fov_degrees / self.screen.pixel_height as f32;
        let pixel_to_angle = |pixel| radians(pixel * degrees_per_pixel);

        let pixel_diff_x = end.x - begin.x;
        let pixel_diff_y = end.y - begin.y;

        let angle_diff_heading = pixel_to_angle(pixel_diff_x);
        let angle_diff_pitch = pixel_to_angle(pixel_diff_y);

        // "natural scrolling" - turning follows the inverse cursor motion
        // the heading turn is positive when turning to the left -> when drag_begin is left of drag_end
        let angle_diff_heading = match begin.x < end.x {
            true => angle_diff_heading.abs(),
            false => -angle_diff_heading.abs()
        };
        // the pitch turn is positive when turning upwards -> when drag_begin is above drag_end
        let angle_diff_pitch = match begin.y > end.y {
            true => angle_diff_pitch.abs(),
            false => -angle_diff_pitch.abs()
        };

        let camera_orientation = &mut self.camera.orientation;
        camera_orientation.x += angle_diff_pitch;
        camera_orientation.y += angle_diff_heading;

        // clamp pitch
        camera_orientation.x = camera_orientation.x.clamp(radians(-90.),
                                                          radians(90.));
        // modulo heading
        camera_orientation.y %= radians(360.);

        self.screen_to_world = matrix::screen_to_world(&self.camera, &self.screen);
    }
}
