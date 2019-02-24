extern crate cfg_if;
extern crate itertools;
extern crate nalgebra as na;
extern crate pyro;
extern crate wasm_bindgen;
extern crate web_sys;
extern crate js_sys;
extern crate rand;

mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

use pyro::*;
use itertools::Itertools;
use std::f32::consts::PI;
use std::time::Duration;
use rand::Rng;
use rand::rngs::OsRng;

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

#[derive(Debug, Copy, Clone)]
pub struct Position(pub na::Point2<f32>);

#[derive(Copy, Clone)]
pub struct Velocity(pub na::Vector2<f32>);

#[derive(Copy, Clone)]
pub struct Speed(pub f32);

#[derive(Copy, Clone)]
pub struct Enemy {
    pub health: f32,
}

#[derive(Copy, Clone)]
pub struct Explosion {
    pub radius: f32,
    pub max_radius: f32,
}

#[derive(Copy, Clone)]
pub struct Damage(pub f32);

pub struct TimeToLive {
    pub time_until_death: Duration,
}

pub type Missile<Projectile: Component> = (
    Position,
    Velocity,
    Render,
    Orientation,
    TimeToLive,
    Flip,
    Damage,
    Projectile,
);

pub struct Bullet;

pub type BulletEntity = (
    Position,
    Velocity,
    Render,
    Orientation,
    TimeToLive,
    Flip,
    Bullet,
);

#[derive(Debug, Copy, Clone)]
pub struct Render {
    pub asset: AssetId,
    pub scale: f32,
    pub inital_rotation: f32,
}

pub struct MoveTorwards {
    pub destination: na::Point2<f32>,
    pub side: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct Orientation(pub f32);

#[derive(Debug, Copy, Clone)]
pub enum Flip {
    Left,
    Right,
}

#[derive(Copy, Clone)]
pub struct Shoot {
    pub recover: Recover,
}

#[derive(Debug, Copy, Clone)]
pub struct DeltaTime(pub f32);

impl DeltaTime {
    pub fn as_duration(&self) -> Duration {
        Duration::from_millis((self.0 * 1000.0) as u64)
    }
}

#[derive(Copy, Clone)]
pub struct Recover {
    pub remaining: Duration,
    pub recover: Duration,
}

impl Recover {
    pub fn new(recover: Duration) -> Self {
        let remaining = recover.clone();
        Self {
            remaining,
            recover,
        }
    }
    pub fn action(&mut self, dt: DeltaTime) -> Option<()> {
        self.remaining =
            if let Some(remaining) = self.remaining.checked_sub(dt.as_duration()) {
                remaining
            } else {
                Duration::from_secs(0)
            };

        if self.remaining >= Duration::from_secs(0) {
            None
        } else {
            self.remaining = self.recover.clone();
            Some(())
        }
    }
}

pub fn shoot_at_enemy(world: &mut World, dt: DeltaTime) {
    let projectiles: Vec<_> = world
        .matcher::<All<(Read<Position>, Write<Shoot>)>>()
        .filter_map(|(&spawn_pos, shoot)| shoot.recover.action(dt).map(move |_| spawn_pos))
        .flat_map(|spawn_pos| {
            world
                .matcher::<All<(Read<Position>, Read<Enemy>)>>()
                .take(10)
                .map(move |(&target_pos, _)| {
                    let dir = (target_pos.0 - spawn_pos.0).normalize();
                    let offset = dir * 30.0;
                    let new_pos = Position(spawn_pos.0 + offset);
                    create_missile(AssetId::Missile, new_pos, dir, 700.0, SpawnMissile {})
                })
        }).collect();
    world.append_components(projectiles);
}

pub fn update_orientation(world: &mut World) {
    world
        .matcher::<All<(Read<Velocity>, Write<Orientation>)>>()
        .for_each(|(vel, orientation)| {
            let dir = vel.0.normalize();
            let mut angle = na::Matrix::angle(&dir, &na::Vector2::new(1.0, 0.0));
            if dir.y < 0.0 {
                angle = -angle;
            }
            orientation.0 = angle;
        });
}

pub fn move_torwards(world: &mut World, dt: DeltaTime) {
    world
        .matcher::<All<(
            Write<Position>,
            Read<MoveTorwards>,
            Read<Speed>,
            Write<Flip>,
        )>>().for_each(|(pos, target, speed, flip)| {
            let dir = (target.destination - pos.0).normalize();
            pos.0 += dir * speed.0 * dt.0;
            let angle = na::Matrix::angle(&na::Vector2::new(1.0, 0.0), &dir);

            *flip = if angle > PI / 2.0 {
                Flip::Left
            } else {
                Flip::Right
            };
        });
}

pub fn update_destination(world: &mut World, sides: &Sides) {
    world
        .matcher::<All<(Read<Position>, Write<MoveTorwards>)>>()
        .for_each(|(pos, target)| {
            let distance = na::distance(&target.destination, &pos.0);
            if distance <= 1.0 {
                *target = sides.get_random_point(target.side);
            }
        });
}

pub fn create_radial_missiles<Projectile: Component + Copy>(
    pos: Position,
    speed: f32,
    offset: f32,
    count: usize,
    projectile: Projectile,
) -> impl Iterator<Item = Missile<Projectile>> {
    let step_size = 2.0 * PI / count as f32;
    (0..count)
        .scan(0.0, move |acc, _| {
            *acc += step_size;
            Some(*acc)
        }).map(move |angle| {
            let x = offset * f32::cos(angle);
            let y = offset * f32::sin(angle);
            let dir = na::Vector2::new(x, y).normalize();
            create_missile(AssetId::SmallMissile, pos, dir, speed, projectile)
        })
}

pub fn kill_enemies(world: &mut World) {
    let dead_enemies: Vec<_> = world
        .matcher_with_entities::<All<(Read<Enemy>,)>>()
        .filter_map(|(entity, (enemy,))| {
            if enemy.health <= 0.0 {
                Some(entity)
            } else {
                None
            }
        }).collect();
    world.remove_entities(dead_enemies);
}

pub trait OnProjectileHit {
    type Projectile: Component + Sized;
    fn finish(&mut self, _world: &mut World) {}
    fn on_projectile_hit(&mut self, pos: Position, projectile: &Self::Projectile);
    fn hit(&mut self, world: &mut World) {
        const HIT_RADIUS: f32 = 10.0;
        let mut explosions = Vec::new();
        let mut entities = Vec::new();
        world
            .matcher_with_entities::<All<(Read<Self::Projectile>, Read<Position>, Read<Damage>)>>()
            .for_each(|(entity, (projectile, &missile, damage))| {
                let colliding_enemy = world
                    .matcher::<All<(Write<Enemy>, Read<Position>)>>()
                    .find_map(|(enemy, enemy_pos)| {
                        if na::distance(&missile.0, &enemy_pos.0) <= HIT_RADIUS {
                            Some(enemy)
                        } else {
                            None
                        }
                    });

                if let Some(enemy) = colliding_enemy {
                    enemy.health -= damage.0;
                    self.on_projectile_hit(missile, projectile);
                    explosions.push((
                        Explosion {
                            radius: 0.0,
                            max_radius: 25.0,
                        },
                        missile,
                    ));
                    entities.push(entity);
                }
            });
        world.append_components(explosions);
        world.remove_entities(entities);
        self.finish(world);
    }
}

#[derive(Copy, Clone)]
pub struct StandardMissile;
pub struct StandardMissileSystem;
impl StandardMissileSystem {
    pub fn new() -> Self {
        StandardMissileSystem {}
    }
}
impl OnProjectileHit for StandardMissileSystem {
    type Projectile = StandardMissile;
    fn on_projectile_hit(&mut self, _pos: Position, _projectile: &Self::Projectile) {}
}

#[derive(Copy, Clone)]
pub struct SpawnMissile;
pub struct SpawnMissileSystem {
    spawn: Vec<Missile<StandardMissile>>,
}
impl SpawnMissileSystem {
    pub fn new() -> Self {
        Self { spawn: Vec::new() }
    }
}
impl OnProjectileHit for SpawnMissileSystem {
    type Projectile = SpawnMissile;
    fn on_projectile_hit(&mut self, pos: Position, _projectile: &Self::Projectile) {
        let missiles = create_radial_missiles(pos, 150.0, 15.0, 12, StandardMissile {});
        self.spawn.extend(missiles);
    }
    fn finish(&mut self, world: &mut World) {
        let spawn = self.spawn.drain(0..);
        world.append_components(spawn);
    }
}

pub fn animate_explosion(world: &mut World, dt: DeltaTime) {
    const EXPANSION_SPEED: f32 = 25.0;
    world
        .matcher::<All<(Write<Explosion>, Read<Position>)>>()
        .for_each(|(explosion, _)| {
            explosion.radius += EXPANSION_SPEED * dt.0;
        });
    let entities: Vec<_> = world
        .matcher_with_entities::<All<(Write<Explosion>,)>>()
        .filter_map(|(entity, (explosion,))| {
            if explosion.radius >= explosion.max_radius {
                Some(entity)
            } else {
                None
            }
        }).collect();
    world.remove_entities(entities);
}

pub fn create_bullet(location: Position, target: Position, speed: f32) -> BulletEntity {
    let dir = (target.0 - location.0).normalize() * speed;
    (
        location,
        Velocity(dir),
        Render {
            asset: AssetId::Missile,
            scale: 0.2,
            inital_rotation: PI / 2.0,
        },
        Orientation(0.0),
        TimeToLive {
            time_until_death: Duration::from_secs(3),
        },
        Flip::Right,
        Bullet {},
    )
}
pub fn create_missile<Projectile: Component>(
    asset: AssetId,
    location: Position,
    dir: na::Vector2<f32>,
    speed: f32,
    projectile: Projectile,
) -> Missile<Projectile> {
    (
        location,
        Velocity(dir * speed),
        Render {
            asset,
            scale: 1.0,
            inital_rotation: PI / 2.0,
        },
        Orientation(0.0),
        TimeToLive {
            time_until_death: Duration::from_secs(3),
        },
        Flip::Right,
        Damage(1.0),
        projectile,
    )
}

pub fn kill_entities(world: &mut World, dt: DeltaTime) {
    let entities: Vec<_> = world
        .matcher_with_entities::<All<(Write<TimeToLive>,)>>()
        .filter_map(|(entity, (time,))| {
            time.time_until_death =
                if let Some(remaining) = time.time_until_death.checked_sub(dt.as_duration()) {
                    remaining
                } else {
                    Duration::from_secs(0)
                };
            time.time_until_death -= dt.as_duration();
            if time.time_until_death >= Duration::from_secs(0) {
                Some(entity)
            } else {
                None
            }
        }).collect();
    world.remove_entities(entities);
}

pub fn move_velocity(world: &mut World, dt: DeltaTime) {
    world
        .matcher::<All<(Write<Position>, Read<Velocity>)>>()
        .for_each(|(pos, vel)| {
            pos.0 += vel.0 * dt.0;
        })
}

pub fn spawn_random_grunts(world: &mut World, count: usize, sides: &Sides) {
    let ships = (0..count).map(|_| {
        let move_torwards = sides.get_random_point(sides.get_random_side());
        (
            Position(sides.get_random_point(move_torwards.side).destination),
            move_torwards,
            Orientation(0.0),
            Speed(OsRng::new().unwrap().gen_range(150.0, 200.0)),
            Render {
                asset: AssetId::Grunt,
                scale: 1.0,
                inital_rotation: 0.0,
            },
            Flip::Right,
            Enemy { health: 100.0 },
        )
    });
    world.append_components(ships);
}

// Let's not overcomplicate the asset loading system for a simple demo
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AssetId {
    Grunt = 0,
    Missile = 1,
    SmallMissile = 2,
    Tower = 3,
    Explosion = 4,
}
pub fn assetid_as_usize(asset_id: AssetId) -> usize {
    match asset_id {
        AssetId::Grunt => 0,
        AssetId::Missile => 1,
        AssetId::SmallMissile => 2,
        AssetId::Tower => 3,
        AssetId::Explosion => 4,
    }
}

pub struct AssetSettings {
    pub scale: f32,
    pub rotation: f32,
}

pub struct Sides {
    waypoints: [Waypoints; 2],
}
impl Sides {
    pub fn new((width, height): (f32, f32), spacing: f32, count: usize) -> Sides {
        let left = Waypoints::line((spacing, height), spacing, count);
        let right = Waypoints::line((width - spacing, height), spacing, count);
        Sides {
            waypoints: [left, right],
        }
    }

    pub fn get_random_side(&self) -> usize {
        OsRng::new().unwrap().gen_range(0, self.waypoints.len())
    }
    pub fn get_random_point(&self, previous_side: usize) -> MoveTorwards {
        let next_side = (previous_side + 1) % self.waypoints.len();
        MoveTorwards {
            destination: self.waypoints[next_side].get_random_point(),
            side: next_side,
        }
    }
}

pub struct Waypoints {
    pub points: Vec<na::Point2<f32>>,
}

impl Waypoints {
    pub fn line((offset, height): (f32, f32), spacing: f32, count: usize) -> Self {
        let adjusted_height = height - spacing;
        let step = (adjusted_height - spacing) / count as f32;
        let create_waypoints = |offset: f32| {
            (0..count).scan(na::Point2::new(0.0f32, spacing), move |state, _| {
                *state += na::Vector2::new(0.0, step);
                state.x = offset;
                Some(*state)
            })
        };
        let mut points = Vec::new();
        points.extend(create_waypoints(offset));
        Waypoints { points }
    }
    pub fn get_random_point(&self) -> na::Point2<f32> {
        let index: usize = OsRng::new().unwrap().gen_range(0, self.points.len());
        self.points[index]
    }
}
pub struct EnemySpawner {
    pub enemies_to_spawn: usize,
}
impl EnemySpawner {
    pub fn spawn_enemies(&mut self, world: &mut World, sides: &Sides) {
        let living_enemies = world.matcher::<All<(Read<Enemy>,)>>().count();
        if living_enemies > 0 {
            return;
        }
        spawn_random_grunts(world, self.enemies_to_spawn, &sides);
    }
}

pub fn spawn_towers(world: &mut World, (width, height): (f32, f32), offset: f32) {
    let spawn_points = [
        na::Point2::new(0.0 + offset, 0.0 + offset),
        na::Point2::new(width - offset, 0.0 + offset),
        na::Point2::new(width - offset, height - offset),
        na::Point2::new(0.0 + offset, height - offset),
    ];
    let towers = spawn_points.iter().map(|&pos| {
        (
            Position(pos),
            Render {
                asset: AssetId::Tower,
                scale: 1.0,
                inital_rotation: 0.0,
            },
            Shoot {
                recover: Recover::new(Duration::from_millis(250)),
            },
            Orientation(0.0),
            Flip::Right,
        )
    });
    world.append_components(towers);
}

#[wasm_bindgen]
pub struct RenderFrame {
    capacity: usize,
    size: usize,
    asset_id: Vec<usize>,
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
            .matcher::<All<(Read<Position>, Read<Orientation>, Read<Flip>, Read<Render>)>>()
            .collect();
        let capacity_needed = entities.len();
        if capacity_needed > self.capacity {
            self.resize(capacity_needed * 2);
        }
        self.clear();
        let mut idx = 0;
        for (pos, orientation, _flip, render) in entities {
            self.asset_id.push(assetid_as_usize(render.asset));
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
    pub fn asset_ids(&self) -> *const usize { self.asset_id.as_ptr() }
    pub fn pos_x(&self) -> *const f32 { self.pos_x.as_ptr() }
    pub fn pos_y(&self) -> *const f32 { self.pos_y.as_ptr() }
    pub fn orientation(&self) -> *const f32 { self.orientation.as_ptr() }
}

#[wasm_bindgen]
pub struct Main {
    world: World,
    sides: Sides,
    spawner: EnemySpawner,
    render_frame: RenderFrame,
}

#[wasm_bindgen]
impl Main {
    pub fn new() -> Main {
        utils::set_panic_hook();
        let mut world = World::new();
        let size = (700.0, 700.0);
        /*
            ctx.conf.window_mode.width as f32,
            ctx.conf.window_mode.height as f32,
        );
        */
        let sides = Sides::new(size, 100.0, 100);
        spawn_towers(&mut world, size, 50.0);
        let spawner = EnemySpawner {
            enemies_to_spawn: 500,
        };
        let render_frame = RenderFrame::new(1000);
        Main {
            world,
            sides,
            spawner,
            render_frame,
        }
    }

    pub fn get_render_size(&self) -> usize { self.render_frame.size() }
    pub fn get_render_asset_ids(&self) -> *const usize { self.render_frame.asset_ids() }
    pub fn get_render_pos_x(&self) -> *const f32 { self.render_frame.pos_x() }
    pub fn get_render_pos_y(&self) -> *const f32 { self.render_frame.pos_y() }
    pub fn get_render_orientation(&self) -> *const f32 { self.render_frame.orientation() }

    pub fn start(&self) {
    }

    pub fn stop(&self) {
    }

    pub fn pause(&self) {
    }

    pub fn resume(&self) {
    }

    pub fn update(&mut self, time_delta: f32) {
        let dt = DeltaTime(time_delta / 1000.0);
        let world = &mut self.world;

        self.spawner.spawn_enemies(world, &self.sides);
        move_torwards(world, dt);
        update_destination(world, &self.sides);
        move_velocity(world, dt);
        kill_entities(world, dt);
        update_orientation(world);
        animate_explosion(world, dt);
        shoot_at_enemy(world, dt);
        kill_enemies(world);
        StandardMissileSystem::new().hit(world);
        SpawnMissileSystem::new().hit(world);

        self.render_frame.snapshot_world(world);
    }
}
