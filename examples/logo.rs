use std::f32::consts::{FRAC_PI_4, PI, SQRT_2, TAU};

use paper::*;

struct AppState {
    transform: Transform,
    camera: OrthographicCamera,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            transform: Transform::from_scale(Vec3::splat(0.65)),
            camera: OrthographicCamera::default(),
        }
    }
}

impl State for AppState {
    fn draw<'a>(&'a mut self, frame: &mut Frame<'a>) {
        frame.config.resolution = 0.1;

        let shape = Line::new(
            (0.0, -FRAC_PI_4.sin() * 2.0),
            (-FRAC_PI_4.sin(), -FRAC_PI_4.sin()),
        )
        .turn(1.0, -PI * 1.5)
        .forward(0.1)
        .offset(0.35)
        .thicken(0.6, false)
        .split(
            |s| s.fill([0.1, 0.2, 0.6, 1.0]),
            |s| s.outline(0.1).fill([0.0, 0.0, 0.0, 1.0]),
        )
        .combine(); 

        frame.draw_shape(&shape, self.transform.clone(), &self.camera);
    }
}

fn main() {
    App::new().window_size(500, 500).run(AppState::new());
}
