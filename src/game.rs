use macroquad::prelude::*;
use std::collections::HashMap;

use crate::{
    bullet::{Bullet, BulletHurtType},
    constants::*,
    enemy::{Enemy, EnemyDeathMethod, EnemyStateHoming},
    enemy::{EnemyColor, EnemyState, EnemyType},
    player::{Player, PlayerCommand, PlayerState},
    resources::{Resources, SoundIdentifier},
    variant_eq,
    wave::{
        spawn_enemy, LastEnemyDeathReason, SpawnBlueprint, WaveManager, WaveManagerMessage,
        WaveManagerState,
    },
};

use quad_snd::mixer::{SoundMixer, Volume};

// okay this is pretty hacky...
// if last kill was from player then a life should've been gained, so animate lives
pub fn draw_lives(
    player_lives: &i32,
    texture_life: Texture2D,
    texture_ground_bg: &Texture2D,
    game_manager: &WaveManager,
) {
    let lives_padding = 2f32;
    let last_kill_from_player =
        game_manager.last_enemy_death_reason == LastEnemyDeathReason::Player;
    let wave_speed = 20f32;
    let wave_offset_y = -7f32;
    let wave_time_offset = 0.7f32;

    match &game_manager.state {
        WaveManagerState::Spawning(_spawning_state) if last_kill_from_player => {
            for i in 0..*player_lives {
                let wave = ((get_time() as f32 * wave_speed + i as f32 * wave_time_offset).sin()
                    + 1f32)
                    * 0.5f32;
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

pub struct MenuPayload {
    score: i32,
}

// hacky way to transfer data between states
pub enum ChangeStatePayload {
    MenuPayload(MenuPayload),
}

pub enum GameStateCommand {
    ChangeState(GameStateIdentifier, Option<ChangeStatePayload>),
}

#[derive(PartialEq, Eq, Hash)]
pub enum GameStateIdentifier {
    Menu,
    Game,
}

pub trait GameState {
    fn update(
        &mut self,
        dt: f32,
        resources: &Resources,
        sound_mixer: &mut SoundMixer,
    ) -> Option<GameStateCommand>;
    fn draw(&self, resources: &Resources);
    fn draw_unscaled(&self, resources: &Resources);
    fn on_enter(&mut self, resources: &Resources, payload_optional: Option<ChangeStatePayload>);
}

pub struct GameStateGame {
    wave_manager: WaveManager,
    player_score: i32,
    player_lives: i32,
    bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    player: Player,
}

impl GameStateGame {
    pub fn new(resources: &Resources) -> Self {
        let player_spawn_y =
            GAME_SIZE_Y as f32 - resources.ground_bg.height() - resources.player.height();
        let player_pos = vec2(GAME_CENTER_X, player_spawn_y);
        let player = Player::new(
            player_pos,
            resources.player,
            resources.player_missile,
            resources.player_explotion,
        );

        GameStateGame {
            wave_manager: WaveManager::new(),
            player_score: 0,
            player_lives: PLAYER_LIVES_START,
            bullets: Vec::<Bullet>::new(),
            enemies: Vec::<Enemy>::new(),
            player,
        }
    }
}

impl GameState for GameStateGame {
    fn on_enter(&mut self, resources: &Resources, _payload_optional: Option<ChangeStatePayload>) {
        self.wave_manager.reset();
        self.player.reset(resources);
        self.player_score = 0;
        self.player_lives = PLAYER_LIVES_START;
        self.enemies.clear();
        self.bullets.clear();
    }

    fn update(
        &mut self,
        dt: f32,
        resources: &Resources,
        sound_mixer: &mut SoundMixer,
    ) -> Option<GameStateCommand> {
        let manager_message_optional =
            self.wave_manager
                .update(dt, &mut self.enemies, resources, sound_mixer);
        if let Some(manager_message) = manager_message_optional {
            match manager_message {
                WaveManagerMessage::LevelCleared => {
                    self.player_lives += 1;
                    self.player_lives = self.player_lives.min(PLAYER_LIVES_MAX);
                    let score_add = match self.wave_manager.last_enemy_death_reason {
                        LastEnemyDeathReason::Environment => SCORE_SURVIVED_ALL,
                        LastEnemyDeathReason::Player => SCORE_KILL_ALL,
                    };
                    resources.play_sound(SoundIdentifier::WaveCleared, sound_mixer, Volume(0.6f32));
                    self.player_score += score_add;
                }
            }
        }

        for enemy in self.enemies.iter_mut() {
            enemy.update(
                dt,
                &mut self.bullets,
                resources,
                &self.player.pos,
                &mut self.wave_manager,
                sound_mixer,
            );
            enemy.draw();
        }

        for bullet in self.bullets.iter_mut() {
            bullet.update(dt);
            bullet.draw();
        }

        // bullets hurting player
        for bullet in self
            .bullets
            .iter_mut()
            .filter(|b| b.hurt_type == BulletHurtType::Player)
        {
            if bullet.overlaps(&self.player.collision_rect) {
                if self.player.state != PlayerState::Normal {
                    continue;
                }
                self.player_lives -= 1;
                resources.play_sound(SoundIdentifier::PlayerOuch, sound_mixer, Volume(1.0f32));
                // CHANGE PLAYER STATE
                self.player
                    .process_command_optional(Some(PlayerCommand::ChangeState(
                        PlayerState::Invisible(PLAYER_TIME_INVISBLE),
                    )));
                if self.player_lives <= 0 {
                    return Some(GameStateCommand::ChangeState(
                        GameStateIdentifier::Menu,
                        Some(ChangeStatePayload::MenuPayload(MenuPayload {
                            score: self.player_score,
                        })),
                    ));
                }
                bullet.is_kill = true;
                break;
            }
        }

        // homing enemies hurting player
        for enemy in self
            .enemies
            .iter_mut()
            // filter enemies containing homing state, variant_eq is used so we can disregard homing data
            .filter(|e| variant_eq(&e.state, &EnemyState::Homing(EnemyStateHoming {})))
        {
            if enemy.overlaps(&self.player.collision_rect) {
                let player_invisible =
                    variant_eq(&self.player.state, &PlayerState::Invisible(0f32));
                if !player_invisible {
                    self.player_lives -= 1;
                    resources.play_sound(SoundIdentifier::PlayerOuch, sound_mixer, Volume(1.0f32));
                    self.player
                        .process_command_optional(Some(PlayerCommand::ChangeState(
                            PlayerState::Invisible(PLAYER_TIME_INVISBLE),
                        )));
                    enemy.state_shared.health = 0;
                }
            }
        }

        // todo explain
        let mut death_methods =
            Vec::<(Vec2, EnemyDeathMethod, EnemyType, EnemyColor)>::with_capacity(4);

        // bullets hurting enemies
        for bullet in self
            .bullets
            .iter_mut()
            .filter(|b| b.hurt_type == BulletHurtType::Enemy)
        {
            for enemy in self.enemies.iter_mut() {
                if enemy.overlaps(&bullet.collision_rect) && !bullet.is_kill {
                    enemy.state_shared.health -= 1;
                    self.wave_manager.last_enemy_death_reason = LastEnemyDeathReason::Player;
                    // death
                    if enemy.state_shared.health <= 0 {
                        resources.play_sound(
                            SoundIdentifier::EnemyOuch,
                            sound_mixer,
                            Volume(1.0f32),
                        );
                        death_methods.push((
                            enemy.state_shared.pos,
                            enemy.state_shared.death_method,
                            enemy.state_shared.enemy_type,
                            enemy.state_shared.enemy_color,
                        ));
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
            self.player_score += score_add;
            match death_method {
                EnemyDeathMethod::None => {}
                EnemyDeathMethod::SpawnChildren(amount) => {
                    resources.play_sound(SoundIdentifier::SpawnMini, sound_mixer, Volume(1.0f32));
                    let spawn_width = 20f32;
                    let step = 1. / (*amount as f32);
                    for i in 0..*amount {
                        let spawn_pos = *pos + vec2(step * spawn_width * i as f32, 0f32);
                        spawn_enemy(
                            &mut self.enemies,
                            resources,
                            SpawnBlueprint::Mini(spawn_pos),
                            *enemy_color,
                        );
                    }
                }
            }
        }

        // remove bullets that hit something
        self.bullets.retain(|e| !e.is_kill);
        // remove dead enemies
        self.enemies.retain(|e| e.state_shared.health > 0);

        draw_texture_ex(
            resources.ground_bg,
            0f32,
            GAME_SIZE_Y as f32 - resources.ground_bg.height(),
            WHITE,
            DrawTextureParams {
                //dest_size: Some(vec2(screen_width(), screen_height())),
                dest_size: Some(Vec2::new(GAME_SIZE_X as f32, resources.ground_bg.height())),
                ..Default::default()
            },
        );

        draw_lives(
            &self.player_lives,
            resources.life,
            &resources.ground_bg,
            &self.wave_manager,
        );

        self.player
            .update(dt, &mut self.bullets, resources, sound_mixer);
        self.player.draw();
        None
    }

    fn draw(&self, _resources: &Resources) {}

    fn draw_unscaled(&self, resources: &Resources) {
        let game_diff_w = screen_width() / GAME_SIZE_X as f32;
        let game_diff_h = screen_height() / GAME_SIZE_Y as f32;
        let aspect_diff = game_diff_w.min(game_diff_h);

        let scaled_game_size_w = GAME_SIZE_X as f32 * aspect_diff;
        let scaled_game_size_h = GAME_SIZE_Y as f32 * aspect_diff;

        let width_padding = (screen_width() - scaled_game_size_w) * 0.5f32;
        let height_padding = (screen_height() - scaled_game_size_h) * 0.5f32;

        let score_text = format!("{}", self.player_score);
        let font_size = (aspect_diff * 10f32) as u16;
        let mut text_x = width_padding + scaled_game_size_w * 0.5f32;
        text_x -= score_text.len() as f32 * 0.5f32 * font_size as f32 * 0.6f32;
        draw_text_ex(
            score_text.as_ref(),
            text_x,
            height_padding + font_size as f32 * 2f32,
            TextParams {
                font: resources.font,
                font_size,
                font_scale: 1f32,
                color: YELLOW,
                font_scale_aspect: 1f32,
            },
        );
    }
}

pub struct GameStateMenu {
    last_score_optional: Option<i32>,
}

impl GameStateMenu {
    pub fn new() -> Self {
        GameStateMenu {
            last_score_optional: None,
        }
    }
}

impl GameState for GameStateMenu {
    fn update(
        &mut self,
        _dt: f32,
        _resources: &Resources,
        _sound_mixer: &mut SoundMixer,
    ) -> Option<GameStateCommand> {
        if is_key_pressed(KEY_START_GAME) {
            return Some(GameStateCommand::ChangeState(
                GameStateIdentifier::Game,
                None,
            ));
        }
        None
    }

    fn draw(&self, resources: &Resources) {
        draw_texture_ex(
            resources.ground_bg,
            0f32,
            GAME_SIZE_Y as f32 - resources.ground_bg.height(),
            WHITE,
            DrawTextureParams {
                //dest_size: Some(vec2(screen_width(), screen_height())),
                dest_size: Some(Vec2::new(GAME_SIZE_X as f32, resources.ground_bg.height())),
                ..Default::default()
            },
        );
    }

    fn on_enter(&mut self, _resources: &Resources, payload_optional: Option<ChangeStatePayload>) {
        if let Some(payload) = payload_optional {
            match payload {
                ChangeStatePayload::MenuPayload(menu_payload) => {
                    self.last_score_optional = Some(menu_payload.score)
                }
            }
        }
    }

    fn draw_unscaled(&self, resources: &Resources) {
        let game_diff_w = screen_width() / GAME_SIZE_X as f32;
        let game_diff_h = screen_height() / GAME_SIZE_Y as f32;
        let aspect_diff = game_diff_w.min(game_diff_h);

        let scaled_game_size_w = GAME_SIZE_X as f32 * aspect_diff;
        let scaled_game_size_h = GAME_SIZE_Y as f32 * aspect_diff;

        let width_padding = (screen_width() - scaled_game_size_w) * 0.5f32;
        let height_padding = (screen_height() - scaled_game_size_h) * 0.5f32;

        let font_size = (aspect_diff * 10f32) as u16;

        if let Some(last_score) = self.last_score_optional {
            let score_text = format!("{}", last_score);
            let mut text_x = width_padding + scaled_game_size_w * 0.5f32;
            text_x -= score_text.len() as f32 * 0.5f32 * font_size as f32 * 0.6f32;
            draw_text_ex(
                score_text.as_ref(),
                text_x,
                height_padding + font_size as f32 * 2f32,
                TextParams {
                    font: resources.font,
                    font_size,
                    font_scale: 1f32,
                    color: YELLOW,
                    font_scale_aspect: 1f32,
                },
            );
        }
        let start_text = "TAP SPACE TO START";
        let mut text_x = width_padding + scaled_game_size_w * 0.5f32;
        text_x -= start_text.len() as f32 * 0.5f32 * font_size as f32 * 0.6f32;

        draw_text_ex(
            start_text,
            text_x,
            screen_height() * 0.5f32,
            TextParams {
                font: resources.font,
                font_size,
                font_scale: 1f32,
                color: YELLOW,
                font_scale_aspect: 1f32,
            },
        );
    }
}

pub struct GameManager {
    states: HashMap<GameStateIdentifier, Box<dyn GameState>>,
    current_state_identifier: GameStateIdentifier,
    resources: Resources,
    sound_mixer: SoundMixer,
}

impl GameManager {
    pub fn new(
        all_states: Vec<(GameStateIdentifier, Box<dyn GameState>)>,
        resources: Resources,
        sound_mixer: SoundMixer,
    ) -> Self {
        let mut states = HashMap::new();
        for state in all_states.into_iter() {
            states.insert(state.0, state.1);
        }
        GameManager {
            states,
            current_state_identifier: GameStateIdentifier::Menu,
            resources,
            sound_mixer,
        }
    }

    pub fn frame_sounds(&mut self) {
        self.sound_mixer.frame();
    }

    pub fn update(&mut self, dt: f32) {
        // since we access the state through identifier instead of reference
        // we try to get the state, then update it. If we ChangeState, then we can't call on_enter IN this scope,
        // because we would have 2 state references, the current one and the one we change to.
        // (we can't set state if we are holding a reference to the current state)
        let state_command_optional =
            if let Some(game_state) = self.states.get_mut(&self.current_state_identifier) {
                game_state.update(dt, &self.resources, &mut self.sound_mixer)
            } else {
                None
            };

        if let Some(state_command) = state_command_optional {
            match state_command {
                GameStateCommand::ChangeState(next_state, payload_optional) => {
                    self.current_state_identifier = next_state;
                    if let Some(game_state) = self.states.get_mut(&self.current_state_identifier) {
                        game_state.on_enter(&self.resources, payload_optional);
                    }
                }
            }
        }
    }

    pub fn draw(&self) {
        if let Some(game_state) = self.states.get(&self.current_state_identifier) {
            game_state.draw(&self.resources);
        }
    }

    pub fn draw_unscaled(&self) {
        if let Some(game_state) = self.states.get(&self.current_state_identifier) {
            game_state.draw_unscaled(&self.resources);
        }
    }
}
