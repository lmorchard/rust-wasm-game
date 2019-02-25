extern crate cfg_if;
extern crate itertools;
extern crate js_sys;
extern crate nalgebra as na;
extern crate pyro;
extern crate rand;
extern crate wasm_bindgen;
extern crate web_sys;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

use na::{Point2, Vector2};
use pyro::*;

mod utils;

mod core;
use core::*;

use std::f32::consts::PI;
const PI2: f32 = PI * 2.0;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[allow(unused_macros)]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub type BouncingEntity = (
    motion::Position,
    motion::Velocity,
    motion::Orientation,
    motion::Rotation,
    bouncer::Bouncer,
    sprite::Sprite,
);

pub fn create_bouncing_entity() -> BouncingEntity {
    (
        motion::Position(Point2::new(
            utils::gen_range(0.0, 400.0),
            utils::gen_range(0.0, 400.0),
        )),
        motion::Velocity(Vector2::new(
            utils::gen_range(100.0, 500.0),
            utils::gen_range(100.0, 500.0),
        )),
        motion::Orientation(0.0),
        motion::Rotation(utils::gen_range(0.0 - PI2, PI2)),
        bouncer::Bouncer(Point2::new(0.0, 0.0), Point2::new(600.0, 600.0)),
        sprite::Sprite {
            asset_id: sprite::AssetId::Missile,
        },
    )
}

#[wasm_bindgen]
pub struct Main {
    world: World,
    frame: render_frame::RenderFrame,
}

#[wasm_bindgen]
impl Main {
    pub fn new() -> Main {
        utils::set_panic_hook();

        let mut world = World::new();
        let frame = render_frame::RenderFrame::new(10);

        let bouncers = (0..10).map(|_| create_bouncing_entity());
        world.append_components(bouncers);

        Main { world, frame }
    }

    pub fn update(&mut self, time_delta: f32) {
        let dt = DeltaTime(time_delta / 1000.0);
        let world = &mut self.world;

        let update_funcs = [motion::update_motion, bouncer::update_bouncers];
        for update_func in update_funcs.iter() {
            update_func(world, dt);
        }

        self.frame.snapshot_world(world);
    }

    pub fn get_render_size(&self) -> usize {
        self.frame.size()
    }
    pub fn get_render_asset_ids(&self) -> *const u8 {
        self.frame.asset_ids()
    }
    pub fn get_render_pos_x(&self) -> *const f32 {
        self.frame.pos_x()
    }
    pub fn get_render_pos_y(&self) -> *const f32 {
        self.frame.pos_y()
    }
    pub fn get_render_orientation(&self) -> *const f32 {
        self.frame.orientation()
    }

    pub fn start(&self) {}
    pub fn stop(&self) {}
    pub fn pause(&self) {}
    pub fn resume(&self) {}
}
