pub mod bouncer;
pub mod motion;
pub mod render_frame;
pub mod sprite;

#[derive(Debug, Copy, Clone)]
pub struct DeltaTime(pub f32);

/*
use std::time::Duration;
impl DeltaTime {
    pub fn as_duration(&self) -> Duration {
        Duration::from_millis((self.0 * 1000.0) as u64)
    }
}
*/
