use macroquad::prelude::*;
use macroquad::ui::hash;

use macroquad::ui::{
    root_ui,
    widgets::{self, Slider},
};

const DT: f32 = 0.005;

struct Point {
    pos: f32,
    velocity: f32,
    acceleration: f32,
}
impl Point {
    fn update(&mut self) {
        self.velocity += self.acceleration * DT;
        self.pos += self.velocity * DT;
        self.acceleration = 0.;
    }
}

struct LinearMedium {
    points: Vec<Point>,
    coef: f32,
}

impl LinearMedium {
    fn update(&mut self, t: f32, f: f32, damping: f32, fixed_end: bool) {
        for i in 0..self.points.len() {
            if i == 0 {
                self.points[i].pos = (t * f).sin();
            } else if i != self.points.len() - 1 {
                self.points[i].acceleration +=
                    (self.points[i + 1].pos - self.points[i].pos) * self.coef;
            }
            if i != 0 {
                self.points[i].acceleration +=
                    (self.points[i - 1].pos - self.points[i].pos) * self.coef;
                self.points[i].acceleration -= self.points[i].velocity * damping;
            }
        }
        for p in self.points.iter_mut() {
            p.update()
        }
        if fixed_end {
            if let Some(p) = self.points.last_mut() {
                p.pos = 0.;
            }
        }
    }
}

#[cfg(feature = "oscillations")]
#[macroquad::main("oscillations")]
async fn main() {
    use std::time::Instant;

    let point_count = 100;
    let mut points = vec![];
    for _ in 0..point_count {
        points.push(Point {
            pos: 0.,
            velocity: 0.,
            acceleration: 0.,
        });
    }

    let mut points = LinearMedium {
        points: points,
        coef: 35.,
    };

    let mut t = 0.;
    let mut f = 1.0;
    let mut old_f = 1.0;
    let mut iter_per_frame = 20;
    let mut damping = 0.1;
    let mut fixed_end = false;

    loop {
        clear_background(BLACK);

        //check frequensy update
        t += DT * iter_per_frame as f32;

        for (i, point) in points.points.iter().enumerate() {
            draw_circle(
                screen_width() * (i + 1) as f32 / (point_count + 2) as f32,
                screen_height() / 2. + point.pos * screen_height() / 50.,
                2.,
                WHITE,
            );
        }
        widgets::Window::new(hash!("Settings"), Vec2::new(0., 0.), Vec2::new(400., 150.))
            .label("Settings")
            .ui(&mut *root_ui(), |ui| {
                ui.slider(hash!(), "freq:", 0.0..5.0, &mut f);
                ui.slider(hash!(), "tension", 10.0..5000.0, &mut points.coef);
                ui.slider(hash!(), "damping: ", 0.0..1.0, &mut damping);
                ui.checkbox(hash!(), "fixed end", &mut fixed_end);
                if ui.button(None, "reset") {
                    for p in points.points.iter_mut() {
                        p.pos = 0.;
                        t = 0.;
                        p.velocity = 0.;
                        p.acceleration = 0.;
                    }
                }
            });
        for _ in 0..(iter_per_frame as usize) {
            points.update(t, f, damping, fixed_end);
        }

        if f != old_f {
            t = t * old_f / f;
            old_f = f;
        }
        next_frame().await;
    }
}

#[cfg(feature = "plate")]
#[macroquad::main("plate")]
async fn main() {}
