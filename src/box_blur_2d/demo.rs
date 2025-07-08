use clap::Args;
use wgpu::{ComputePipelineDescriptor, util::DeviceExt};

use crate::gpu_context::GpuContext;

#[derive(Args)]
pub struct Opts {
    /// Path of image to blur
    in_path: std::path::PathBuf,
    /// Output path
    out_path: std::path::PathBuf,
    /// Radius of box blur
    #[arg(default_value_t = 3)]
    radius: u32
}

fn blur(img: &image::RgbaImage, radius: u32, gpu_context: &GpuContext) -> anyhow::Result<image::RgbaImage> {
    let bind_group_layout =
        gpu_context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

    let pipeline_layout =
        gpu_context
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("pipeline layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

    let shader_module = gpu_context
        .device
        .create_shader_module(wgpu::include_wgsl!("box_blur_2d.wgsl"));

    let pipeline = gpu_context
        .device
        .create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("compute pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

    let uniforms_buffer =
        gpu_context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("uniforms buffer"),
                usage: wgpu::BufferUsages::UNIFORM,
                contents: bytemuck::bytes_of(&[img.width(), img.height(), radius]),
            });

    let img_buffer = gpu_context
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("img buffer"),
            usage: wgpu::BufferUsages::STORAGE,
            contents: img.as_raw(),
        });

    let out_buffer = gpu_context.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("out buffer"),
        size: img_buffer.size(),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let staging_buffer = gpu_context.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging buffer"),
        size: img_buffer.size(),
        usage: wgpu::BufferUsages::MAP_READ
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });


    let bind_group = gpu_context
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("img bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: img_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: out_buffer.as_entire_binding(),
                },
            ],
        });

    let mut encoder = gpu_context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("command encoder"),
        });

    let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("compute pass"),
        timestamp_writes: None,
    });

    compute_pass.set_pipeline(&pipeline);
    compute_pass.set_bind_group(0, Some(&bind_group), &[]);

    // assuming a workgroup size of 8, 8, 1
    // we need to dispatch width / 8 work groups in x and y
    // we use the ceil of this quantity to make sure we always have enough
    // work groups
    let wg_x = (img.width() + 8 - 1) / 8;
    let wg_y = (img.height() + 8 - 1) / 8;

    compute_pass.dispatch_workgroups(wg_x, wg_y, 1);
    drop(compute_pass);

    encoder.copy_buffer_to_buffer(&out_buffer, 0, &staging_buffer, 0, out_buffer.size());

    let submission_index = gpu_context.queue.submit([encoder.finish()]);
    gpu_context
        .device
        .poll(wgpu::PollType::WaitForSubmissionIndex(submission_index))?;

    // copy from buffer back to an image
    let (tx, rx) = std::sync::mpsc::channel();
    staging_buffer.map_async(wgpu::MapMode::Read, .., move |r| {
        tx.send(r).unwrap();
    });

    gpu_context.device.poll(wgpu::PollType::Wait)?;

    rx.recv()??;
    let staging_buffer_view = staging_buffer.get_mapped_range(..);
    let out_image = image::RgbaImage::from_vec(img.width(), img.height(), staging_buffer_view.to_vec())
        .ok_or(anyhow::anyhow!("Unable to convert GPU buffer to image"))?;

    drop(staging_buffer_view);
    staging_buffer.unmap();

    Ok(out_image)
}

pub fn demo(opts: Opts) -> anyhow::Result<()> {
    let dynamic_img = image::ImageReader::open(opts.in_path)?.decode()?;

    let img = dynamic_img.into_rgba8();

    let gpu_context = GpuContext::new(wgpu::Features::empty())?;

    let out_img = blur(&img, opts.radius, &gpu_context)?;

    out_img.save(opts.out_path)?;

    Ok(())
}
