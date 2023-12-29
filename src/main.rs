use std::cmp;
use std::f64::consts::PI;
use std::time::Instant;

use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{drivers, WindowCanvas};
use sdl2::rwops::RWops;
use sdl2::video::Window;
use sdl2::{Sdl, VideoSubsystem};

const WIDTH: u32 = 600;
const HEIGHT: u32 = 440;
const BORDER: i32 = 10;

const L: f64 = 200.0;
// cm
const G: f64 = 981.0;
// cm/s2
const CENTER: (i16, i16) = (300, 220);
const THETA_0: f64 = -1.0 * PI * 0.65;

fn main() {
    let font_bytes = include_bytes!("../Roboto.ttf");

    let sdl_context: Sdl = sdl2::init().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    let video_subsystem: VideoSubsystem = sdl_context.video().unwrap();
    let timer = sdl_context.timer().unwrap();
    let font = ttf_context
        .load_font_from_rwops(RWops::from_bytes(font_bytes).unwrap(), 24)
        .unwrap();

    let window: Window = video_subsystem
        .window("Pendulum", WIDTH, HEIGHT)
        .opengl()
        .position_centered()
        .build()
        .expect("Window couldn't be created.");

    let mut canvas: WindowCanvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .index(
            drivers()
                .enumerate()
                .filter(|it| it.1.name == "opengl")
                .map(|it| it.0 as u32)
                .next()
                .unwrap(),
        )
        .build()
        .unwrap();

    let texture_creator = canvas.texture_creator();

    // pendulum angle
    let mut theta: f64 = THETA_0;
    // instant to calculate dt -> t0
    let mut now = Instant::now();
    // angular velocity -> w0
    let mut w: f64 = 0.0;

    let mut events = sdl_context.event_pump().unwrap();
    let mut elapsed: u64 = 1;
    'main: loop {
        let start = timer.performance_counter();
        canvas.set_draw_color(Color::RGB(u8::MAX, u8::MAX, u8::MAX));
        // fills the canvas with the color we set in `set_draw_color`.
        canvas.clear();

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'main;
                }
                _ => {}
            }
        }

        // calculating the new angular velocity using the approximation to the differential equation
        // Δω = -g/l * sin(θ) * Δt
        // elapsed time from last iteration
        let delta_t = Instant::now().duration_since(now);
        w += -1.0 * G / L * theta.sin() * delta_t.as_secs_f64();
        // calculating the new angle
        theta += w * delta_t.as_secs_f64();

        // calculating position of the pendulum
        let x: i16 = CENTER.0 + (theta.sin() * L).round() as i16;
        let y: i16 = CENTER.1 + (theta.cos() * L).round() as i16;

        // drawing pendulum
        canvas
            .aa_line(CENTER.0, CENTER.1, x, y, Color::BLACK)
            .expect("Unable to draw line");
        canvas
            .aa_circle(x, y, 5, Color::BLACK)
            .expect("Unable to draw circle");

        // calculating speed vector of the pendulum
        let vx: i16 = x + (theta.cos() * L * w / 10.0).round() as i16;
        let vy: i16 = y - (theta.sin() * L * w / 10.0).round() as i16;
        canvas
            .aa_line(x, y, vx, vy, Color::RED)
            .expect("Unable to draw line");

        let angle = 90 - (theta * 180.0 / PI) as i16;
        canvas
            .filled_pie(
                CENTER.0,
                CENTER.1,
                50,
                cmp::min(angle, 90),
                cmp::max(angle, 90),
                Color::RGBA(0, 0, 255, 100),
            )
            .unwrap();

        // drawing axis
        canvas
            .aa_line(
                CENTER.0,
                CENTER.1,
                CENTER.0,
                CENTER.1 + 100,
                Color::RGBA(0, 0, 255, 100),
            )
            .expect("Unable to draw line");
        canvas
            .aa_line(
                CENTER.0,
                CENTER.1,
                CENTER.0 + 100,
                CENTER.1,
                Color::RGBA(0, 0, 255, 100),
            )
            .expect("Unable to draw line");

        let radians_per_sec = texture_creator
            .create_texture_from_surface(
                &font
                    .render(format!("ω: {:.3} rad/s", w).as_str())
                    .blended(Color::BLACK)
                    .unwrap(),
            )
            .unwrap();

        let radians = texture_creator
            .create_texture_from_surface(
                &font
                    .render(format!("θ: {:.3} rad", theta).as_str())
                    .blended(Color::BLACK)
                    .unwrap(),
            )
            .unwrap();

        let speed = texture_creator
            .create_texture_from_surface(
                &font
                    .render(format!("v: {:.3} m/s", w * L / 100.0).as_str())
                    .blended(Color::BLACK)
                    .unwrap(),
            )
            .unwrap();
        let fps = texture_creator
            .create_texture_from_surface(
                &font
                    .render(
                        format!(
                            "FPS: {:.2}",
                            timer.performance_frequency() as f64 / elapsed as f64
                        )
                        .as_str(),
                    )
                    .blended(Color::BLACK)
                    .unwrap(),
            )
            .unwrap();

        let fps_query = fps.query();

        canvas
            .copy(
                &radians_per_sec,
                None,
                Rect::new(
                    BORDER,
                    BORDER,
                    radians_per_sec.query().width,
                    radians_per_sec.query().height,
                ),
            )
            .unwrap();
        canvas
            .copy(
                &radians,
                None,
                Rect::new(
                    BORDER,
                    radians_per_sec.query().height as i32 + BORDER,
                    radians.query().width,
                    radians.query().height,
                ),
            )
            .unwrap();
        canvas
            .copy(
                &speed,
                None,
                Rect::new(
                    BORDER,
                    radians_per_sec.query().height as i32 + radians.query().height as i32 + BORDER,
                    speed.query().width,
                    speed.query().height,
                ),
            )
            .unwrap();
        canvas
            .copy(
                &fps,
                None,
                Rect::new(
                    WIDTH as i32 - BORDER - fps_query.width as i32,
                    HEIGHT as i32 - BORDER - fps_query.height as i32,
                    fps_query.width,
                    fps_query.height,
                ),
            )
            .unwrap();

        now = Instant::now();
        // drawing frame
        canvas.present();

        elapsed = timer.performance_counter() - start;
    }
}
