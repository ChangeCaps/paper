use std::f32::consts::{PI, SQRT_2, TAU};

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
        frame.config.resolution = 0.05;

        let shape = Parametric::new(|x| Vec2::new(x.sin(), -x.cos()), 0.0..TAU)
            .complete()
            .map(|v| {
                let f = PI / 4.0;

                if v.y < -f.sin() {
                    v.y = v.x.abs() - SQRT_2;
                }
            })
            .hole(Circle::new(0.6))
            .split(
                |shape| shape.fill([0.1, 0.2, 0.6, 1.0]),
                |shape| shape.outline(0.1).fill([0.0, 0.0, 0.0, 1.0]),
            )
            .combine();

        frame.draw_shape(&shape, self.transform.clone(), &self.camera);
    }
}

fn main() {
    App::new().window_size(500, 500).run(AppState::new());
}
