use glam::*;
use io::*;
use shoyu::renderer2d::SpriteDrawCommand;
use shoyu::renderer2d::SpriteInfo;
use shoyu::renderer2d::SpriteSheetDrawCommand;
use shoyu::renderer2d::SpriteSheetInfo;
use shoyu::sdl2::keyboard::Keycode;
use shoyu::sdl2::mouse::MouseButton;
use shoyu::database::*;
use shoyu::renderer2d::FontInfo;
use shoyu::renderer2d::ParticleBehaviour;
use shoyu::renderer2d::ParticleEmitInfo;
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
    let database = Database::new(&args[1]).unwrap();
    let canvas = Canvas::from_json(&mut ctx, &format!("{}/canvas.json", &args[1]));
    let mut renderer = renderer2d::Renderer2D::new(&mut ctx, database, canvas);

    let sprite = renderer.resources().make_sprite(&SpriteInfo {
        name: "test",
        db_key: "name",
    });

    let font = renderer.resources().make_font(&FontInfo {
        name: "font",
        db_key: "font",
    });

    let sheet = renderer.resources().make_sprite_sheet(&SpriteSheetInfo {
        name: "sheet",
        db_key: "character",
    });
    let mut pos = vec2(0.0, 0.0);
    let mut rot = 0.0;
    let mut sprite_id = 0;
    let mut io_controller = IOController::new(ctx.get_sdl_ctx());
    io_controller.map_action_keys("up", vec![Keycode::W]);
    io_controller.map_action_keys("down", vec![Keycode::S]);
    io_controller.map_action_keys("left", vec![Keycode::A]);
    io_controller.map_action_keys("right", vec![Keycode::D]);
    io_controller.map_action_keys("rotate", vec![Keycode::Q]);
    io_controller.map_action_keys("increment_sprite", vec![Keycode::UP]);
    io_controller.map_action_keys("decrement_sprite", vec![Keycode::DOWN]);
    io_controller.map_action_buttons("emit_particles", vec![MouseButton::Left]);
    'running: loop {
        io_controller.update();

        if io_controller.event_cache().is_quit() {
            break 'running;
        }
        if io_controller
            .is_action_pressed("increment_sprite")
        {
            sprite_id += 1;
        }
        if io_controller
            .is_action_pressed("decrement_sprite")
        {
            if sprite_id != 0 {
                sprite_id -= 1;
            }
        }
        if io_controller
            .is_action_pressed("emit_particles")
        {
            let pos = io_controller.get_mouse_position_interp(1024.0, 1024.0);

            renderer.particle_system().emit_random(&ParticleEmitInfo {
                particle_id: 0,
                lifetime_ms: 2000.0,
                amount: 20,
                position: vec2(pos.position.0 - 0.5, pos.position.1 - 0.5),
                initial_velocity: vec2(0.0, 0.0),
                behaviour: ParticleBehaviour::GRAVITY,
            });
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
            position: vec2(0.0, 0.0),
            size: vec2(1.0, 1.0),
            rotation: 0.0,
        });

        renderer.draw_spritesheet(&SpriteSheetDrawCommand {
            position: pos,
            size: vec2(0.2, 0.2),
            rotation: rot,
            sheet,
            sprite_id,
        });

        renderer.draw_text(&TextDrawCommand {
            font,
            position: vec2(0.0, 0.1),
            scale: 1.0,
            text: "The quick brown fox jumps over the lazy dog.",
            ..Default::default()
        });

        renderer.finish_drawing();
    }
}
