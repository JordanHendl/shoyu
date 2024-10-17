pub mod types;
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
            let (sem, fence) = (*self.ctx).submit(&mut self.cmd, Some(&[self.display_sem])).unwrap();

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
