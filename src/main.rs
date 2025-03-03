use ggez::conf::{WindowMode, WindowSetup};
use ggez::{event, graphics, Context, ContextBuilder, GameResult};
use std::ops::{Add, Div, Mul, Sub};

use ggez::mint;

const G: f32 = 1.0;

#[derive(Debug, Clone, Copy)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }
    fn magnitude(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    fn normalize(self) -> Self {
        let mag = self.magnitude();
        if mag == 0.0 {
            self
        } else {
            self / mag
        }
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, scalar: f32) -> Vec2 {
        Vec2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}
impl Div<f32> for Vec2 {
    type Output = Vec2;
    fn div(self, scalar: f32) -> Vec2 {
        Vec2 {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

#[derive(Debug, Clone)]
struct Body {
    pos: Vec2,
    vel: Vec2,
    mass: f32,
    color: graphics::Color,
    trail: Vec<Vec2>,
}

struct Simulation {
    bodies: Vec<Body>,
    dt: f32,
    accumulator: f32,
}

impl Simulation {
    fn new() -> Self {
        Simulation {
            bodies: vec![
                Body {
                    pos: Vec2::new(-100.0, 0.0),
                    vel: Vec2::new(0.0, 0.5),
                    mass: 70.0,
                    color: graphics::Color::from_rgb(255, 0, 0),
                    trail: Vec::new(),
                },
                Body {
                    pos: Vec2::new(0.0, 0.0),
                    vel: Vec2::new(0.0, 0.0),
                    mass: 100.0,
                    color: graphics::Color::from_rgb(0, 255, 0),
                    trail: Vec::new(),
                },
                Body {
                    pos: Vec2::new(100.0, 0.0),
                    vel: Vec2::new(0.0, -0.50),
                    mass: 30.0,
                    color: graphics::Color::from_rgb(0, 0, 255),
                    trail: Vec::new(),
                },
            ],
            dt: 0.01,
            accumulator: 0.0,
        }
    }

    fn compute_accelerations(&self) -> Vec<Vec2> {
        let n = self.bodies.len();
        let mut acc = vec![Vec2::new(0.0, 0.0); n];
        for (i, acc_i) in acc.iter_mut().enumerate() {
            for j in 0..n {
                if i == j {
                    continue;
                }
                let diff = self.bodies[j].pos - self.bodies[i].pos;
                let distance = diff.magnitude().max(0.1); // Softening to prevent division by zero
                *acc_i =
                    *acc_i + diff.normalize() * (G * self.bodies[j].mass / (distance * distance));
            }
        }
        acc
    }

    fn step(&mut self) {
        let acc_old = self.compute_accelerations();
        let dt = self.dt;

        for (i, body) in self.bodies.iter_mut().enumerate() {
            body.pos = body.pos + body.vel * dt + acc_old[i] * (0.5 * dt * dt);
        }

        let acc_new = self.compute_accelerations();
        for (i, body) in self.bodies.iter_mut().enumerate() {
            body.vel = body.vel + (acc_old[i] + acc_new[i]) * (0.5 * dt);
            let pos = body.pos;
            body.trail.push(pos);
        }
    }
}

impl event::EventHandler for Simulation {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let delta = ctx.time.delta().as_secs_f32();
        let speed_factor = 500.0;
        self.accumulator += delta * speed_factor;
        while self.accumulator >= self.dt {
            self.step();
            self.accumulator -= self.dt;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from_rgb(255, 255, 255));
        let (screen_w, screen_h) = ctx.gfx.drawable_size();

        for body in &self.bodies {
            if body.trail.len() > 1 {
                let trail_points: Vec<mint::Point2<f32>> = body
                    .trail
                    .iter()
                    .map(|p| mint::Point2 {
                        x: p.x + screen_w / 2.0,
                        y: p.y + screen_h / 2.0,
                    })
                    .collect();
                let trail_line = graphics::Mesh::new_line(ctx, &trail_points, 1.0, body.color)?;
                canvas.draw(&trail_line, graphics::DrawParam::default());
            }
        }

        for body in &self.bodies {
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                mint::Point2 {
                    x: body.pos.x + screen_w / 2.0,
                    y: body.pos.y + screen_h / 2.0,
                },
                5.0,
                0.1,
                body.color,
            )?;
            canvas.draw(&circle, graphics::DrawParam::default());
        }

        canvas.finish(ctx)
    }
}

pub fn main() -> GameResult {
    let window_mode = WindowMode {
        width: 1280.0,
        height: 960.0,
        ..Default::default()
    };

    let window_setup = WindowSetup {
        title: "Three-Body Simulation".to_string(),
        ..Default::default()
    };

    let (ctx, event_loop) = ContextBuilder::new("three_body_simulation", "me")
        .window_mode(window_mode)
        .window_setup(window_setup)
        .build()?;

    let simulation = Simulation::new();
    event::run(ctx, event_loop, simulation)
}
