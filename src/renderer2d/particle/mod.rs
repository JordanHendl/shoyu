mod json;
use glam::*;
use json::*;
use std::collections::HashMap;

use dashi::utils::*;
use dashi::*;

mod pipelines;
use pipelines::*;

use crate::database::load_funcs;
use crate::utils::{Canvas, HotBuffer, SizedImage, Timer};
#[repr(C)]
#[derive(Default, Clone)]
struct ShaderConfig {
    camera: Vec2,
    delta_time: f32,
}

#[repr(C)]
#[derive(Default, Clone)]
struct ShaderParticle {
    position: Vec2,
    size: Vec2,
    rot: f32,
    velocity: Vec2,
    tex_coords: Vec4,
    current_frame: u32,
    max_lifetime: f32,
    curr_lifetime: f32,
    behaviour: u32,
    active: u32,
}

pub enum ParticleBehaviour {
    LINEAR(u32),
    GRAVITY(u32),
}

pub struct ParticleEmitInfo {
    pub particle_id: u32,
    pub lifetime_ms: f32,
    pub amount: u32,
    pub position: glam::Vec2,
    pub initial_velocity: glam::Vec2,
    pub behaviour: ParticleBehaviour,
}

pub struct ParticleSystem {
    ctx: *mut Context,
    vertices: Handle<Buffer>,
    indices: Handle<Buffer>,
    particle_list_gpu: Handle<Buffer>,
    particle_list: &'static mut [ShaderParticle],
    curr_particle: u32,
    atlas: SizedImage,
    sampler: Handle<Sampler>,
    alloc: DynamicAllocator,
    draw_bg: Handle<BindGroup>,
    cfg: ParticleSystemJSON,
    pipelines: ParticlePipelineInfo,
    timer: Timer,
}

impl ParticleSystem {
    pub fn new(ctx: &mut Context, canvas: &Canvas, base_path: &str, particle_cfg: &str) -> Self {
        const MAX_PARTICLES: usize = 2048;
        let json_data =
            std::fs::read_to_string(format!("{}/{}", base_path, particle_cfg)).expect("Unable to load Particle System JSON!");
        let info: ParticleSystemJSON =
            serde_json::from_str(&json_data).expect("Unable to parse Particle System JSON!");

        let initial_data = vec![ShaderParticle::default(); MAX_PARTICLES];

        let particle_buffer = ctx
            .make_buffer(&BufferInfo {
                debug_name: "Particle System Buffers",
                byte_size: (std::mem::size_of::<ShaderParticle>() * MAX_PARTICLES) as u32,
                visibility: MemoryVisibility::CpuAndGpu,
                usage: BufferUsage::STORAGE,
                initial_data: Some(unsafe { &initial_data.align_to::<u8>().1 }),
            })
            .unwrap();

        drop(initial_data);
        let alloc = ctx.make_dynamic_allocator(&Default::default()).unwrap();
        let pipelines = pipelines::make_pipelines(ctx, canvas);

        let mut image: Option<SizedImage> = None;
        for particle in &info.particles {
            if image.is_none() {
                let img = load_funcs::load_image_rgba8(&format!("{}/{}", base_path, &particle.image_path)).unwrap();
                let gpu_img = ctx
                    .make_image(&ImageInfo {
                        debug_name: &particle.image_path,
                        dim: [img.size[0], img.size[1], 1],
                        format: Format::RGBA8,
                        mip_levels: 1,
                        initial_data: Some(&img.bytes),
                    })
                    .unwrap();

                let view = ctx
                    .make_image_view(&ImageViewInfo {
                        debug_name: &particle.image_path,
                        img: gpu_img,
                        layer: 0,
                        mip_level: 1,
                    })
                    .unwrap();

                image = Some(SizedImage {
                    handle: gpu_img,
                    view,
                    dim: [img.size[0], img.size[1], 1],
                    format: Format::RGBA8,
                });
            }
        }

        let sampler = ctx.make_sampler(&Default::default()).unwrap();

        let draw_bg = ctx
            .make_bind_group(&BindGroupInfo {
                layout: pipelines.bg_layout,
                bindings: &[
                    BindingInfo {
                        resource: ShaderResource::Dynamic(&alloc),
                        binding: 0,
                    },
                    BindingInfo {
                        resource: ShaderResource::Dynamic(&alloc),
                        binding: 1,
                    },
                    BindingInfo {
                        resource: ShaderResource::SampledImage(
                            image.as_ref().unwrap().view,
                            sampler,
                        ),
                        binding: 1,
                    },
                ],
                set: 0,
            })
            .unwrap();
        let raw_slice = ctx
            .map_buffer_mut::<ShaderParticle>(particle_buffer)
            .unwrap();
        let ptr = raw_slice.as_mut_ptr();

        let s_indices: [u32; 6] = [
            0, 1, 2, // First triangle
            2, 3, 0, // Second triangle
        ];

        let s_vertices = [
            // Top-left corner of the screen
            Vec2::new(-1.0, 1.0),
            // Bottom-left corner of the screen
            -Vec2::new(1.0, -1.0),
            // Bottom-right corner of the screen
            Vec2::new(1.0, -1.0),
            // Top-right corner of the screen
            Vec2::new(1.0, 1.0),
        ];
        
        assert!(size_of::<Vec2>() * s_vertices.len() == 32);
        let initial_data = unsafe { s_vertices.align_to::<u8>().1 };
        assert!(initial_data.len() == 32);
        let vertices = ctx
            .make_buffer(&BufferInfo {
                debug_name: "renderer2d-vertices",
                byte_size: (size_of::<Vec2>() *  s_vertices.len()) as u32,
                visibility: MemoryVisibility::Gpu,
                usage: BufferUsage::VERTEX,
                initial_data: unsafe { Some(s_vertices.align_to::<u8>().1) },
            })
            .unwrap();

        let indices = ctx
            .make_buffer(&BufferInfo {
                debug_name: "renderer2d-indices",
                byte_size: (size_of::<u32>() * s_indices.len()) as u32,
                visibility: MemoryVisibility::Gpu,
                usage: BufferUsage::INDEX,
                initial_data: unsafe { Some(s_indices.align_to::<u8>().1) },
            })
            .unwrap();

        Self {
            cfg: info,
            pipelines,
            timer: Timer::new(),
            ctx,
            particle_list: unsafe { std::slice::from_raw_parts_mut(ptr, MAX_PARTICLES) },
            particle_list_gpu: particle_buffer,
            curr_particle: 0,
            alloc,
            draw_bg,
            atlas: image.unwrap(),
            sampler,
            vertices,
            indices,
        }
    }
    
    pub fn emit(& mut self, info: &ParticleEmitInfo) {
        for 0..info.amount {

        }
    }

    // MUST NOT happen during an active render pass.
    // This is because this call performs compute dispatches.
    pub fn update(&mut self, cmd: &mut CommandList) {
        //        cmd.dispatch(&Dispatch {
        //            compute: self.pipelines.compute_pipeline,
        //            dynamic_buffers: [None, None, None, None],
        //            bind_groups: [None, None, None, None],
        //            workgroup_size: [2048 / 32, 1, 1],
        //        });
    }

    pub fn draw(&mut self, cmd: &mut CommandList) {
        let mut buff = self.alloc.bump().unwrap();

        let cfg = &mut buff.slice::<ShaderConfig>()[0];
        *cfg = ShaderConfig {
            camera: vec2(0.0, 0.0),
            delta_time: self.timer.elapsed_ms() as f32 / 1000.0,
        };

        self.timer.start();
        cmd.draw_indexed(&DrawIndexed {
            vertices: self.vertices,
            indices: self.indices,
            dynamic_buffers: [Some(buff), None, None, None],
            bind_groups: [Some(self.draw_bg), None, None, None],
            index_count: 6,
            instance_count: 2048,
            first_instance: 0,
        });
    }
}
