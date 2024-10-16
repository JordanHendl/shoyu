pub mod types;
pub use types::*;
pub mod resource_manager;
pub use resource_manager::*;

pub use dashi::utils::*;
pub use dashi::*;

use crate::database::Database;
use crate::utils::Canvas;
mod pipeline;
use pipeline::*;

pub struct Renderer2D {
    display: Display,
    manager: ResourceManager,
    canvas: Canvas,
    ctx: *mut Context,
    gfx: pipeline::GraphicsPipelineInfo,
    allocator: DynamicAllocator,
}

pub struct SpriteDrawCommand {
    pub sprite: Handle<Sprite>,
    pub position: glam::Vec2,
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

        let allocator = ctx.make_dynamic_allocator(&Default::default()).unwrap();

        let manager = ResourceManager::new(ctx, database);

        let gfx = make_graphics_pipeline(ctx, &canvas);

        Self {
            display,
            manager,
            canvas,
            ctx,
            gfx,
            allocator,
        }
    }
    
    pub fn resources(& mut self) -> & mut ResourceManager {
        return & mut self.manager;
    }

    pub fn draw_sprite(&mut self, cmd: &SpriteDrawCommand) {}

    pub fn draw_spritesheet(&mut self, cmd: &SpriteSheetDrawCommand) {}
}
