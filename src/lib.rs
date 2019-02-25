extern crate cfg_if;
extern crate itertools;
extern crate nalgebra as na;
extern crate pyro;
extern crate wasm_bindgen;
extern crate web_sys;
extern crate js_sys;
extern crate rand;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

use pyro::*;
use rand::Rng;
use rand::rngs::OsRng;
use na::{Point2, Vector2};

use std::f32::consts::PI;
const PI2: f32 = PI * 2.0;

mod utils;

mod core;
use core::*;

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
        motion::Position(Point2::new(gen_range(0.0, 400.0), gen_range(0.0, 400.0))),
        motion::Velocity(Vector2::new(gen_range(100.0, 500.0), gen_range(100.0, 500.0))),
        motion::Orientation(0.0),
        motion::Rotation(gen_range(0.0 - PI2, PI2)),
        bouncer::Bouncer(Point2::new(0.0, 0.0), Point2::new(600.0, 600.0)),
        sprite::Sprite { asset_id: sprite::AssetId::Missile },
    )
}

// ----------------------------------------------------------------------

#[wasm_bindgen]
pub struct RenderFrame {
    capacity: usize,
    size: usize,
    asset_id: Vec<u8>,
    pos_x: Vec<f32>,
    pos_y: Vec<f32>,
    orientation: Vec<f32>,
}

impl RenderFrame {
    pub fn new(capacity: usize) -> RenderFrame {
        RenderFrame {
            capacity,
            size: capacity,
            asset_id: Vec::with_capacity(capacity),
            pos_x: Vec::with_capacity(capacity),
            pos_y: Vec::with_capacity(capacity),
            orientation: Vec::with_capacity(capacity),
        }
    }

    pub fn capacity(&self) -> usize { self.capacity }

    pub fn resize(&mut self, capacity: usize) {
        self.capacity = capacity;
        self.asset_id.resize(capacity, 0);
        self.pos_x.resize(capacity, 0.0);
        self.pos_y.resize(capacity, 0.0);
        self.orientation.resize(capacity, 0.0);
    }

    pub fn clear(&mut self) {
        self.asset_id.clear();
        self.pos_x.clear();
        self.pos_y.clear();
        self.orientation.clear();
    }

    pub fn snapshot_world(&mut self, world: &World) {
        let entities: Vec<_> = world
            .matcher::<All<(Read<motion::Position>, Read<motion::Orientation>, Read<sprite::Sprite>)>>()
            .collect();

        let capacity_needed = entities.len();
        if capacity_needed > self.capacity {
            self.resize(capacity_needed * 2);
        }
        self.clear();

        let mut idx = 0;
        for (pos, orientation, sprite) in entities {
            self.asset_id.push(sprite::assetid_as_u8(sprite.asset_id));
            self.pos_x.push(pos.0.x);
            self.pos_y.push(pos.0.y);
            self.orientation.push(orientation.0);
            idx += 1;
        }
        self.size = idx;
    }
}

#[wasm_bindgen]
impl RenderFrame {
    pub fn size(&self) -> usize { self.size }
    pub fn asset_ids(&self) -> *const u8 { self.asset_id.as_ptr() }
    pub fn pos_x(&self) -> *const f32 { self.pos_x.as_ptr() }
    pub fn pos_y(&self) -> *const f32 { self.pos_y.as_ptr() }
    pub fn orientation(&self) -> *const f32 { self.orientation.as_ptr() }
}

pub fn gen_range(low: f32, high: f32) -> f32 {
    OsRng::new().unwrap().gen_range(low, high)
}


#[wasm_bindgen]
pub struct Main {
    world: World,
    render_frame: RenderFrame,
}

#[wasm_bindgen]
impl Main {
    pub fn new() -> Main {
        utils::set_panic_hook();

        let mut world = World::new();
        let render_frame = RenderFrame::new(10);

        let bouncers = (0..50).map(|_| {
            create_bouncing_entity()
        });
        world.append_components(bouncers);

        Main {
            world,
            render_frame,
        }
    }

    pub fn update(&mut self, time_delta: f32) {
        let dt = DeltaTime(time_delta / 1000.0);
        let world = &mut self.world;

        let update_funcs = [
            motion::update_motion,
            bouncer::update_bouncers,
        ];
        for update_func in update_funcs.iter() {
            update_func(world, dt);
        }

        self.render_frame.snapshot_world(world);
    }

    pub fn get_render_size(&self) -> usize { self.render_frame.size() }
    pub fn get_render_asset_ids(&self) -> *const u8 { self.render_frame.asset_ids() }
    pub fn get_render_pos_x(&self) -> *const f32 { self.render_frame.pos_x() }
    pub fn get_render_pos_y(&self) -> *const f32 { self.render_frame.pos_y() }
    pub fn get_render_orientation(&self) -> *const f32 { self.render_frame.orientation() }

    pub fn start(&self) { }
    pub fn stop(&self) { }
    pub fn pause(&self) { }
    pub fn resume(&self) { }
}
