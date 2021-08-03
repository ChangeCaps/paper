use paper::*;

struct AppState;

impl State for AppState {}

fn main() {
    App::new().run(AppState);
}
