use macroquad::prelude::*;

//pub const GAME_SIZE_X: i32 = 160;
pub const GAME_SIZE_X: i32 = 240;
pub const GAME_SIZE_Y: i32 = 130;
pub const GAME_CENTER_X: f32 = GAME_SIZE_X as f32 * 0.5f32;
pub const GAME_CENTER_Y: f32 = GAME_SIZE_Y as f32 * 0.5f32;
pub const _ASPECT_RATIO: f32 = GAME_SIZE_X as f32 / GAME_SIZE_Y as f32;

pub const KEY_RIGHT: KeyCode = KeyCode::Right;
pub const KEY_LEFT: KeyCode = KeyCode::Left;
pub const KEY_SHOOT: KeyCode = KeyCode::Space;
pub const KEY_START_GAME: KeyCode = KeyCode::Space;

pub const SCORE_NORMAL: i32 = 100;
pub const SCORE_MINI: i32 = 20;

pub const SCORE_KILL_ALL: i32 = 1000;
pub const SCORE_SURVIVED_ALL: i32 = 750;

pub const PLAYER_SPEED: f32 = 90f32;
pub const PLAYER_SHOOT_TIME: f32 = 0.8f32;
pub const PLAYER_BULLET_SPEED: f32 = 80f32;
pub const PLAYER_LIVES_START: i32 = 3i32;
pub const PLAYER_LIVES_MAX: i32 = 7i32;
pub const PLAYER_TIME_INVISBLE: f32 = 2f32;

pub const ENEMY_SPEED: f32 = 50.0f32;
pub const ENEMY_ANGLE_SPEED_RANGE: Vec2 = const_vec2!([0.2f32, 3f32]);

pub const ENEMY_SPEED_HOMING: Vec2 = const_vec2!([60f32, 30f32]);
pub const ENEMY_BULLET_SPEED: f32 = 80f32;
pub const ENEMY_SHOOT_TIME: f32 = 2f32;
// when shooting more than 1 bullet
pub const ENEMY_SHOOT_BURST_TIME: f32 = 0.2f32;
pub const ENEMY_MAX_BURST_COUNT: i32 = 5;
pub const ENEMY_ANIM_TIME_SPAWN: f32 = 0.7f32;
pub const ENEMY_MINI_ANIM_TIME_SPAWN: f32 = 0.3f32;
pub const ENEMY_ANIM_TIME_FLAP: f32 = 0.12f32;
pub const ENEMY_ANIM_SPAWN_SCALE: f32 = 4.0f32;
// how far away the spawn animation starts
pub const ENEMY_ANIM_DISTANCE: f32 = 140f32;
// The min to max time until a mini will start homing
pub const ENEMY_MINI_HOMING_TIME_RANGE: Vec2 = const_vec2!([4f32, 10f32]);

// Enemy Spawn management
pub const ENEMY_SPAWN_STARTING_COUNT: i32 = 2;
pub const ENEMY_SPAWN_MAX_COUNT: i32 = 9;
pub const TIME_UNTIL_MAX_DIFFICULTY: f32 = 70f32;
// spawn every x sec
pub const ENEMY_SPAWN_TIME: f32 = 0.5f32;

pub const BULLET_ANIM_TIME_SPAWN: f32 = 0.3f32;
