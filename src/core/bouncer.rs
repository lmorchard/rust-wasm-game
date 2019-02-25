extern crate pyro;
extern crate nalgebra as na;

use pyro::*;
use na::Point2;
use DeltaTime;

use motion;

#[derive(Debug, Copy, Clone)]
pub struct Bouncer(pub Point2<f32>, pub Point2<f32>);

pub fn update_bouncers(world: &mut World, dt: DeltaTime) {
    world
        .matcher::<All<(Read<motion::Position>, Write<motion::Velocity>, Read<Bouncer>)>>()
        .for_each(|(pos, vel, bouncer)| {
            if pos.0.x <= bouncer.0.x || pos.0.x >= bouncer.1.x {
                vel.0.x = 0.0 - vel.0.x;
            }
            if pos.0.y <= bouncer.0.y || pos.0.y >= bouncer.1.y {
                vel.0.y = 0.0 - vel.0.y;
            }
        });
}

