use macroquad::prelude::*;
use quad_snd::mixer::{SoundMixer, Volume};

use crate::{
    bullet::{Bullet, BulletHurtType},
    constants::*,
    resources::{Resources, SoundIdentifier},
    wave::{LastEnemyDeathReason, WaveManager},
};

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
    pub pos: Vec2,
    angle: f32,
    angle_speed: f32,
    collision_rect: Rect,
    pub health: i32,
    pub death_method: EnemyDeathMethod,
    animation_timer: f32,
    pub enemy_type: EnemyType,
    pub enemy_color: EnemyColor,

    // used for mini enemies, that home in on player
    charge_timer_optional: Option<f32>,
}

#[derive(PartialEq)]
pub struct EnemyStateNormal {
    shoot_timer: f32,
}

#[derive(PartialEq)]
pub struct EnemyStateHoming {}

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
    pub state_shared: EnemyStateShared,
    pub state: EnemyState,
}

impl Enemy {
    pub fn new(
        pos: Vec2,
        texture: Texture2D,
        health: i32,
        death_method: EnemyDeathMethod,
        enemy_type: EnemyType,
        enemy_color: EnemyColor,
    ) -> Self {
        let charge_timer_optional = match enemy_type {
            EnemyType::Normal => None,
            EnemyType::Mini => Some(rand::gen_range(
                ENEMY_MINI_HOMING_TIME_RANGE.x,
                ENEMY_MINI_HOMING_TIME_RANGE.y,
            )),
        };
        Enemy {
            state_shared: EnemyStateShared {
                pos,
                texture,
                collision_rect: Rect::new(0f32, 0f32, texture.width(), texture.height()),
                health,
                angle: 0f32,
                angle_speed: rand::gen_range(ENEMY_ANGLE_SPEED_RANGE.x, ENEMY_ANGLE_SPEED_RANGE.y),
                death_method,
                animation_timer: 0f32,
                enemy_type,
                charge_timer_optional,
                enemy_color,
            },
            state: EnemyState::Spawning(EnemyStateSpawning { spawn_timer: 0f32 }),
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        bullets: &mut Vec<Bullet>,
        resources: &Resources,
        player_pos: &Vec2,
        game_manager: &mut WaveManager,
        sound_mixer: &mut SoundMixer,
    ) {
        let command_optional = match &mut self.state {
            EnemyState::Spawning(state_data) => {
                Self::update_state_spawning(&mut self.state_shared, dt, state_data)
            }
            EnemyState::Normal(state_data) => {
                Self::update_state_normal(&mut self.state_shared, dt, state_data)
            }
            EnemyState::Shooting(state_data) => Self::update_state_shooting(
                &mut self.state_shared,
                dt,
                bullets,
                resources,
                state_data,
                sound_mixer,
            ),
            EnemyState::Homing(_state_data) => Self::update_state_homing(
                &mut self.state_shared,
                dt,
                player_pos,
                game_manager,
                sound_mixer,
                resources,
            ),
        };
        match command_optional {
            None => {}
            Some(command) => match command {
                EnemyCommand::ChangeState(new_state) => {
                    self.state = new_state;
                }
            },
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

    fn update_state_spawning(
        state_shared: &mut EnemyStateShared,
        dt: f32,
        state_data: &mut EnemyStateSpawning,
    ) -> Option<EnemyCommand> {
        state_data.spawn_timer += dt;
        // different enemy types spawn differently
        let end_time = match state_shared.enemy_type {
            EnemyType::Normal => ENEMY_ANIM_TIME_SPAWN,
            EnemyType::Mini => ENEMY_MINI_ANIM_TIME_SPAWN,
        };

        let fraction = state_data.spawn_timer / end_time;
        if fraction >= 1.0f32 {
            return Some(EnemyCommand::ChangeState(EnemyState::Normal(
                EnemyStateNormal { shoot_timer: 0f32 },
            )));
        }
        None
    }

    fn update_state_normal(
        state_shared: &mut EnemyStateShared,
        dt: f32,
        state_data: &mut EnemyStateNormal,
    ) -> Option<EnemyCommand> {
        let angle_change_speed = std::f32::consts::PI * state_shared.angle_speed;
        state_shared.angle +=
            (get_time() as f32 * angle_change_speed).sin() * std::f32::consts::PI * 2f32 * dt;
        let dir = vec2(state_shared.angle.sin(), -state_shared.angle.cos());
        state_shared.pos += dir * ENEMY_SPEED * dt;
        // state_shared.pos.x += rand::gen_range(-1f32, 1f32) * ENEMY_SPEED * dt;
        // state_shared.pos.y += rand::gen_range(-1f32, 1f32) * ENEMY_SPEED * dt;
        Self::clamp_in_view(&mut state_shared.pos);
        state_shared.collision_rect.x = state_shared.pos.x - state_shared.texture.width() * 0.5f32;
        state_shared.collision_rect.y = state_shared.pos.y;

        state_data.shoot_timer += dt;
        // update timer for charging in on player
        if let Some(charge_timer) = &mut state_shared.charge_timer_optional {
            *charge_timer -= dt;
            if *charge_timer <= 0f32 {
                return Some(EnemyCommand::ChangeState(EnemyState::Homing(
                    EnemyStateHoming {},
                )));
            }
        }

        state_shared.animation_timer += dt;
        if state_shared.animation_timer > ENEMY_ANIM_TIME_FLAP * 4f32 {
            state_shared.animation_timer -= ENEMY_ANIM_TIME_FLAP * 4f32;
        }
        if state_data.shoot_timer > ENEMY_SHOOT_TIME {
            let shot_count = rand::gen_range(1, ENEMY_MAX_BURST_COUNT);
            // every time we change state, the enemy will chose a random speed at which it changes its velocity
            state_shared.angle_speed =
                rand::gen_range(ENEMY_ANGLE_SPEED_RANGE.x, ENEMY_ANGLE_SPEED_RANGE.y);
            state_shared.angle = rand::gen_range(-std::f32::consts::PI, std::f32::consts::PI);
            return Some(EnemyCommand::ChangeState(EnemyState::Shooting(
                EnemyStateShooting {
                    shoot_timer: ENEMY_SHOOT_BURST_TIME,
                    shots_left: shot_count,
                },
            )));
        }
        None
    }

    fn update_state_shooting(
        state_shared: &mut EnemyStateShared,
        dt: f32,
        bullets: &mut Vec<Bullet>,
        resources: &Resources,
        state_data: &mut EnemyStateShooting,
        sound_mixer: &mut SoundMixer,
    ) -> Option<EnemyCommand> {
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
                let spawn_offset = vec2((state_shared.texture.width() / 4f32) * 0.5f32, 0f32);
                bullets.push(Bullet::new(
                    state_shared.pos + spawn_offset,
                    BulletHurtType::Player,
                    resources,
                ));
                bullets.push(Bullet::new(
                    state_shared.pos - spawn_offset,
                    BulletHurtType::Player,
                    resources,
                ));
            } else {
                let spawn_offset = vec2(0f32, -3f32);
                bullets.push(Bullet::new(
                    state_shared.pos + spawn_offset,
                    BulletHurtType::Player,
                    resources,
                ));
            }
            resources.play_sound(SoundIdentifier::EnemyShoot, sound_mixer, Volume(1.0f32));

            // for fun move enemy up when shooting
            state_shared.pos.y -= 2f32;
        }

        state_shared.collision_rect.x = state_shared.pos.x - state_shared.texture.width() * 0.5f32;
        state_shared.collision_rect.y = state_shared.pos.y;

        state_shared.animation_timer += dt;
        if state_shared.animation_timer > ENEMY_ANIM_TIME_FLAP * 4f32 {
            state_shared.animation_timer -= ENEMY_ANIM_TIME_FLAP * 4f32;
        }

        // done shooting
        if state_data.shots_left <= 0 {
            return Some(EnemyCommand::ChangeState(EnemyState::Normal(
                EnemyStateNormal { shoot_timer: 0f32 },
            )));
        }
        None
    }

    fn update_state_homing(
        state_shared: &mut EnemyStateShared,
        dt: f32,
        player_pos: &Vec2,
        game_manager: &mut WaveManager,
        sound_mixer: &mut SoundMixer,
        resources: &Resources,
    ) -> Option<EnemyCommand> {
        state_shared.animation_timer += dt;
        if state_shared.animation_timer > ENEMY_ANIM_TIME_FLAP * 4f32 {
            state_shared.animation_timer -= ENEMY_ANIM_TIME_FLAP * 4f32;
            resources.play_sound(SoundIdentifier::Warning, sound_mixer, Volume(1.0f32));
        }
        // MOVE TOWARDS PLAYER
        let player_dx = player_pos.x - state_shared.pos.x;
        let dx = if player_dx > 0f32 { 1f32 } else { -1f32 };
        let sway_speed = 20f32;
        let sway = (get_time() as f32 * sway_speed).sin();
        // remap from -1 -> 1 TO 0 -> 1
        let sway = (sway + 1f32) * 0.5f32;

        let vel = vec2(dx * ENEMY_SPEED_HOMING.x * sway, ENEMY_SPEED_HOMING.y);
        state_shared.pos += vel * dt;
        state_shared.collision_rect.x = state_shared.pos.x - state_shared.texture.width() * 0.5f32;
        state_shared.collision_rect.y = state_shared.pos.y;

        // kill monsters below screen
        if state_shared.pos.y > GAME_SIZE_Y as f32 {
            state_shared.health = 0;
            game_manager.last_enemy_death_reason = LastEnemyDeathReason::Environment;
        }
        // collision against player checks happends in main
        None
    }

    fn draw_state_spawning_normal(
        state_shared: &EnemyStateShared,
        state_data: &EnemyStateSpawning,
    ) {
        let rand_frame = rand::gen_range(0i32, 2i32);
        let fraction = 1.0f32 - state_data.spawn_timer / ENEMY_ANIM_TIME_SPAWN;
        let offset = fraction * ENEMY_ANIM_DISTANCE;
        let sprite_width = state_shared.texture.width() / 3f32;
        let scale = sprite_width + fraction * ENEMY_ANIM_SPAWN_SCALE * sprite_width;
        // Left wing
        draw_texture_ex(
            state_shared.texture,
            state_shared.pos.x - ((state_shared.texture.width() / 3.0f32) * 1.0f32) - offset,
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
        let sprite_width = state_shared.texture.width() / 4f32;
        let scale = sprite_width * 0.5f32 + fraction * 1.5f32 * sprite_width;
        draw_texture_ex(
            state_shared.texture,
            state_shared.pos.x - ((state_shared.texture.width() / 4.0f32) * 1.0f32),
            state_shared.pos.y,
            WHITE,
            DrawTextureParams {
                rotation: fraction * std::f32::consts::PI * 2f32,
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
                rotation: fraction * std::f32::consts::PI * 2f32,
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
        let rand_frame = (self.state_shared.animation_timer / ENEMY_ANIM_TIME_FLAP).floor();
        // Left wing
        draw_texture_ex(
            self.state_shared.texture,
            self.state_shared.pos.x - ((self.state_shared.texture.width() / 4.0f32) * 1.0f32),
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
            EnemyState::Spawning(state_data) => {
                Self::draw_state_spawning(&self.state_shared, state_data)
            }
            EnemyState::Normal(_state_data) => self.draw_state_normal(),
            // enemy doesn't look different when shooting
            EnemyState::Shooting(_state_data) => self.draw_state_normal(),
            EnemyState::Homing(_state_data) => self.draw_state_normal(),
        }
    }
}
