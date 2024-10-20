pub mod types;
use glam::{vec2, vec4};
pub use types::*;
pub mod resource_manager;
pub use resource_manager::*;

pub use dashi::utils::*;
pub use dashi::*;

use crate::database::Database;
use crate::utils::Canvas;
mod pipeline;

pub struct Renderer2D {
    display: Display,
    manager: ResourceManager,
    cmd: CommandList,
    ctx: *mut Context,
    display_img: Handle<ImageView>,
    display_sem: Semaphore,
}

pub struct SpriteDrawCommand {
    pub sprite: Handle<Sprite>,
    pub position: glam::Vec2,
    pub size: glam::Vec2,
    pub rotation: f32,
}

pub struct SpriteSheetDrawCommand {
    pub sheet: Handle<SpriteSheet>,
    pub sprite_id: u32,
    pub position: glam::Vec2,
    pub size: glam::Vec2,
    pub rotation: f32,
}

pub struct TextDrawCommand<'a> {
    pub font: Handle<Font>,
    pub position: glam::Vec2,
    pub scale: f32,
    pub text: &'a str,
    pub color: glam::Vec4,
}

impl<'a> Default for TextDrawCommand<'a> {
    fn default() -> Self {
        Self {
            font: Default::default(),
            position: Default::default(),
            scale: Default::default(),
            text: Default::default(),
            color: vec4(1.0, 1.0, 1.0, 1.0),
        }
    }
}
impl Renderer2D {
    pub fn new(ctx: &mut Context, database: Database, canvas: Canvas) -> Self {
        let display = ctx
            .make_display(&DisplayInfo {
                window: WindowInfo {
                    title: canvas.name(),
                    size: [
                        canvas.viewport().area.w as u32,
                        canvas.viewport().area.h as u32,
                    ],
                    resizable: false,
                },
                vsync: true,
                ..Default::default()
            })
            .unwrap();

        let manager = ResourceManager::new(ctx, canvas, database);

        Self {
            cmd: Default::default(),
            display,
            manager,
            ctx,
            display_img: Default::default(),
            display_sem: Default::default(),
        }
    }
    pub fn begin_drawing(&mut self) {
        self.manager.allocator().reset();
        let (img, sem, _idx, _good) =
            unsafe { (*self.ctx).acquire_new_image(&mut self.display).unwrap() };

        self.display_img = img;
        self.display_sem = sem;

        self.cmd = unsafe { (*self.ctx).begin_command_list(&Default::default()).unwrap() };
        self.cmd
            .begin_drawing(&DrawBegin {
                viewport: self.manager.canvas().viewport(),
                pipeline: self.manager.gfx().pipeline,
            })
            .unwrap();
    }

    pub fn finish_drawing(&mut self) {
        unsafe {
            self.cmd.end_drawing().expect("Error ending drawing!");

            // Blit the framebuffer to the display's image
            self.cmd.blit(ImageBlit {
                src: self.manager.canvas().color_attachment(0),
                dst: self.display_img,
                filter: Filter::Nearest,
                ..Default::default()
            });

            // Submit our recorded commands
            let (sem, fence) = (*self.ctx)
                .submit(&mut self.cmd, Some(&[self.display_sem]))
                .unwrap();

            // Present the display image, waiting on the semaphore that will signal when our
            // drawing/blitting is done.
            (*self.ctx).present_display(&self.display, &[sem]).unwrap();

            // Signal the context to free our command list on the next submit call. This is nice so
            // that we don't have to manually manage it.
            (*self.ctx).release_list_on_next_submit(fence, self.cmd.clone());
        }
    }
    pub fn resources(&mut self) -> &mut ResourceManager {
        return &mut self.manager;
    }

    pub fn draw_text(&mut self, cmd: &TextDrawCommand) {
        #[repr(C)]
        #[derive(Clone, Copy)]
        struct TextVertex {
            pos: glam::Vec2,
            tex: glam::Vec2,
        }

        let font_handle = self.manager.fetch_font(cmd.font).unwrap();
        let font_bg = font_handle.bg;
        let font = font_handle.font;
        let dim = font_handle.dim;
        self.cmd
            .begin_drawing(&DrawBegin {
                viewport: self.manager.canvas().viewport(),
                pipeline: self.manager.gfx().text_pipeline,
            })
            .unwrap();

        let mut xpos = cmd.position.x();
        let ypos = cmd.position.y();
        for ch in cmd.text.chars() {
            unsafe {
                if let Some(g) = (*font).glyphs.get(&ch) {
                    let mut vert_alloc = self.manager.allocator().bump().unwrap();
                    let mut info = self.manager.allocator().bump().unwrap();
                    let vertices = vert_alloc.slice::<TextVertex>().split_at_mut(4).0;
                    let color = info.slice::<glam::Vec4>();

                    color[0] = cmd.color;

                    let scale = cmd.scale;
                    let gw = g.bounds.w as f32 / dim[0] as f32;
                    let gh = g.bounds.h as f32 / dim[1] as f32;

                    let x0 = (scale * (xpos)) - 1.0;
                    let y0 = (scale * (ypos - gh - g.bearing_y)) - 1.0;
                    let x1 = (scale * ((xpos) + (g.bounds.w as f32 / dim[0] as f32))) - 1.0;
                    let y1 = (scale * ((ypos - gh - g.bearing_y) + gh)) - 1.0;

                    let tex_x0 = (g.bounds.x as f32 / dim[0] as f32) as f32;
                    let tex_y0 = (g.bounds.y as f32 / dim[1] as f32) as f32;

                    let tex_x1 = tex_x0 + gw;
                    let tex_y1 = tex_y0 + gh;

                    vertices.copy_from_slice(&[
                        TextVertex {
                            pos: vec2(x1, y1),
                            tex: vec2(tex_x1, tex_y1),
                        },
                        TextVertex {
                            pos: vec2(x0, y1),
                            tex: vec2(tex_x0, tex_y1),
                        },
                        TextVertex {
                            pos: vec2(x0, y0),
                            tex: vec2(tex_x0, tex_y0),
                        },
                        TextVertex {
                            pos: vec2(x1, y0),
                            tex: vec2(tex_x1, tex_y0),
                        },
                    ]);

                    xpos += g.advance;
                    self.cmd.draw_dynamic_indexed(&DrawIndexedDynamic {
                        vertices: vert_alloc,
                        indices: self.manager.indices().to_unmapped_dynamic(0),
                        dynamic_buffers: [Some(info), None, None, None],
                        bind_groups: [Some(font_bg), None, None, None],
                        index_count: 6,
                        ..Default::default()
                    });
                }
            }
        }
    }
    pub fn draw_sprite(&mut self, cmd: &SpriteDrawCommand) {
        let mut b1 = self.manager.allocator().bump().unwrap();
        let mut b2 = self.manager.allocator().bump().unwrap();
        let transform = &mut b1.slice::<glam::Mat4>()[0];
        let camera = &mut b2.slice::<glam::Vec2>()[0];

        let angle_radian = cmd.rotation.to_radians();
        let scale = glam::Mat4::from_scale(glam::Vec3::new(cmd.size.x(), cmd.size.y(), 1.0));
        let translate =
            glam::Mat4::from_translation(glam::Vec3::new(cmd.position.x(), cmd.position.y(), 0.0));
        let rotate = glam::Mat4::from_rotation_z(angle_radian);

        let t = translate * rotate * scale;
        *transform = t;
        *camera = glam::Vec2::new(0.0, 0.0);
        let sprite_bg = self.manager.fetch_sprite(cmd.sprite).unwrap().bg;

        self.cmd.draw_indexed(&DrawIndexed {
            vertices: self.manager.vertices(),
            indices: self.manager.indices(),
            dynamic_buffers: [Some(b1), Some(b2), None, None],
            bind_groups: [Some(sprite_bg), None, None, None],
            index_count: 6,
            ..Default::default()
        });
    }

    pub fn draw_spritesheet(&mut self, cmd: &SpriteSheetDrawCommand) {
        let mut vert_alloc = self.manager.allocator().bump().unwrap();
        let mut b1 = self.manager.allocator().bump().unwrap();
        let mut b2 = self.manager.allocator().bump().unwrap();
        if let Some(sheet) = self.manager.fetch_sprite_sheet(cmd.sheet) {
            if let Some(bounds) = sheet.sprites.get(&cmd.sprite_id) {
                let transform = &mut b1.slice::<glam::Mat4>()[0];
                let camera = &mut b2.slice::<glam::Vec2>()[0];
                let vertices = vert_alloc.slice::<Vertex>().split_at_mut(4).0;

                vertices.copy_from_slice(&[
                    // Top-left corner of the screen
                    Vertex {
                        position: [-1.0, 1.0],
                        tex_coords: [bounds.x, bounds.h],
                    },
                    // Bottom-left corner of the screen
                    Vertex {
                        position: [-1.0, -1.0],
                        tex_coords: [bounds.x, bounds.y],
                    },
                    // Bottom-right corner of the screen
                    Vertex {
                        position: [1.0, -1.0],
                        tex_coords: [bounds.w, bounds.y],
                    },
                    // Top-right corner of the screen
                    Vertex {
                        position: [1.0, 1.0],
                        tex_coords: [bounds.w, bounds.h],
                    },
                ]);

                let angle_radian = cmd.rotation.to_radians();
                let half_size = glam::Vec3::new(cmd.size.x() / 2.0, cmd.size.y() / 2.0, 0.0);

                // Step 1: Translate to move the quad's center to the origin
                let translate_to_origin = glam::Mat4::from_translation(-half_size);

                // Step 2: Apply rotation around the origin
                let rotate = glam::Mat4::from_rotation_z(angle_radian);

                // Step 3: Translate the quad back to its original position
                let translate_back = glam::Mat4::from_translation(
                    glam::Vec3::new(cmd.position.x(), cmd.position.y(), 0.0) + half_size,
                );
                let scale =
                    glam::Mat4::from_scale(glam::Vec3::new(cmd.size.x(), cmd.size.y(), 1.0));

                let t = translate_back * rotate * translate_to_origin * scale;

                *transform = t;
                *camera = glam::Vec2::new(0.0, 0.0);
                let sprite_bg = self.manager.fetch_sprite_sheet(cmd.sheet).unwrap().bg;

                self.cmd.draw_dynamic_indexed(&DrawIndexedDynamic {
                    vertices: vert_alloc,
                    indices: self.manager.indices().to_unmapped_dynamic(0),
                    dynamic_buffers: [Some(b1), Some(b2), None, None],
                    bind_groups: [Some(sprite_bg), None, None, None],
                    index_count: 6,
                    ..Default::default()
                });
            }
        }
    }
}
