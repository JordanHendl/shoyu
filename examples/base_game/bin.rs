use glam::*;
use io::*;
use renderer2d::SpriteDrawCommand;
use renderer2d::SpriteInfo;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use shoyu::database::*;
use shoyu::renderer2d::FontInfo;
use shoyu::renderer2d::TextDrawCommand;
use shoyu::utils::*;
use shoyu::*;
use std::env;
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("Usage: {} <path_to_database>", args[0]);
        return;
    }

    let mut ctx = dashi::Context::new(&Default::default()).unwrap();
    let mut database = Database::new(&args[1]).unwrap();
    let mut canvas = Canvas::from_json(&mut ctx, &format!("{}/canvas.json", &args[1]));
    let mut renderer = renderer2d::Renderer2D::new(&mut ctx, database, canvas);

    let sprite = renderer.resources().make_sprite(&SpriteInfo {
        name: "test",
        db_key: "name",
    });
    
    let font = renderer.resources().make_font(&FontInfo {
        name: "font",
        db_key: "font",
    });

    let mut pos = vec2(0.0, 0.0);
    let mut rot = 0.0;

    let mut io_controller = IOController::new(ctx.get_sdl_ctx());
    io_controller.map_action("up", vec![Keycode::W]);
    io_controller.map_action("down", vec![Keycode::S]);
    io_controller.map_action("left", vec![Keycode::A]);
    io_controller.map_action("right", vec![Keycode::D]);
    io_controller.map_action("rotate", vec![Keycode::Q]);

    'running: loop {
        io_controller.update();

        if io_controller.event_cache().is_quit() {
            break 'running;
        }

        if io_controller.is_action_active("left") {
            pos = vec2(pos.x() - 0.01, pos.y());
        }
        if io_controller.is_action_active("right") {
            pos = vec2(pos.x() + 0.01, pos.y());
        }
        if io_controller.is_action_active("up") {
            pos = vec2(pos.x(), pos.y() - 0.01);
        }
        if io_controller.is_action_active("down") {
            pos = vec2(pos.x(), pos.y() + 0.01);
        }
        if io_controller.is_action_active("rotate") {
            rot += 1.0;
        }

        renderer.begin_drawing();
        renderer.draw_sprite(&SpriteDrawCommand {
            sprite,
            position: pos,
            size: vec2(0.5, 0.5),
            rotation: rot,
            flip: false,
        });
        
        renderer.draw_text(&TextDrawCommand {
            font,
            position: vec2(-0.5, 0.0),
            scale: 2.0,
            text: "The quick brown fox jumps over the lazy dog.",
        });

        renderer.draw_text(&TextDrawCommand {
            font,
            position: vec2(-0.5, 0.5),
            scale: 2.0,
            text: ":) uwu owo >3<",
        });

        renderer.finish_drawing();
    }
}
