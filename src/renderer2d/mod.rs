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
    pub flip: bool,
}

pub struct SpriteSheetDrawCommand {
    pub sheet: Handle<SpriteSheet>,
    pub sprite_bounds: Rect2D,
    pub position: glam::Vec2,
    pub rotation: f32,
    pub flip: bool,
}

pub struct TextDrawCommand<'a> {
    pub font: Handle<Font>,
    pub position: glam::Vec2,
    pub text: &'a str,
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
        struct TextVertex {
            pos: glam::Vec2,
            tex: glam::Vec2,
        }

        let font_bg = self.manager.fetch_font(cmd.font).unwrap().bg;
        let view = self.manager.fetch_font(cmd.font).unwrap().atlas_view;
        let font = self.manager.fetch_font(cmd.font).unwrap().font;
        let dim = self.manager.fetch_font(cmd.font).unwrap().dim;
        self.cmd
            .begin_drawing(&DrawBegin {
                viewport: self.manager.canvas().viewport(),
                pipeline: self.manager.gfx().text_pipeline,
            })
            .unwrap();

        for ch in cmd.text.chars() {
            unsafe {
                if let Some(g) = (*font).glyphs.get(&ch) {
                    let mut vert_alloc = self.manager.allocator().bump().unwrap();
                    let mut index_alloc = self.manager.allocator().bump().unwrap();
                    let mut info = self.manager.allocator().bump().unwrap();
                    let mut vertices = vert_alloc.slice::<TextVertex>();
                    let mut indices = index_alloc.slice::<u32>();
                    let mut color = info.slice::<glam::Vec4>();

                    color[0] = vec4(1.0, 1.0, 1.0, 1.0);

                    let fbounds = FRect2D {
                        x: g.bounds.x as f32 / dim[0] as f32,
                        y: g.bounds.y as f32 / dim[1] as f32,
                        w: g.bounds.w as f32 / dim[0] as f32,
                        h: g.bounds.h as f32 / dim[1] as f32,
                    };

                    let glyph_width = fbounds.w - fbounds.x;
                    let glyph_height = fbounds.h - fbounds.y;
                    //todo fill
                    vertices[0] = TextVertex {
                        pos: cmd.position,
                        tex: vec2(fbounds.x, fbounds.y),
                    };
                    vertices[1] = TextVertex {
                        pos: vec2(cmd.position.x(), cmd.position.y() + glyph_height),
                        tex: vec2(fbounds.x, fbounds.h),
                    };
                    vertices[2] = TextVertex {
                        pos: vec2(
                            cmd.position.x() + glyph_width,
                            cmd.position.y() + glyph_height,
                        ),
                        tex: vec2(fbounds.w, fbounds.h),
                    };
                    vertices[3] = TextVertex {
                        pos: vec2(cmd.position.x() + glyph_width, cmd.position.y()),
                        tex: vec2(fbounds.w, fbounds.y),
                    };

                    indices[0] = 0;
                    indices[1] = 1;
                    indices[2] = 2;
                    indices[3] = 2;
                    indices[4] = 3;
                    indices[5] = 0;

                    self.cmd.draw_indexed(&DrawIndexed {
                        vertices: vert_alloc.handle(),
                        indices: index_alloc.handle(),
                        dynamic_buffers: [Some(info), None, None, None],
                        bind_groups: [Some(font_bg), None, None, None],
                        vert_offset: vert_alloc.offset(),
                        index_offset: index_alloc.offset(),
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

    pub fn draw_spritesheet(&mut self, cmd: &SpriteSheetDrawCommand) {}
}
