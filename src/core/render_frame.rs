use pyro::*;

use motion;
use sprite;

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
            .matcher::<All<(
                Read<motion::Position>,
                Read<motion::Orientation>,
                Read<sprite::Sprite>,
            )>>()
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

impl RenderFrame {
    pub fn size(&self) -> usize {
        self.size
    }
    pub fn asset_ids(&self) -> *const u8 {
        self.asset_id.as_ptr()
    }
    pub fn pos_x(&self) -> *const f32 {
        self.pos_x.as_ptr()
    }
    pub fn pos_y(&self) -> *const f32 {
        self.pos_y.as_ptr()
    }
    pub fn orientation(&self) -> *const f32 {
        self.orientation.as_ptr()
    }
}
