use std::collections::HashMap;

use super::pipeline;
use super::types::*;
use crate::database::*;
use crate::utils::Canvas;
use dashi::utils::*;
use dashi::*;
pub struct ResourceManager {
    ctx: *mut Context,
    allocator: DynamicAllocator,
    vertices: Handle<Buffer>,
    indices: Handle<Buffer>,
    canvas: Canvas,
    database: Database,
    sprites: Pool<Sprite>,
    fonts: Pool<Font>,
    gfx: pipeline::GraphicsPipelineInfo,
    sampler: Handle<Sampler>,
    sprite_sheets: Pool<SpriteSheet>,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

impl ResourceManager {
    pub fn new(ctx: &mut Context, canvas: Canvas, database: Database) -> Self {
        let s_vertices = [
            // Top-left corner of the screen
            Vertex {
                position: [-1.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            // Bottom-left corner of the screen
            Vertex {
                position: [-1.0, -1.0],
                tex_coords: [0.0, 0.0],
            },
            // Bottom-right corner of the screen
            Vertex {
                position: [1.0, -1.0],
                tex_coords: [1.0, 0.0],
            },
            // Top-right corner of the screen
            Vertex {
                position: [1.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
        ];

        let s_indices: [u32; 6] = [
            0, 1, 2, // First triangle
            2, 3, 0, // Second triangle
        ];

        let vertices = ctx
            .make_buffer(&BufferInfo {
                debug_name: "renderer2d-vertices",
                byte_size: (size_of::<Vertex>() * s_vertices.len()) as u32,
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

        let allocator = ctx
            .make_dynamic_allocator(&DynamicAllocatorInfo {
                debug_name: "renderer2d alloc",
                usage: BufferUsage::ALL,
                byte_size: 80000000,
                num_allocations: 8080,
            })
            .unwrap();

        let gfx = pipeline::make_graphics_pipeline(ctx, &canvas);
        let sampler = ctx
            .make_sampler(&SamplerInfo {
                border_color: BorderColor::TransparentBlack,
                min_filter: Filter::Nearest,
                mag_filter: Filter::Nearest,
                mipmap_mode: SamplerMipmapMode::Nearest,
                ..Default::default()
            })
            .expect("Unable to make sampler!");

        Self {
            ctx,
            sampler,
            database,
            sprites: Default::default(),
            sprite_sheets: Default::default(),
            fonts: Default::default(),
            vertices,
            indices,
            allocator,
            canvas,
            gfx,
        }
    }

    pub fn canvas(&self) -> &Canvas {
        return &self.canvas;
    }

    pub fn gfx(&self) -> &pipeline::GraphicsPipelineInfo {
        &self.gfx
    }

    pub fn allocator(&mut self) -> &mut DynamicAllocator {
        &mut self.allocator
    }

    pub fn vertices(&self) -> Handle<Buffer> {
        self.vertices
    }

    pub fn indices(&self) -> Handle<Buffer> {
        self.indices
    }

    pub fn fetch_sprite(&mut self, handle: Handle<Sprite>) -> Option<&mut Sprite> {
        Some(self.sprites.get_mut_ref(handle)?)
    }

    pub fn fetch_font(&mut self, handle: Handle<Font>) -> Option<&mut Font> {
        Some(self.fonts.get_mut_ref(handle)?)
    }

    pub fn fetch_sprite_sheet(&mut self, handle: Handle<SpriteSheet>) -> Option<&mut SpriteSheet> {
        Some(self.sprite_sheets.get_mut_ref(handle)?)
    }

    pub fn make_font(&mut self, info: &FontInfo) -> Handle<Font> {
        let img: *const TTFont = self
            .database
            .fetch_ttf(info.db_key)
            .unwrap()
            .loaded
            .as_ref()
            .unwrap();

        unsafe {
            let size = ((*img).atlas_width, (*img).atlas_height);
            let initial_data = (*img).atlas.as_ref().unwrap();
            let spr = (*self.ctx)
                .make_image(&ImageInfo {
                    debug_name: info.name,
                    dim: [size.0, size.1, 1],
                    format: Format::R8_UINT,
                    mip_levels: 1,
                    initial_data: Some(initial_data),
                })
                .unwrap();

            let spr_view = (*self.ctx)
                .make_image_view(&ImageViewInfo {
                    debug_name: info.name,
                    img: spr,
                    ..Default::default()
                })
                .unwrap();

            return self
                .fonts
                .insert(Font {
                    dim: [size.0, size.1],
                    atlas: spr,
                    atlas_view: spr_view,
                    bg: (*self.ctx)
                        .make_bind_group(&BindGroupInfo {
                            debug_name: info.name,
                            layout: self.gfx.text_bg_layout,
                            bindings: &[
                                BindingInfo {
                                    resource: ShaderResource::Dynamic(&self.allocator),
                                    binding: 1,
                                },
                                BindingInfo {
                                    resource: ShaderResource::SampledImage(spr_view, self.sampler),
                                    binding: 2,
                                },
                            ],
                            ..Default::default()
                        })
                        .unwrap(),
                    font: img,
                })
                .unwrap();
        }
    }

    pub fn make_sprite(&mut self, info: &SpriteInfo) -> Handle<Sprite> {
        let img = self
            .database
            .fetch_sprite(info.db_key)
            .unwrap()
            .loaded
            .as_ref()
            .unwrap();
        unsafe {
            let spr = (*self.ctx)
                .make_image(&ImageInfo {
                    debug_name: info.name,
                    dim: [img.size[0], img.size[1], 1],
                    format: img.format,
                    mip_levels: 1,
                    initial_data: Some(&img.bytes),
                })
                .unwrap();

            let spr_view = (*self.ctx)
                .make_image_view(&ImageViewInfo {
                    debug_name: info.name,
                    img: spr,
                    ..Default::default()
                })
                .unwrap();

            return self
                .sprites
                .insert(Sprite {
                    dim: [img.size[0], img.size[1]],
                    handle: spr,
                    view: spr_view,
                    bg: (*self.ctx)
                        .make_bind_group(&BindGroupInfo {
                            debug_name: info.name,
                            layout: self.gfx.bg_layout,
                            bindings: &[
                                BindingInfo {
                                    resource: ShaderResource::Dynamic(&self.allocator),
                                    binding: 0,
                                },
                                BindingInfo {
                                    resource: ShaderResource::Dynamic(&self.allocator),
                                    binding: 1,
                                },
                                BindingInfo {
                                    resource: ShaderResource::SampledImage(spr_view, self.sampler),
                                    binding: 2,
                                },
                            ],
                            ..Default::default()
                        })
                        .unwrap(),
                })
                .unwrap();
        }
    }

    pub fn make_sprite_sheet(&mut self, info: &SpriteSheetInfo) -> Handle<SpriteSheet> {
        let mut hashed = HashMap::new();
        {
            let dim = self
                .database
                .fetch_sprite_sheet(info.db_key)
                .unwrap()
                .loaded
                .as_ref()
                .unwrap()
                .size;

            let sprites = &self.database.fetch_sprite_sheet(info.db_key).unwrap().cfg;
            if let Some(spr) = sprites.sprites.as_ref() {
                hashed = spr
                    .into_iter()
                    .map(|x| {
                        (
                            x.id,
                            FRect2D {
                                x: x.bounds.x as f32 / dim[0] as f32,
                                y: x.bounds.y as f32 / dim[1] as f32,
                                w: x.bounds.x as f32 / dim[0] as f32 + x.bounds.w as f32 / dim[0] as f32,
                                h: x.bounds.y as f32 / dim[0] as f32 + x.bounds.h as f32 / dim[1] as f32,
                            },
                        )
                    })
                    .collect::<HashMap<u32, FRect2D>>();
            }
        }
        assert!(!hashed.is_empty());

        unsafe {
            let img = self
                .database
                .fetch_sprite_sheet(info.db_key)
                .unwrap()
                .loaded
                .as_ref()
                .unwrap();

            let spr = (*self.ctx)
                .make_image(&ImageInfo {
                    debug_name: info.name,
                    dim: [img.size[0], img.size[1], 1],
                    format: img.format,
                    mip_levels: 1,
                    initial_data: Some(&img.bytes),
                })
                .unwrap();

            let spr_view = (*self.ctx)
                .make_image_view(&ImageViewInfo {
                    debug_name: info.name,
                    img: spr,
                    ..Default::default()
                })
                .unwrap();

            return self
                .sprite_sheets
                .insert(SpriteSheet {
                    dim: [img.size[0], img.size[1]],
                    handle: spr,
                    sprites: hashed,
                    view: spr_view,
                    bg: (*self.ctx)
                        .make_bind_group(&BindGroupInfo {
                            debug_name: info.name,
                            layout: self.gfx.bg_layout,
                            bindings: &[
                                BindingInfo {
                                    resource: ShaderResource::Dynamic(&self.allocator),
                                    binding: 0,
                                },
                                BindingInfo {
                                    resource: ShaderResource::Dynamic(&self.allocator),
                                    binding: 1,
                                },
                                BindingInfo {
                                    resource: ShaderResource::SampledImage(spr_view, self.sampler),
                                    binding: 2,
                                },
                            ],
                            ..Default::default()
                        })
                        .unwrap(),
                })
                .unwrap();
        }
    }

    pub fn release_sprite(&mut self, handle: Handle<Sprite>) {}

    pub fn release_sprite_sheet(&mut self, handle: Handle<SpriteSheet>) {}
}
