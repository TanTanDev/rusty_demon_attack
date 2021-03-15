use macroquad::prelude::*;
use std::collections::HashMap;

//const GAME_SIZE_X: i32 = 160;
const GAME_SIZE_X: i32 = 240;
const GAME_SIZE_Y: i32 = 130;
const GAME_CENTER_X: f32 = GAME_SIZE_X as f32 * 0.5f32;
const GAME_CENTER_Y: f32 = GAME_SIZE_Y as f32 * 0.5f32;
const _ASPECT_RATIO: f32 = GAME_SIZE_X as f32 / GAME_SIZE_Y as f32;

const KEY_RIGHT: KeyCode = KeyCode::Right;
const KEY_LEFT: KeyCode = KeyCode::Left;
const KEY_SHOOT: KeyCode = KeyCode::Space;

const SCORE_NORMAL: i32 = 100;
const SCORE_MINI: i32 = 20;

const SCORE_KILL_ALL: i32 = 1000;
const SCORE_SURVIVED_ALL: i32 = 750;

const PLAYER_SPEED: f32 = 80f32;
const PLAYER_SHOOT_TIME: f32 = 0.8f32;
const PLAYER_BULLET_SPEED: f32 = 80f32;
const PLAYER_LIVES_START: i32 = 7i32;
const PLAYER_LIVES_MAX: i32 = 7i32;
const PLAYER_TIME_INVISBLE: f32 = 2f32;

const ENEMY_SPEED: f32 = 50.0f32;
const ENEMY_SPEED_HOMING: Vec2 = Vec2{x: 60f32, y: 30f32};
const ENEMY_BULLET_SPEED: f32 = 80f32;
const ENEMY_SHOOT_TIME: f32 = 2f32;
// when shooting more than 1 bullet
const ENEMY_SHOOT_BURST_TIME: f32 = 0.2f32;
const ENEMY_MAX_BURST_COUNT: i32 = 5;
const ENEMY_ANIM_TIME_SPAWN: f32 = 0.7f32;
const ENEMY_MINI_ANIM_TIME_SPAWN: f32 = 0.3f32;
const ENEMY_ANIM_TIME_FLAP: f32 = 0.12f32;
const ENEMY_ANIM_SPAWN_SCALE: f32 = 4.0f32;
// how far away the spawn animation starts
const ENEMY_ANIM_DISTANCE: f32 = 140f32;
// The min to max time until a mini will start homing
const ENEMY_MINI_HOMING_TIME_RANGE: Vec2 = Vec2{x: 4f32, y: 10f32};

// Enemy Spawn management
const ENEMY_MAX_COUNT: i32 = 5;
// spawn every x sec
const ENEMY_SPAWN_TIME: f32 = 0.5f32;


const BULLET_ANIM_TIME_SPAWN: f32 = 0.3f32; 

fn window_conf() -> Conf {
    Conf {
        window_title: "Demottack".to_owned(),
        window_width: GAME_SIZE_X,
        window_height: GAME_SIZE_Y,
        ..Default::default()
    }
}


#[derive(std::cmp::PartialEq)]
pub enum BulletHurtType {
    Player,
    Enemy,
}

pub struct Bullet {
    texture: Texture2D,
    pos: Vec2,
    vel: Vec2,
    hurt_type: BulletHurtType,
    anim_timer: f32,
    collision_rect: Rect,
    is_kill: bool,
}

impl Bullet {
    pub fn new(pos: Vec2, hurt_type: BulletHurtType, textures: &Textures) -> Self {
        let (vel, texture) = match hurt_type{
            BulletHurtType::Enemy => (vec2(0f32, -1f32 * PLAYER_BULLET_SPEED), textures.player_missile), 
            BulletHurtType::Player => (vec2(0f32, ENEMY_BULLET_SPEED), textures.demon_missile),
        };

        Bullet {
            pos,
            texture,
            vel,
            hurt_type,
            anim_timer: 0f32,
            collision_rect: Rect::new(pos.x, pos.y, 2.0f32, 6f32),
            is_kill: false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.pos += self.vel*dt;
        self.anim_timer += dt;
        self.collision_rect.x = self.pos.x;
        self.collision_rect.y = self.pos.y;
    }

    pub fn overlaps(&self, other_rect: &Rect) -> bool {
        self.collision_rect.overlaps(other_rect)
    }

    pub fn draw(&mut self) {
        let rect = &self.collision_rect;
        //draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2f32, GREEN);

        let frame = ((self.anim_timer / BULLET_ANIM_TIME_SPAWN) * 3.0f32) as i32; 
        draw_texture_ex(
            self.texture,
            self.pos.x,
            self.pos.y,
            WHITE,
            DrawTextureParams {
                rotation: 0f32,
                source: Some(Rect::new(
                    self.texture.width() / 3f32 * frame as f32,
                    0f32,
                    self.texture.width() / 3f32,
                    self.texture.height(),
                )),
                ..Default::default()
            },
        );
    }
}

#[derive(PartialEq)]
pub enum EnemyState {
    Spawning(EnemyStateSpawning),
    Normal(EnemyStateNormal),
    Shooting(EnemyStateShooting),
    Homing(EnemyStateHoming),
}

#[derive(Clone, Copy)]
pub enum EnemyDeathMethod {
    None,
    // count
    SpawnChildren(i32),
}

pub enum EnemyCommand {
    ChangeState(EnemyState),
}


#[derive(Clone, Copy)]
pub enum EnemyType {
    Normal,
    Mini,
}

#[derive(Clone, Copy)]
pub enum EnemyColor {
    Purple,
    Green,
    Red,
}

impl EnemyColor {
    pub fn random() -> Self {
        use EnemyColor::*;
        let all = [Purple, Green, Red];
        all[rand::gen_range(0, all.len())]
    }
}

pub struct EnemyStateShared {
    texture: Texture2D,
    pos: Vec2,
    collision_rect: Rect,
    health: i32,
    death_method: EnemyDeathMethod,
    animation_timer: f32,
    enemy_type: EnemyType,
    enemy_color: EnemyColor, 

    // used for mini enemies, that home in on player
    charge_timer_optional: Option<f32>,
}

#[derive(PartialEq)]
pub struct EnemyStateNormal {
    shoot_timer: f32,
}

#[derive(PartialEq)]
pub struct EnemyStateHoming {
}

#[derive(PartialEq)]
pub struct EnemyStateShooting {
    shots_left: i32,
    shoot_timer: f32,
}

#[derive(PartialEq)]
pub struct EnemyStateSpawning {
    spawn_timer: f32,
}

pub struct Enemy {
    state_shared: EnemyStateShared,
    state: EnemyState,
}

impl Enemy {
    pub fn new(pos: Vec2, texture: Texture2D, health: i32, death_method: EnemyDeathMethod, enemy_type: EnemyType, enemy_color: EnemyColor) -> Self {
        let charge_timer_optional = match enemy_type{
            EnemyType::Normal => None,
            EnemyType::Mini => Some(rand::gen_range(ENEMY_MINI_HOMING_TIME_RANGE.x, ENEMY_MINI_HOMING_TIME_RANGE.y)),
        };
        Enemy {
            state_shared: EnemyStateShared {
                pos,
                texture,
                collision_rect: Rect::new(0f32, 0f32, texture.width(), texture.height()),
                health,
                death_method,
                animation_timer: 0f32,
                enemy_type,
                charge_timer_optional,
                enemy_color,
            },
            state: EnemyState::Spawning(EnemyStateSpawning{
                spawn_timer: 0f32,
            }),
        }
    }

    pub fn update(&mut self, dt: f32, bullets: &mut Vec::<Bullet>, textures: &Textures, player_pos: &Vec2, game_manager: &mut WaveManager) {
        let command_optional = match &mut self.state {
            EnemyState::Spawning(state_data) => Self::update_state_spawning(&mut self.state_shared, dt, state_data),
            EnemyState::Normal(state_data) => Self::update_state_normal(&mut self.state_shared, dt, bullets, textures, state_data),
            EnemyState::Shooting(state_data) => Self::update_state_shooting(&mut self.state_shared, dt, bullets, textures, state_data),
            EnemyState::Homing(state_data) => Self::update_state_homing(&mut self.state_shared, dt, state_data, player_pos, game_manager),
        };
        match command_optional {
            None => {},
            Some(command) => {
                match command {
                    EnemyCommand::ChangeState(new_state) => {
                        self.state = new_state;
                    }
                }
            }
        };
    }


    pub fn overlaps(&self, other_rect: &Rect) -> bool {
        self.state_shared.collision_rect.overlaps(other_rect)
    }

    pub fn clamp_in_view(pos: &mut Vec2) {
        let x_padding = 4f32;
        if pos.x < x_padding {
            pos.x = x_padding;
        } else if pos.x > GAME_SIZE_X as f32 - x_padding {
            pos.x = GAME_SIZE_X as f32 - x_padding
        }
        let top_padding = 7f32;
        let bottom_padding = 60f32;
        if pos.y < top_padding {
            pos.y = top_padding;
        } else if pos.y > GAME_SIZE_Y as f32 - bottom_padding {
            pos.y = GAME_SIZE_Y as f32 - bottom_padding;
        }

    }

    fn update_state_spawning(state_shared: &mut EnemyStateShared, dt: f32, state_data: &mut EnemyStateSpawning) -> Option<EnemyCommand> {
        state_data.spawn_timer += dt;
        // different enemy types spawn differently 
        let end_time = match state_shared.enemy_type{
            EnemyType::Normal => ENEMY_ANIM_TIME_SPAWN,
            EnemyType::Mini => ENEMY_MINI_ANIM_TIME_SPAWN,
        };

        let fraction = state_data.spawn_timer / end_time;
        if fraction >= 1.0f32 {
            return Some(EnemyCommand::ChangeState(EnemyState::Normal(EnemyStateNormal {shoot_timer: 0f32, })));
        }
        None
    }

    fn update_state_normal(state_shared: &mut EnemyStateShared, dt: f32, bullets: &mut Vec::<Bullet>, textures: &Textures, state_data: &mut EnemyStateNormal) -> Option<EnemyCommand> {
        state_shared.pos.x += rand::gen_range(-1f32, 1f32) * ENEMY_SPEED * dt;
        state_shared.pos.y += rand::gen_range(-1f32, 1f32) * ENEMY_SPEED * dt;
        Self::clamp_in_view(&mut state_shared.pos);
        state_shared.collision_rect.x = state_shared.pos.x - state_shared.texture.width()*0.5f32;
        state_shared.collision_rect.y = state_shared.pos.y;

        state_data.shoot_timer += dt;
        // update timer for charging in on player
        if let Some(charge_timer) = &mut state_shared.charge_timer_optional {
            *charge_timer -= dt;
            if *charge_timer <= 0f32 {
                return Some(EnemyCommand::ChangeState(EnemyState::Homing(EnemyStateHoming{})));
            }
        }

        state_shared.animation_timer += dt;
        if state_shared.animation_timer > ENEMY_ANIM_TIME_FLAP*4f32 {
            state_shared.animation_timer -= ENEMY_ANIM_TIME_FLAP*4f32;
        }
        if state_data.shoot_timer > ENEMY_SHOOT_TIME {
            let shot_count = rand::gen_range(1, ENEMY_MAX_BURST_COUNT);
            return Some(EnemyCommand::ChangeState(EnemyState::Shooting(EnemyStateShooting{shoot_timer: ENEMY_SHOOT_BURST_TIME, shots_left: shot_count,})))
        }
        None
    }

    fn update_state_shooting(state_shared: &mut EnemyStateShared, dt: f32, bullets: &mut Vec::<Bullet>, textures: &Textures, state_data: &mut EnemyStateShooting) -> Option<EnemyCommand> {
        state_shared.pos.x += rand::gen_range(-1f32, 1f32) * ENEMY_SPEED * 0.5f32 * dt;
        state_shared.pos.y += rand::gen_range(-1f32, 1f32) * ENEMY_SPEED * 0.5f32 * dt;
        Self::clamp_in_view(&mut state_shared.pos);
        state_data.shoot_timer -= dt;
        // SPAWN SHOT
        if state_data.shoot_timer <= 0f32 {
            state_data.shoot_timer = ENEMY_SHOOT_BURST_TIME;
            state_data.shots_left -= 1;

            let should_spawn_2 = rand::gen_range(0, 2) > 0;
            if should_spawn_2 {
                let spawn_offset = vec2((state_shared.texture.width()/4f32)*0.5f32, 0f32);
                bullets.push(Bullet::new(state_shared.pos + spawn_offset, BulletHurtType::Player, &textures));
                bullets.push(Bullet::new(state_shared.pos - spawn_offset, BulletHurtType::Player, &textures));
            }
            else {
                let spawn_offset = vec2(0f32, -3f32);
                bullets.push(Bullet::new(state_shared.pos + spawn_offset, BulletHurtType::Player, &textures));
            }

            // for fun move enemy up when shooting
            state_shared.pos.y -= 1f32;
        }

        state_shared.collision_rect.x = state_shared.pos.x - state_shared.texture.width()*0.5f32;
        state_shared.collision_rect.y = state_shared.pos.y;

        state_shared.animation_timer += dt;
        if state_shared.animation_timer > ENEMY_ANIM_TIME_FLAP*4f32 {
            state_shared.animation_timer -= ENEMY_ANIM_TIME_FLAP*4f32;
        }

        // done shooting
        if state_data.shots_left <= 0 {
            return Some(EnemyCommand::ChangeState(EnemyState::Normal(EnemyStateNormal{ shoot_timer: 0f32, })));
        }
        None
    }

    fn update_state_homing(state_shared: &mut EnemyStateShared, dt: f32, state_data: &mut EnemyStateHoming, player_pos: &Vec2, game_manager: &mut WaveManager) -> Option<EnemyCommand> {
        state_shared.animation_timer += dt;
        if state_shared.animation_timer > ENEMY_ANIM_TIME_FLAP*4f32 {
            state_shared.animation_timer -= ENEMY_ANIM_TIME_FLAP*4f32;
        }
        // MOVE TOWARDS PLAYER
        let player_dx = player_pos.x - state_shared.pos.x;
        let dx = if player_dx > 0f32 {1f32} else {-1f32};
        let sway_speed = 20f32;
        let sway = (get_time() as f32 * sway_speed).sin(); 
        // remap from -1 -> 1 TO 0 -> 1
        let sway = (sway + 1f32)*0.5f32; 

        let vel = vec2(dx * ENEMY_SPEED_HOMING.x*sway, ENEMY_SPEED_HOMING.y);
        state_shared.pos += vel * dt;
        state_shared.collision_rect.x = state_shared.pos.x - state_shared.texture.width()*0.5f32;
        state_shared.collision_rect.y = state_shared.pos.y;

        // kill monsters below screen
        if state_shared.pos.y > GAME_SIZE_Y as f32 {
            state_shared.health = 0;
            game_manager.last_enemy_death_reason = LastEnemyDeathReason::Environment;
        }
        // collision against player checks happends in main
        None
    }

    fn draw_state_spawning_normal(state_shared: &EnemyStateShared, state_data: &EnemyStateSpawning) {
        let rand_frame = rand::gen_range(0i32, 2i32);
        let fraction = 1.0f32 - state_data.spawn_timer / ENEMY_ANIM_TIME_SPAWN;
        let offset = fraction * ENEMY_ANIM_DISTANCE;
        let sprite_width = state_shared.texture.width()/3f32;
        let scale = sprite_width + fraction * ENEMY_ANIM_SPAWN_SCALE * sprite_width;
        // Left wing
        draw_texture_ex(
            state_shared.texture,
            state_shared.pos.x-((state_shared.texture.width()/3.0f32)*1.0f32) - offset,
            state_shared.pos.y,
            WHITE,
            DrawTextureParams {
                rotation: 0f32,
                dest_size: Some(vec2(scale, state_shared.texture.height())),
                source: Some(Rect::new(
                    state_shared.texture.width() / 3f32 * rand_frame as f32,
                    0f32,
                    state_shared.texture.width() / 3f32,
                    state_shared.texture.height(),
                )),
                ..Default::default()
            },
        );
        // right wing
        draw_texture_ex(
            state_shared.texture,
            state_shared.pos.x + offset,
            state_shared.pos.y,
            WHITE,
            DrawTextureParams {
                rotation: 0f32,
                flip_x: true,
                dest_size: Some(vec2(scale, state_shared.texture.height())),
                source: Some(Rect::new(
                    state_shared.texture.width() / 3f32 * rand_frame as f32,
                    0f32,
                    state_shared.texture.width() / 3f32,
                    state_shared.texture.height(),
                )),
                ..Default::default()
            },
        );
    }

    fn draw_state_spawning_mini(state_shared: &EnemyStateShared, state_data: &EnemyStateSpawning) {
        let rand_frame = rand::gen_range(0i32, 2i32);
        let fraction = state_data.spawn_timer / ENEMY_MINI_ANIM_TIME_SPAWN;
        let sprite_width = state_shared.texture.width()/4f32;
        let scale = sprite_width*0.5f32 + fraction * 1.5f32 * sprite_width;
        draw_texture_ex(
            state_shared.texture,
            state_shared.pos.x-((state_shared.texture.width()/4.0f32)*1.0f32),
            state_shared.pos.y,
            WHITE,
            DrawTextureParams {
                rotation: fraction*3.1415f32*2f32,
                dest_size: Some(vec2(scale, scale)),
                source: Some(Rect::new(
                    state_shared.texture.width() / 4f32 * rand_frame as f32,
                    0f32,
                    state_shared.texture.width() / 4f32,
                    state_shared.texture.height(),
                )),
                ..Default::default()
            },
        );
        // right wing
        draw_texture_ex(
            state_shared.texture,
            state_shared.pos.x,
            state_shared.pos.y,
            WHITE,
            DrawTextureParams {
                rotation: fraction*3.1415f32*2f32,
                flip_x: true,
                dest_size: Some(vec2(scale, scale)),
                source: Some(Rect::new(
                    state_shared.texture.width() / 4f32 * rand_frame as f32,
                    0f32,
                    state_shared.texture.width() / 4f32,
                    state_shared.texture.height(),
                )),
                ..Default::default()
            },
        );
    }

    fn draw_state_spawning(state_shared: &EnemyStateShared, state_data: &EnemyStateSpawning) {
        match state_shared.enemy_type {
            EnemyType::Normal => Self::draw_state_spawning_normal(state_shared, state_data), 
            EnemyType::Mini => Self::draw_state_spawning_mini(state_shared, state_data),
        }
    }

    fn draw_state_normal(&self) {
        let rand_frame = (self.state_shared.animation_timer/ENEMY_ANIM_TIME_FLAP).floor();  

        let time = get_time()*3.1415f64;
        let rot = (time.sin()+1f64)*0.5f64*3.141596f64*2f64;
        let rect = self.state_shared.collision_rect;
        //draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0f32, GREEN);

        //draw_circle(self.pos.x, self.pos.y, 1.0f32, RED);
        // Left wing
        draw_texture_ex(
            self.state_shared.texture,
            self.state_shared.pos.x-((self.state_shared.texture.width()/4.0f32)*1.0f32),
            self.state_shared.pos.y,
            WHITE,
            DrawTextureParams {
                rotation: 0f32,
                source: Some(Rect::new(
                    self.state_shared.texture.width() / 4f32 * rand_frame as f32,
                    0f32,
                    self.state_shared.texture.width() / 4f32,
                    self.state_shared.texture.height(),
                )),
                ..Default::default()
            },
        );
        // right wing
        draw_texture_ex(
            self.state_shared.texture,
            self.state_shared.pos.x,
            self.state_shared.pos.y,
            WHITE,
            DrawTextureParams {
                rotation: 0f32,
                flip_x: true,
                source: Some(Rect::new(
                    self.state_shared.texture.width() / 4f32 * rand_frame as f32,
                    0f32,
                    self.state_shared.texture.width() / 4f32,
                    self.state_shared.texture.height(),
                )),
                ..Default::default()
            },
        );
    }

    pub fn draw(&mut self) {
        match &self.state {
            EnemyState::Spawning(state_data) => Self::draw_state_spawning(&self.state_shared, state_data),
            EnemyState::Normal(_state_data) => self.draw_state_normal(),
            // enemy doesn't look different when shooting
            EnemyState::Shooting(_state_data) => self.draw_state_normal(),
            EnemyState::Homing(_state_data) => self.draw_state_normal(),
        }
    }
}

#[derive(PartialEq)]
pub enum PlayerState
{
    Normal,
    // time left to be invisible
    Invisible(f32),
}

pub enum PlayerCommand {
    ChangeState(PlayerState),
}

pub struct Player {
    pub pos: Vec2,
    texture: Texture2D,
    texture_explotion: Texture2D,
    bullet_decoy_texture: Texture2D,
    shoot_timer: f32,
    collision_rect: Rect,
    state: PlayerState,
}

impl Player {
    pub fn new(pos: Vec2, texture: Texture2D, bullet_decoy_texture: Texture2D, texture_explotion: Texture2D) -> Self {
        Player {
            pos, 
            texture,
            bullet_decoy_texture,
            texture_explotion,
            shoot_timer: 0f32,
            collision_rect: Rect::new(pos.x, pos.y, 7.0f32, 6.0f32),
            state: PlayerState::Normal,
        }
    }

    pub fn update(&mut self, dt: f32, bullets: &mut Vec::<Bullet>, textures: &Textures) {
        self.shoot_timer += dt;
        if is_key_down(KEY_LEFT) {
            self.pos.x -= PLAYER_SPEED * dt; 
            if self.pos.x < 0f32 {
                self.pos.x = 0f32;
            }
        }
        if is_key_down(KEY_RIGHT) {
            self.pos.x += PLAYER_SPEED * dt; 
            if self.pos.x > GAME_SIZE_X as f32 - self.texture.width() {
                self.pos.x = GAME_SIZE_X as f32 - self.texture.width();
            }
        }

        // state specific update
        let player_command_optional = match &mut self.state {
            PlayerState::Normal => {
                if is_key_down(KEY_SHOOT) {
                    if self.shoot_timer >= PLAYER_SHOOT_TIME {
                        let spawn_offset = vec2(3f32, -4f32);
                        bullets.push(Bullet::new(self.pos + spawn_offset, BulletHurtType::Enemy, &textures));
                        self.shoot_timer = 0f32;
                    }
                }
                None
            },
            PlayerState::Invisible(time_left) => {
                *time_left -= dt;
                if *time_left <= 0.0f32 {
                    Some(PlayerCommand::ChangeState(PlayerState::Normal))
                } else {
                None
                }
            }
        };

        self.process_command_optional(player_command_optional);

        self.collision_rect.x = self.pos.x;
        self.collision_rect.y = self.pos.y;
    }

    pub fn process_command_optional(&mut self, command_optional: Option<PlayerCommand>) {
        if let Some(player_command) = command_optional {
            match player_command {
                PlayerCommand::ChangeState(state) => {
                    self.state = state;
                }
            }
        }
    }

    pub fn overlaps(&self, other_rect: &Rect) -> bool {
        self.collision_rect.overlaps(other_rect)
    }

    pub fn draw(&self) {
        match self.state {
            PlayerState::Normal => self.draw_state_normal(),
            PlayerState::Invisible(time_left) => self.draw_state_invisible(&time_left),
        }
    }

    pub fn draw_state_normal(&self) {
        let rect = self.collision_rect;
        //draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0f32, GREEN);
        draw_texture_ex(
            self.texture,
            self.pos.x,
            self.pos.y,
            WHITE,
            DrawTextureParams {
                //dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        let decoy_frame_index = ((self.shoot_timer / PLAYER_SHOOT_TIME)*3f32) as i32;

        draw_texture_ex(
            self.bullet_decoy_texture,
            self.pos.x + 3.,
            self.pos.y - 1.,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    self.bullet_decoy_texture.width() / 3f32 * decoy_frame_index as f32,
                    0f32,
                    self.bullet_decoy_texture.width() / 3f32,
                    self.bullet_decoy_texture.height(),
                )),
                ..Default::default()
            },
        );
    }

    pub fn draw_state_invisible(&self, time_left: &f32) {
        let anim_frames = 7f32;
        let time_per_frame = PLAYER_TIME_INVISBLE / anim_frames;
        let fraction = (PLAYER_TIME_INVISBLE - time_left)/PLAYER_TIME_INVISBLE;
        let frame_index = (PLAYER_TIME_INVISBLE - time_left)/time_per_frame;
        let frame_index = frame_index.floor();

        draw_texture_ex(
            self.texture_explotion,
            self.pos.x-5f32,
            self.pos.y-4f32,
            WHITE,
            DrawTextureParams {
                rotation: fraction*3.1415f32*2f32,
                source: Some(Rect::new(
                    self.texture_explotion.width() / anim_frames * frame_index,
                    0f32,
                    self.texture_explotion.width() / anim_frames,
                    self.texture_explotion.height(),
                )),
                ..Default::default()
            },
        );
    }
}

pub struct Textures {
    demons_normal_purple: Vec::<Texture2D>,
    demons_normal_green: Vec::<Texture2D>,
    demons_normal_red: Vec::<Texture2D>,
    demons_mini_purple: Vec::<Texture2D>,
    demons_mini_green: Vec::<Texture2D>,
    demons_mini_red: Vec::<Texture2D>,

    demon_missile: Texture2D,
    player_missile: Texture2D,
}

impl Textures {
    pub fn new(demon_missile: Texture2D, player_missile: Texture2D) -> Self {
        Textures {
            demons_normal_purple: Vec::<Texture2D>::new(),
            demons_normal_green: Vec::<Texture2D>::new(),
            demons_normal_red: Vec::<Texture2D>::new(),
            demons_mini_purple: Vec::<Texture2D>::new(),
            demons_mini_green: Vec::<Texture2D>::new(),
            demons_mini_red: Vec::<Texture2D>::new(),
            demon_missile,
            player_missile,
        }
    }

    pub async fn load_texture(&mut self, file_name: &str, enemy_color: EnemyColor, enemy_type: EnemyType) {
        let texture: Texture2D = load_texture(file_name).await;
        set_texture_filter(texture, FilterMode::Nearest);
        let texture_vec = match enemy_type {
            EnemyType::Normal => {
                match enemy_color{
                    EnemyColor::Purple => &mut self.demons_normal_purple,
                    EnemyColor::Green => &mut self.demons_normal_green,
                    EnemyColor::Red => &mut self.demons_normal_red,
                }
            }
            EnemyType::Mini => {
                match enemy_color {
                    EnemyColor::Purple => &mut self.demons_mini_purple,
                    EnemyColor::Green => &mut self.demons_mini_green,
                    EnemyColor::Red => &mut self.demons_mini_red,
                }
            }
        };
        texture_vec.push(texture);
    }
    pub fn rand_enemy_normal(&self, enemy_color: EnemyColor) -> Texture2D {
        let normal_list = match enemy_color {
            EnemyColor::Purple => &self.demons_normal_purple, 
            EnemyColor::Green => &self.demons_normal_green,
            EnemyColor::Red => &self.demons_normal_red,
        };
        normal_list[rand::gen_range(0, normal_list.len())]
    }

    pub fn rand_enemy_mini(&self, enemy_color: EnemyColor) -> Texture2D {
        let mini_list = match enemy_color {
            EnemyColor::Purple => &self.demons_mini_purple, 
            EnemyColor::Green => &self.demons_mini_green,
            EnemyColor::Red => &self.demons_mini_red,
        };
        mini_list[rand::gen_range(0, mini_list.len())]
    }
}

pub struct WaveManagerStateSpawning {
    enemies_left: i32,
    spawn_timer: f32,
}
pub enum WaveManagerState {
    Spawning(WaveManagerStateSpawning),
    Battle,
}

// used to internally modify gamestate
pub enum WaveManagerCommand {
    ChangeState(WaveManagerState)
}

// used to get information from gamestate
pub enum WaveManagerMessage {
    LevelCleared
}

// the reason the last enemy died
#[derive(PartialEq)]
pub enum LastEnemyDeathReason {
    Environment,
    Player,
}

pub struct WaveManager {
    state: WaveManagerState,
    last_enemy_death_reason: LastEnemyDeathReason,
}

impl WaveManager {
    pub fn update(&mut self, dt: f32, enemies: &mut Vec<Enemy>, textures: &Textures) -> Option<WaveManagerMessage> {
        let state_command_optional = match &mut self.state {
            WaveManagerState::Spawning(game_state_spawning) => Self::update_state_spawning(game_state_spawning, dt, enemies, textures),
            WaveManagerState::Battle =>  Self::update_state_battle(dt, enemies),
        };

        if let Some(state_command) = state_command_optional {
            match state_command {
                WaveManagerCommand::ChangeState(target_state) => {
                    self.state = target_state;  

                    let cleared_screen = variant_eq(&self.state, &WaveManagerState::Spawning(WaveManagerStateSpawning{spawn_timer:0f32, enemies_left:0,}));
                    if cleared_screen {
                        return Some(WaveManagerMessage::LevelCleared);
                    }
                }
            }
        }
        None
    }

    fn update_state_battle(dt: f32, enemies: &mut Vec<Enemy>) -> Option<WaveManagerCommand> {
        if enemies.len() <= 0 {
            return Some(WaveManagerCommand::ChangeState(WaveManagerState::Spawning(WaveManagerStateSpawning{enemies_left: 9, spawn_timer: 0f32,})));
        }
        None
    }

    fn update_state_spawning(game_state_spawning: &mut WaveManagerStateSpawning, dt: f32, enemies: &mut Vec<Enemy>, textures: &Textures) -> Option<WaveManagerCommand> {
        game_state_spawning.spawn_timer += dt;
        if game_state_spawning.spawn_timer > ENEMY_SPAWN_TIME { 
            game_state_spawning.enemies_left -= 1;
            game_state_spawning.spawn_timer -= ENEMY_SPAWN_TIME;
            spawn_enemy(enemies, &textures, SpawnBlueprint::Normal, EnemyColor::random());
        }
        if game_state_spawning.enemies_left <= 0 {
            return Some(WaveManagerCommand::ChangeState(WaveManagerState::Battle));
        }
        None
    }
}

pub enum SpawnBlueprint {
    Normal,
    Mini(Vec2),
}

// construct an enemy with randomized features based on a blueprint
pub fn spawn_enemy(enemies: &mut Vec<Enemy>, textures: &Textures, spawn_blueprint: SpawnBlueprint, enemy_color: EnemyColor) { 
    let health = 1;
    let enemy = match spawn_blueprint{
        SpawnBlueprint::Normal => {
            let spawn_offset = vec2(rand::gen_range(-100f32, 100f32), rand::gen_range(-60f32, 10f32));
            let spawn_pos = vec2(GAME_CENTER_X, GAME_CENTER_Y) + spawn_offset;
            let death_method = if rand::gen_range(0f32, 1f32) > 0.5f32 {
                let spawn_amount = rand::gen_range(1, 2+1);
                EnemyDeathMethod::SpawnChildren(spawn_amount)
            } else {
                EnemyDeathMethod::None
            };

            Enemy::new(spawn_pos, textures.rand_enemy_normal(enemy_color), health, death_method, EnemyType::Normal, enemy_color)
        }
        SpawnBlueprint::Mini(pos) => {
            Enemy::new(pos, textures.rand_enemy_mini(enemy_color), health, EnemyDeathMethod::None, EnemyType::Mini, enemy_color)
        }
    };
    enemies.push(enemy);
}

// used to compare enums without having to match against it's values 
// example what we avoid: emotion_enum == Emotion::Happy{happines_level: 0f32, visible_on_face: false,}
// the values needs to be constructed, but comparison is top-level
fn variant_eq<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

// okay this is pretty hacky...
// if last kill was from player then a life should've been gained, so animate lives
pub fn draw_lives(player_lives: &i32, texture_life: Texture2D, texture_ground_bg: &Texture2D, game_manager: &WaveManager) {
    let lives_padding = 2f32;
    let last_kill_from_player = game_manager.last_enemy_death_reason == LastEnemyDeathReason::Player; 
    let wave_speed = 20f32;
    let wave_offset_y = -7f32;
    let wave_time_offset = 0.7f32;

    match &game_manager.state {
         WaveManagerState::Spawning(_spawning_state) if last_kill_from_player => {
            for i in 0..*player_lives {
                let wave = ((get_time() as f32*wave_speed + i as f32*wave_time_offset).sin()+1f32)*0.5f32;
                draw_texture_ex(
                    texture_life,
                    5f32 + i as f32 * (texture_life.width() + lives_padding),
                    GAME_SIZE_Y as f32 - texture_ground_bg.height() + 3f32 + wave * wave_offset_y,
                    PINK,
                    DrawTextureParams {
                        ..Default::default()
                    },
                );
            }
        }
        _ => {
            for i in 0..*player_lives {
                draw_texture_ex(
                    texture_life,
                    5f32 + i as f32 * (texture_life.width() + lives_padding),
                    GAME_SIZE_Y as f32 - texture_ground_bg.height() + 3f32,
                    WHITE,
                    DrawTextureParams {
                        ..Default::default()
                    },
                );
            }
        }
    }
}

pub enum GameStateCommand {
    ChangeState(GameStateIdentifier),
}

#[derive(PartialEq, Eq, Hash)]
pub enum GameStateIdentifier {
    Menu,
    Game,
}

pub trait GameState {
    fn update(&mut self, dt: f32) -> Option<GameStateCommand>;
    fn draw(&self);
    fn on_enter(&mut self);
}


pub struct GameStateGame {
}

impl GameState for GameStateGame {
    fn update(&mut self, dt: f32) -> Option<GameStateCommand> {
        None
    }

    fn draw(&self) {
    }

    fn on_enter(&mut self) {
    }
}

pub struct GameStateMenu {

}

impl GameState for GameStateMenu {
    fn update(&mut self, dt: f32) -> Option<GameStateCommand> {
        None
    }

    fn draw(&self) {
    }

    fn on_enter(&mut self) {
    }
}

pub struct GameManager {
    states: HashMap::<GameStateIdentifier, Box::<dyn GameState>>,
    current_state_identifier: GameStateIdentifier,
}

impl GameManager {
    pub fn new(all_states: Vec::<(GameStateIdentifier, Box::<dyn GameState>)>) -> Self {
        let mut states = HashMap::new(); 
        for state in all_states.into_iter() {
            states.insert(state.0, state.1);
        }
        GameManager {
            states,
            current_state_identifier: GameStateIdentifier::Menu
        }
    }

    pub fn update(&mut self, dt: f32) {
        // since we access the state through identifier instead of reference 
        // we try to get the state, then update it. If we ChangeState, then we can't call on_enter IN this scope,
        // because we would have 2 state references, the current one and the one we change to.
        // (we can't set state if we are holding a reference to the current state)
        let state_command_optional = if let Some(game_state) = self.states.get_mut(&self.current_state_identifier) {
            game_state.update(dt)
        } else {
            None
        };

        if let Some(state_command) = state_command_optional {
            match state_command {
                GameStateCommand::ChangeState(next_state) => {
                    self.current_state_identifier = next_state;
                    if let Some(game_state) = self.states.get_mut(&self.current_state_identifier) {
                        game_state.on_enter();
                    }
                },
            }
        }
    }
    pub fn draw(&self) {
        if let Some(game_state) = self.states.get(&self.current_state_identifier) {
            game_state.draw();
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let game_render_target = render_target(GAME_SIZE_X as u32, GAME_SIZE_Y as u32);
    let texture_player: Texture2D = load_texture("resources/player.png").await;
    let texture_player_explotion: Texture2D = load_texture("resources/player_explotion.png").await;
    let texture_player_missile: Texture2D = load_texture("resources/player_missile.png").await;
    let texture_demon_missile: Texture2D = load_texture("resources/demon_missile.png").await;
    let texture_ground_bg: Texture2D = load_texture("resources/ground_bg.png").await;
    let texture_life: Texture2D = load_texture("resources/life.png").await;

    // set all textures filter mode to nearest
    for texture in [texture_player, texture_player_explotion
        , texture_player_missile, texture_demon_missile
        , texture_ground_bg, texture_life, game_render_target.texture].iter()
    {
        set_texture_filter(*texture, FilterMode::Nearest);
    }

    let font = load_ttf_font("resources/Kenney Pixel Square.ttf").await;
    let mut player_score: i32 = 0;
    let mut player_lives: i32 = PLAYER_LIVES_START;

    let mut textures = Textures::new(texture_demon_missile, texture_player_missile);
    {
        use EnemyColor::{Green, Purple, Red};
        use EnemyType::{Mini, Normal};
        textures.load_texture("resources/demon_mini_green_1.png", Green, Mini).await;
        textures.load_texture("resources/demon_mini_red_1.png", Red, Mini).await;
        textures.load_texture("resources/demon_mini_purple_1.png", Purple, Mini).await;
        textures.load_texture("resources/demon_normal_green_1.png", Green, Normal).await;
        textures.load_texture("resources/demon_normal_green_2.png", Green, Normal).await;
        textures.load_texture("resources/demon_normal_purple_1.png", Purple, Normal).await;
        textures.load_texture("resources/demon_normal_purple_2.png", Purple, Normal).await;
        textures.load_texture("resources/demon_normal_red_1.png", Red, Normal).await;
    }

    let mut bullets = Vec::<Bullet>::new();
    let mut enemies = Vec::<Enemy>::new();

    let player_spawn_y = GAME_SIZE_Y as f32 - texture_ground_bg.height() - texture_player.height();
    let mut player = Player::new(vec2(GAME_CENTER_X, player_spawn_y), texture_player, texture_player_missile, texture_player_explotion);

    let mut wave_manager = WaveManager {
        state: WaveManagerState::Spawning(WaveManagerStateSpawning{spawn_timer: 0f32, enemies_left: 15, }),
        last_enemy_death_reason: LastEnemyDeathReason::Environment,
    };

    let game_states: Vec::<(GameStateIdentifier, Box::<dyn GameState>)> = vec![
        (GameStateIdentifier::Menu, Box::new(GameStateMenu{})),
        (GameStateIdentifier::Game, Box::new(GameStateGame{})),
    ];
    let mut game_manager = GameManager::new(game_states);

    loop {
        let dt = get_frame_time();

        set_camera(Camera2D {
            // I have no idea why the zoom is this way lmao
            zoom: vec2(1./GAME_SIZE_X as f32*2., 1./GAME_SIZE_Y as f32*2.),
            target: vec2((GAME_SIZE_X as f32*0.5f32).floor(), (GAME_SIZE_Y as f32 * 0.5f32).floor()),
            render_target: Some(game_render_target),
            ..Default::default()
        });
        clear_background(BLACK);
        game_manager.update(dt);
        game_manager.draw();

        let manager_message_optional = wave_manager.update(dt, &mut enemies, &textures);
        if let Some(manager_message) = manager_message_optional {
            match manager_message {
                WaveManagerMessage::LevelCleared => {
                    player_lives +=1;
                    player_lives = player_lives.min(PLAYER_LIVES_MAX);
                    let score_add = match wave_manager.last_enemy_death_reason {
                        LastEnemyDeathReason::Environment => SCORE_SURVIVED_ALL,
                        LastEnemyDeathReason::Player => SCORE_KILL_ALL,
                    };
                    player_score += score_add;
                }
            }
        }

        for enemy in enemies.iter_mut() {
            enemy.update(dt, &mut bullets, &textures, &player.pos, &mut wave_manager);
            enemy.draw();
        }

        for bullet in bullets.iter_mut() {
            bullet.update(dt);
            bullet.draw();
        }

        // bullets hurting player
        for (i, bullet) in bullets.iter_mut().filter(|b| b.hurt_type == BulletHurtType::Player).enumerate() {
            if bullet.overlaps(&player.collision_rect) {
                if player.state != PlayerState::Normal {
                    continue;
                }
                player_lives -= 1;
                // CHANGE PLAYER STATE
                player.process_command_optional(Some(PlayerCommand::ChangeState(PlayerState::Invisible(PLAYER_TIME_INVISBLE))));
                if player_lives <= 0 {
                    debug!("IMPLEMENT LOSE STATE!");
                }
                bullet.is_kill = true;
                break;
            }
        }

        // homing enemies hurting player
        for enemy in enemies.iter_mut()
            // filter enemies containing homing state, variant_eq is used so we can disregard homing data
            .filter(|e| variant_eq(&e.state, &EnemyState::Homing(EnemyStateHoming{})))
        {
            if enemy.overlaps(&player.collision_rect) {
                let player_invisible = variant_eq(&player.state, &PlayerState::Invisible(0f32));
                if !player_invisible {
                    player_lives -= 1;
                    player.process_command_optional(Some(PlayerCommand::ChangeState(PlayerState::Invisible(PLAYER_TIME_INVISBLE))));
                    enemy.state_shared.health = 0;
                }
            }
        }

        // todo explain
        let mut death_methods = Vec::<(Vec2, EnemyDeathMethod, EnemyType, EnemyColor)>::with_capacity(4);

        // bullets hurting enemies
        for (i, bullet) in bullets.iter_mut()
            .filter(|b| b.hurt_type == BulletHurtType::Enemy)
            .enumerate()
        {
            for (i, enemy) in enemies.iter_mut().enumerate() {
                if enemy.overlaps(&bullet.collision_rect) && !bullet.is_kill {
                    enemy.state_shared.health -= 1;
                    wave_manager.last_enemy_death_reason = LastEnemyDeathReason::Player;
                    // death
                    if enemy.state_shared.health <= 0 {
                        death_methods.push((enemy.state_shared.pos, enemy.state_shared.death_method, enemy.state_shared.enemy_type, enemy.state_shared.enemy_color));
                    }
                    // can only hurt one enemy, flag for deletion
                    bullet.is_kill = true;
                }
            }
        }

        for (pos, death_method, enemy_type, enemy_color) in death_methods.iter() {
            let score_add = match enemy_type {
                EnemyType::Normal => SCORE_NORMAL, 
                EnemyType::Mini => SCORE_MINI,
            };
            player_score += score_add;
            match death_method {
                EnemyDeathMethod::None => {
                },
                EnemyDeathMethod::SpawnChildren(amount) => {
                    let spawn_width = 20f32;
                    let step = 1./(*amount as f32);
                    for i in 0..*amount {
                        let spawn_pos = *pos + vec2(step * spawn_width * i as f32, 0f32);
                        spawn_enemy(&mut enemies, &textures, SpawnBlueprint::Mini(spawn_pos), *enemy_color);
                    }
                },
            }
        }

        // remove bullets that hit something
        bullets.retain(|e| !e.is_kill);
        // remove dead enemies
        enemies.retain(|e| e.state_shared.health > 0);

        draw_texture_ex(
            texture_ground_bg,
            0f32,
            GAME_SIZE_Y as f32 - texture_ground_bg.height(),
            WHITE,
            DrawTextureParams {
                //dest_size: Some(vec2(screen_width(), screen_height())),
                dest_size: Some(Vec2::new(GAME_SIZE_X as f32,  texture_ground_bg.height())),
                ..Default::default()
            },
        );

        draw_lives(&player_lives, texture_life, &texture_ground_bg, &wave_manager);

        player.update(dt, &mut bullets, &textures);
        player.draw();


        set_default_camera();

        // calculate game view size based on window size
        let game_diff_w = screen_width() / GAME_SIZE_X as f32;
        let game_diff_h = screen_height() / GAME_SIZE_Y as f32;
        let aspect_diff = game_diff_w.min(game_diff_h);

        let scaled_game_size_w = GAME_SIZE_X as f32 * aspect_diff;
        let scaled_game_size_h = GAME_SIZE_Y as f32 * aspect_diff;

        let width_padding = (screen_width() - scaled_game_size_w) * 0.5f32;
        let height_padding = (screen_height() - scaled_game_size_h) * 0.5f32;

        // draw game
        clear_background(BLACK);

        // fit inside window
        draw_texture_ex(
            game_render_target.texture,
            width_padding,
            height_padding,
            WHITE,
            DrawTextureParams {
                //dest_size: Some(vec2(screen_width(), screen_height())),
                dest_size: Some(Vec2::new(scaled_game_size_w, scaled_game_size_h)),
                ..Default::default()
            },
        );

        let score_text = format!("{}", player_score); 
        let font_size = (aspect_diff * 10f32) as u16;
        let mut text_x = width_padding + scaled_game_size_w * 0.5f32;
        text_x -= score_text.len() as f32 * 0.5f32 * font_size as f32 *0.6f32;
        draw_text_ex(score_text.as_ref(), text_x, height_padding + font_size as f32 * 2f32
            , TextParams{
                font,
                font_size,
                font_scale: 1f32,
                color: YELLOW,
            }
        );

        next_frame().await
    }
}