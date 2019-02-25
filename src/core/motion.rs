use na::{Point2, Vector2};
use pyro::*;
use std::f32::consts::PI;
use DeltaTime;

const PI2: f32 = PI * 2.0;

#[derive(Debug, Copy, Clone)]
pub struct Position(pub Point2<f32>);

#[derive(Debug, Copy, Clone)]
pub struct Velocity(pub Vector2<f32>);

#[derive(Debug, Copy, Clone)]
pub struct Orientation(pub f32);

#[derive(Debug, Copy, Clone)]
pub struct Rotation(pub f32);

pub fn update_motion(world: &mut World, dt: DeltaTime) {
    world
        .matcher::<All<(Write<Position>, Read<Velocity>)>>()
        .for_each(|(pos, vel)| {
            pos.0 += vel.0 * dt.0;
        });
    world
        .matcher::<All<(Write<Orientation>, Read<Rotation>)>>()
        .for_each(|(ori, rot)| {
            ori.0 += rot.0 * dt.0;
            if ori.0 < 0.0 {
                ori.0 += PI2;
            }
            if ori.0 > PI2 {
                ori.0 -= PI2;
            }
        });
}
