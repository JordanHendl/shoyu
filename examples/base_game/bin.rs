use glam::*;
use renderer2d::SpriteDrawCommand;
use renderer2d::SpriteInfo;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use shoyu::database::*;
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

    let mut pos = vec2(0.0, 0.0);
    let mut rot = 0.0;
    let mut event_pump = ctx.get_sdl_event();
    let mut pup = false;
    let mut pdown = false;
    'running: loop {
        // Listen to events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    pos = vec2(pos.x() + 0.01, pos.y());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    pos = vec2(pos.x() - 0.01, pos.y());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    pup = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    pup = false;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    pdown = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    pdown = false;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => {
                    rot += 1.1;
                }
                _ => {}
            }
        }

        if pdown {
            pos = vec2(pos.x(), pos.y() + 0.01);
        }

        if pup {
            pos = vec2(pos.x(), pos.y() - 0.01);
        }

        renderer.begin_drawing();
        renderer.draw_sprite(&SpriteDrawCommand {
            sprite,
            position: pos,
            size: vec2(0.5, 0.5),
            rotation: rot,
            flip: false,
        });

        renderer.finish_drawing();
    }
}
