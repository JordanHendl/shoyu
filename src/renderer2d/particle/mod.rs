mod json;
use glam::*;
use json::*;

use dashi::utils::*;
use dashi::*;

mod pipelines;
use pipelines::*;

use crate::database::load_funcs;
use crate::utils::{Canvas, SizedImage, Timer};
use rand::prelude::*;
#[repr(C)]
#[derive(Default, Clone)]
struct ShaderConfig {
    camera: Vec2,
    delta_time: f32,
}
#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
struct ParticleSpriteRegion {
    region: Vec4,
}

const MAX_PARTICLE_ANIMATIONS: usize = 128;
#[repr(C)]
#[derive(Clone, Copy)]
struct ParticleAnimation {
    time_per_frame_ms: f32,
    anim_count: u32,
    regions: [ParticleSpriteRegion; MAX_PARTICLE_ANIMATIONS],
}

impl Default for ParticleAnimation {
    fn default() -> Self {
        Self {
            time_per_frame_ms: Default::default(),
            anim_count: Default::default(),
            regions: [(); MAX_PARTICLE_ANIMATIONS].map(|_| ParticleSpriteRegion::default()),
        }
    }
}

fn convert_animation(value: &ParticleSystemJSONAnimation, dim: [u32; 3]) -> ParticleAnimation {
    let anim_count = value.sprites.len() as u32;
    let mapped: Vec<ParticleSpriteRegion> = value
        .sprites
        .clone()
        .into_iter()
        .map(|a| {
            return ParticleSpriteRegion {
                region: vec4(
                    a.x as f32 / dim[0] as f32,
                    a.y as f32 / dim[1] as f32,
                    a.w as f32 / dim[0] as f32,
                    a.h as f32 / dim[1] as f32,
                ),
            };
        })
        .collect();

    let mut default = [(); MAX_PARTICLE_ANIMATIONS].map(|_| ParticleSpriteRegion::default());

    for id in 0..mapped.len() {
        default[id] = mapped[id];
    }

    ParticleAnimation {
        time_per_frame_ms: value.time_per_frame_ms,
        anim_count,
        regions: default,
    }
}

#[repr(C, align(4))]
#[derive(Default, Clone)]
struct ShaderParticle {
    position: Vec2,
    size: Vec2,
    velocity: Vec2,
    particle_type: i32,
    rot: f32,
    current_frame: u32,
    max_lifetime: f32,
    curr_lifetime: f32,
    behaviour: u32,
    active: u32,
    anim_timer: f32,
    padding: Vec2,
}

#[derive(Copy, Clone)]
pub enum ParticleBehaviour {
    LINEAR = 0,
    GRAVITY = 1,
}

impl From<ParticleBehaviour> for u32 {
    fn from(value: ParticleBehaviour) -> Self {
        return value as u32;
    }
}

pub struct ParticleRandomEmitInfo {
    pub particle_id: u32,
    pub lifetime_ms: f32,
    pub amount: u32,
    pub center: Vec2,
    pub position_range: FRect2D,
    pub velocity_range: Vec2,
    pub behaviour: ParticleBehaviour,
}

pub struct ParticleEmitInfo {
    pub particle_id: u32,
    pub lifetime_ms: f32,
    pub amount: u32,
    pub position: glam::Vec2,
    pub size: Vec2,
    pub initial_velocity: glam::Vec2,
    pub behaviour: ParticleBehaviour,
}

fn random_offset(min: f32, max: f32) -> f32 {
    let _ = min;
    let mut rng = rand::thread_rng();
    let mut x = rng.gen::<f32>() * max;
    if rng.gen::<bool>() {
        x = -x;
    }

    x
}
#[allow(dead_code)]
pub struct ParticleSystem {
    ctx: *mut Context,
    dim: [f32; 2],
    vertices: Handle<Buffer>,
    indices: Handle<Buffer>,
    particle_list_gpu: Handle<Buffer>,
    particle_animations: Handle<Buffer>,
    particle_list: &'static mut [ShaderParticle],
    curr_particle: u32,
    atlas: SizedImage,
    sampler: Handle<Sampler>,
    alloc: DynamicAllocator,
    draw_bg: Handle<BindGroup>,
    compute_bg: Handle<BindGroup>,
    pipelines: ParticlePipelineInfo,
    timer: Timer,
}

impl ParticleSystem {
    pub fn new(ctx: &mut Context, canvas: &Canvas, base_path: &str, particle_cfg: &str) -> Self {
        const _TEST_CHECKER: [u8; 64] = [0; std::mem::size_of::<ShaderParticle>()];

        const MAX_PARTICLES: usize = 2048;
        let json_data = std::fs::read_to_string(format!("{}/{}", base_path, particle_cfg))
            .expect("Unable to load Particle System JSON!");
        let info: ParticleSystemJSON =
            serde_json::from_str(&json_data).expect("Unable to parse Particle System JSON!");

        // Parse particle info
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

        //        let particle_animations =
        let mut image: Option<SizedImage> = None;
        let mut animations: Vec<ParticleAnimation> = Vec::with_capacity(MAX_PARTICLE_ANIMATIONS);
        animations.resize(MAX_PARTICLE_ANIMATIONS, Default::default());

        for particle in &info.particles {
            if image.is_none() {
                let img = load_funcs::load_image_rgba8(&format!(
                    "{}/{}",
                    base_path, &particle.image_path
                ))
                .unwrap();
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
                        layer: 1,
                        mip_level: 0,
                    })
                    .unwrap();

                image = Some(SizedImage {
                    handle: gpu_img,
                    view,
                    dim: [img.size[0], img.size[1], 1],
                    format: Format::RGBA8,
                });
            }

            for anim in &particle.animations {
                animations[particle.id as usize] =
                    convert_animation(anim, image.as_ref().unwrap().dim);
            }
        }

        let sampler = ctx.make_sampler(&Default::default()).unwrap();

        let particle_anim_buffer = ctx
            .make_buffer(&BufferInfo {
                debug_name: "Particle Animation Info",
                byte_size: size_of::<ParticleAnimation>() as u32 * MAX_PARTICLE_ANIMATIONS as u32,
                visibility: MemoryVisibility::Gpu,
                usage: BufferUsage::STORAGE,
                initial_data: Some(unsafe { &animations.align_to::<u8>().1 }),
            })
            .unwrap();

        let compute_bg = ctx
            .make_bind_group(&BindGroupInfo {
                debug_name: "Particle System Compute BG",
                layout: pipelines.compute_bg_layout,
                bindings: &[
                    BindingInfo {
                        resource: ShaderResource::StorageBuffer(particle_anim_buffer),
                        binding: 0,
                    },
                    BindingInfo {
                        resource: ShaderResource::StorageBuffer(particle_buffer),
                        binding: 1,
                    },
                    BindingInfo {
                        resource: ShaderResource::Dynamic(&alloc),
                        binding: 2,
                    },
                    BindingInfo {
                        resource: ShaderResource::Dynamic(&alloc),
                        binding: 3,
                    },
                    BindingInfo {
                        resource: ShaderResource::SampledImage(
                            image.as_ref().unwrap().view,
                            sampler,
                        ),
                        binding: 4,
                    },
                ],
                set: 0,
            })
            .unwrap();

        let draw_bg = ctx
            .make_bind_group(&BindGroupInfo {
                debug_name: "Particle System Main Buffer",
                layout: pipelines.bg_layout,
                bindings: &[
                    BindingInfo {
                        resource: ShaderResource::StorageBuffer(particle_anim_buffer),
                        binding: 0,
                    },
                    BindingInfo {
                        resource: ShaderResource::StorageBuffer(particle_buffer),
                        binding: 1,
                    },
                    BindingInfo {
                        resource: ShaderResource::Dynamic(&alloc),
                        binding: 2,
                    },
                    BindingInfo {
                        resource: ShaderResource::Dynamic(&alloc),
                        binding: 3,
                    },
                    BindingInfo {
                        resource: ShaderResource::SampledImage(
                            image.as_ref().unwrap().view,
                            sampler,
                        ),
                        binding: 4,
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
            Vec2::new(-1.0, -1.0),
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
                byte_size: (size_of::<Vec2>() * s_vertices.len()) as u32,
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
            pipelines,
            dim: [canvas.viewport().area.w, canvas.viewport().area.h],
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
            particle_animations: particle_anim_buffer,
            compute_bg,
        }
    }

    pub fn emit_random(&mut self, info: &ParticleEmitInfo) {
        let mut rng = rand::thread_rng();

        let pos = super::screen_to_vulkan(info.position, self.dim[0], self.dim[1]);
                for id in self.curr_particle..self.curr_particle + info.amount {
            if (id as usize) < self.particle_list.len() {
                let pos = vec2(
                    pos.x() + random_offset(0.0, 0.2),
                    pos.y() + random_offset(0.0, 0.2),
                );
                self.particle_list[id as usize] = ShaderParticle {
                    position: pos,
                    size: vec2(0.1, 0.1),
                    rot: rng.gen::<f32>(),
                    velocity: vec2(
                        (rng.gen::<f32>() - 0.5) * 5.0,
                        (rng.gen::<f32>() - 0.5) * 5.0,
                    ),
                    current_frame: 0,
                    max_lifetime: info.lifetime_ms,
                    curr_lifetime: 0.0,
                    behaviour: info.behaviour.into(),
                    active: 1,
                    particle_type: info.particle_id as i32,
                    padding: Default::default(),
                    ..Default::default()
                };
            }
        }

        self.curr_particle += info.amount;
        if self.curr_particle > self.particle_list.len() as u32 {
            self.curr_particle = 0;
        }
    }

    pub fn emit(&mut self, info: &ParticleEmitInfo) {
        let pos = super::screen_to_vulkan(info.position, self.dim[0], self.dim[1]);
        let size = super::screen_to_normalized(info.size, self.dim[0], self.dim[1]);
        for id in self.curr_particle..self.curr_particle + info.amount {
            if (id as usize) < self.particle_list.len() {
                self.particle_list[id as usize] = ShaderParticle {
                    position: pos,
                    size,
                    rot: 0.0,
                    velocity: vec2(0.0, 0.0),
                    current_frame: 0,
                    max_lifetime: info.lifetime_ms,
                    curr_lifetime: 0.0,
                    behaviour: info.behaviour.into(),
                    active: 1,
                    particle_type: info.particle_id as i32,
                    padding: Default::default(),
                    ..Default::default()
                };
            }
        }

        self.curr_particle += info.amount;
        if self.curr_particle > self.particle_list.len() as u32 {
            self.curr_particle = 0;
        }
    }

    // MUST NOT happen during an active render pass.
    // This is because this call performs compute dispatches.
    pub fn update(&mut self, cmd: &mut FramedCommandList) {
        self.alloc.reset();
        let mut buff = self.alloc.bump().unwrap();
        let cfg = &mut buff.slice::<ShaderConfig>()[0];
        *cfg = ShaderConfig {
            camera: vec2(0.0, 0.0),
            delta_time: self.timer.elapsed_ms() as f32 / 1000.0,
        };
        self.timer.stop();
        self.timer.start();

        cmd.record(|cmd| {
            cmd.dispatch(&Dispatch {
                compute: self.pipelines.compute_pipeline,
                dynamic_buffers: [Some(buff), Some(buff), None, None],
                bind_groups: [Some(self.compute_bg), None, None, None],
                workgroup_size: [2048 / 32, 1, 1],
            });
        });
    }

    pub fn draw(&mut self, cmd: &mut FramedCommandList) {
        let mut buff = self.alloc.bump().unwrap();
        let mut buff2 = self.alloc.bump().unwrap();

        let pos = &mut buff2.slice::<Mat4>()[0];
        let cfg = &mut buff.slice::<ShaderConfig>()[0];
        *pos = Mat4::identity();
        *cfg = ShaderConfig {
            camera: vec2(0.0, 0.0),
            delta_time: self.timer.elapsed_ms() as f32 / 1000.0,
        };

        cmd.append(|cmd| {
            cmd.begin_drawing(&DrawBegin {
                viewport: Viewport::default(),
                pipeline: self.pipelines.pipeline,
            })
            .unwrap();

            cmd.draw_indexed(&DrawIndexed {
                vertices: self.vertices,
                indices: self.indices,
                dynamic_buffers: [Some(buff2), Some(buff), None, None],
                bind_groups: [Some(self.draw_bg), None, None, None],
                index_count: 6,
                instance_count: 2048,
                first_instance: 0,
            });
        });
    }
}
