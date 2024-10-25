use std::iter::zip;

use macroquad::prelude::*;
use macroquad::ui::hash;

use macroquad::ui::{
    root_ui,
    widgets::{self, Slider},
};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

const DT: f32 = 0.001;

#[derive(Clone)]
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
    fn update(&mut self, t: f32, damping: f32, fixed_end: bool) {
        for i in 0..self.points.len() {
            if i == 0 {
                self.points[i].pos = t.sin();
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
    let mut damping = 0.1;
    let mut fixed_end = false;

    loop {
        clear_background(BLACK);

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
        let delta = get_frame_time();
        let mut accum = 0.;
        while delta > accum {
            points.update(t, damping, fixed_end);
            t += DT * f;
            accum += DT;
        }

        next_frame().await;
    }
}

struct TwoDimMedium {
    coef: f32,
    points: Vec<Vec<Point>>,
}

impl TwoDimMedium {
    fn new(mut w: usize) -> Self {
        if w % 2 == 0 {
            w += 1;
        }
        Self {
            coef: 200.0,
            points: vec![
                vec![
                    Point {
                        pos: 0.,
                        velocity: 0.,
                        acceleration: 0.
                    };
                    w
                ];
                w
            ],
        }
    }

    fn interpolate(&self, x: f32, y: f32) -> f32 {
        let x_grid = x.floor() as usize;
        let y_grid = y.floor() as usize;
        let grid_pos_x = x % 1.0 - 0.5;
        let grid_pos_y = y % 1.0 - 0.5;
        let mut amp = 0.;
        if grid_pos_x < 0. {
            if x_grid == 0 {
                amp += self.points[x_grid][y_grid].pos.abs();
            } else {
                let dist = grid_pos_x.abs();
                amp += self.points[x_grid][y_grid].pos.abs() * dist
                    + self.points[x_grid - 1][y_grid].pos.abs() * (1.0 - dist);
            }
        } else {
            if x_grid == self.points.len() - 1 {
                amp += self.points[x_grid][y_grid].pos.abs();
            } else {
                let dist = grid_pos_x;
                amp += self.points[x_grid][y_grid].pos.abs() * dist
                    + self.points[x_grid + 1][y_grid].pos.abs() * (1.0 - dist);
            }
        }
        if grid_pos_y < 0. {
            if y_grid == 0 {
                amp += self.points[x_grid][y_grid].pos.abs();
            } else {
                let dist = grid_pos_x.abs();
                amp += self.points[x_grid][y_grid].pos.abs() * dist
                    + self.points[x_grid][y_grid - 1].pos.abs() * (1.0 - dist);
            }
        } else {
            if y_grid == self.points.len() - 1 {
                amp += self.points[x_grid][y_grid].pos.abs();
            } else {
                let dist = grid_pos_x;
                amp += self.points[x_grid][y_grid].pos.abs() * dist
                    + self.points[x_grid][y_grid + 1].pos.abs() * (1.0 - dist);
            }
        }
        amp / 2.
    }

    fn update(&mut self, t: f32, damping: f32) {
        let w = self.points.len();
        let ref_points = self.points.clone();
        self.points.par_iter_mut().enumerate().for_each(|(i, row)| {
            row.par_iter_mut().enumerate().for_each(|(j, point)| {
                if i * 2 + 1 == w && j * 2 + 1 == w {
                    point.pos = t.sin();
                } else {
                    if i != 0 {
                        point.acceleration +=
                            (ref_points[i - 1][j].pos - ref_points[i][j].pos) * self.coef;
                    }
                    if i != w - 1 {
                        point.acceleration +=
                            (ref_points[i + 1][j].pos - ref_points[i][j].pos) * self.coef
                    }
                    if j != 0 {
                        point.acceleration +=
                            (ref_points[i][j - 1].pos - ref_points[i][j].pos) * self.coef
                    }
                    if j != w - 1 {
                        point.acceleration +=
                            (ref_points[i][j + 1].pos - ref_points[i][j].pos) * self.coef
                    }
                    point.acceleration -= ref_points[i][j].velocity * damping;
                }
            });
        });
        for row in self.points.iter_mut() {
            for point in row.iter_mut() {
                point.update();
            }
        }
    }
}

#[cfg(feature = "plate")]
#[macroquad::main("plate")]
async fn main() {
    use macroquad::rand;

    let w = 50;
    let mut medium = TwoDimMedium::new(w);
    let mut t = 0.;
    let mut f = 1.;
    let mut damping = 0.1;
    let mut sandgrains = Vec::new();

    let mut color_feild = false;
    let mut sand = true;
    let mut sweep = true;

    for _ in 0..20000 {
        sandgrains.push([
            rand::gen_range(0.5, w as f32 - 0.5),
            rand::gen_range(0.5, w as f32 - 0.5),
        ]);
    }
    // let mut image = Image::gen_image_color(w as u16 + 1, w as u16 + 1, BLACK);
    // let texture = Texture2D::from_image(&image);
    loop {
        clear_background(BLACK);

        if color_feild {
            for (i, row) in medium.points.iter().enumerate() {
                for (j, p) in row.iter().enumerate() {
                    // draw circles

                    let min = screen_height().min(screen_width());
                    draw_circle(
                        min / 2. * ((i as f32 / w as f32) - 0.5) + min / 2.,
                        min / 2. * ((j as f32 / w as f32) - 0.5) + min / 2.,
                        min as f32 * 0.25 / (2. * w as f32)
                            + p.pos.abs().clamp(0., 1.) * min as f32 * 0.5 / (2. * w as f32),
                        Color {
                            r: p.pos * 0.4 + 0.5,
                            g: -p.pos * 0.4 + 0.5,
                            b: 1.0,
                            a: 0.6,
                        },
                    )
                }
            }
        }

        if sand {
            // draw sand

            sandgrains.par_iter_mut().for_each(|p| {
                let amp = medium.interpolate(p[0], p[1]);
                let random_x = rand::gen_range(-1.0, 1.0_f32);
                let random_x = random_x * amp;
                let random_y = rand::gen_range(-1.0, 1.0_f32);
                let random_y = random_y * amp;
                p[0] += random_x;
                p[1] += random_y;
                p[0] = p[0].clamp(0.5, medium.points.len() as f32 - 0.5);
                p[1] = p[1].clamp(0.5, medium.points.len() as f32 - 0.5);
            });

            let min = screen_height().min(screen_width());
            sandgrains.iter().for_each(|p| {
                draw_circle(
                    min / 2. * ((p[0] as f32 / w as f32) - 0.5) + min / 2.,
                    min / 2. * ((p[1] as f32 / w as f32) - 0.5) + min / 2.,
                    0.35,
                    YELLOW,
                )
            });
        }

        widgets::Window::new(hash!("Settings"), Vec2::new(0., 0.), Vec2::new(400., 150.))
            .label("Settings")
            .ui(&mut *root_ui(), |ui| {
                ui.slider(hash!(), "freq:", 0.0..500.0, &mut f);
                ui.slider(hash!(), "tension:", 50.0..200_000.0, &mut medium.coef);
                ui.slider(hash!(), "damping:", 0.0..5.0, &mut damping);
                ui.checkbox(hash!(), "show amplitude feild", &mut color_feild);
                ui.checkbox(hash!(), "show sand", &mut sand);
                ui.checkbox(hash!(), "sweep freq", &mut sweep);

                if ui.button(None, "reset") {
                    for row in medium.points.iter_mut() {
                        for p in row.iter_mut() {
                            p.pos = 0.;
                            t = 0.;
                            p.velocity = 0.;
                            p.acceleration = 0.;
                        }
                    }
                }
            });

        let delta = get_frame_time();
        if sweep {
            f += delta * 0.5;
        }
        let mut accum = 0.;
        while delta > accum {
            medium.update(t, damping);
            t += DT * f;
            accum += DT;
        }

        next_frame().await;
    }
}
