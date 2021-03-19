use bullet::{Bullet, BulletHurtType};
use constants::*;
use enemy::{Enemy, EnemyColor, EnemyDeathMethod, EnemyState, EnemyStateHoming, EnemyType};
use macroquad::prelude::*;
use resources::{load_resources, Resources, SoundIdentifier};
use std::collections::HashMap;

use quad_snd::mixer::{SoundMixer, Volume};

mod bullet;
mod constants;
mod enemy;
mod resources;

fn window_conf() -> Conf {
    Conf {
        window_title: "Demottack".to_owned(),
        window_width: GAME_SIZE_X,
        window_height: GAME_SIZE_Y,
        ..Default::default()
    }
}

#[derive(PartialEq)]
pub enum PlayerState {
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
    pub fn new(
        pos: Vec2,
        texture: Texture2D,
        bullet_decoy_texture: Texture2D,
        texture_explotion: Texture2D,
    ) -> Self {
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

    pub fn reset(&mut self, resources: &Resources) {
        let player_spawn_y =
            GAME_SIZE_Y as f32 - resources.ground_bg.height() - resources.player.height();
        let player_pos = vec2(GAME_CENTER_X, player_spawn_y);
        self.pos = player_pos;
        self.shoot_timer = 0f32;
        self.state = PlayerState::Normal;
    }

    pub fn update(
        &mut self,
        dt: f32,
        bullets: &mut Vec<Bullet>,
        resources: &Resources,
        sound_mixer: &mut SoundMixer,
    ) {
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
                        bullets.push(Bullet::new(
                            self.pos + spawn_offset,
                            BulletHurtType::Enemy,
                            &resources,
                        ));
                        resources.play_sound(
                            SoundIdentifier::PlayerShoot,
                            sound_mixer,
                            Volume(1.0f32),
                        );
                        self.shoot_timer = 0f32;
                    }
                }
                None
            }
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

        let decoy_frame_index = ((self.shoot_timer / PLAYER_SHOOT_TIME) * 3f32) as i32;

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
        let fraction = (PLAYER_TIME_INVISBLE - time_left) / PLAYER_TIME_INVISBLE;
        let frame_index = (PLAYER_TIME_INVISBLE - time_left) / time_per_frame;
        let frame_index = frame_index.floor();

        draw_texture_ex(
            self.texture_explotion,
            self.pos.x - 5f32,
            self.pos.y - 4f32,
            WHITE,
            DrawTextureParams {
                rotation: fraction * 3.1415f32 * 2f32,
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
    ChangeState(WaveManagerState),
}

// used to get information from gamestate
pub enum WaveManagerMessage {
    LevelCleared,
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
    internal_timer: f32,
}

impl WaveManager {
    pub fn new() -> Self {
        let enemies_left = ENEMY_SPAWN_STARTING_COUNT;
        WaveManager {
            state: WaveManagerState::Spawning(WaveManagerStateSpawning {
                spawn_timer: 0f32,
                enemies_left,
            }),
            last_enemy_death_reason: LastEnemyDeathReason::Environment,
            internal_timer: 0f32,
        }
    }

    pub fn reset(&mut self) {
        let enemies_left = ENEMY_SPAWN_STARTING_COUNT;
        self.state = WaveManagerState::Spawning(WaveManagerStateSpawning {
            spawn_timer: 0f32,
            enemies_left,
        });
        self.last_enemy_death_reason = LastEnemyDeathReason::Environment;
        self.internal_timer = 0f32;
    }

    fn get_enemy_spawn_count(time: &f32) -> i32 {
        let fraction = time / TIME_UNTIL_MAX_DIFFICULTY;
        let spawn_countf32 = lininterp::lerp(
            &(ENEMY_SPAWN_STARTING_COUNT as f32),
            &(ENEMY_SPAWN_MAX_COUNT as f32),
            &fraction,
        );
        spawn_countf32 as i32
    }

    pub fn update(
        &mut self,
        dt: f32,
        enemies: &mut Vec<Enemy>,
        resources: &Resources,
        sound_mixer: &mut SoundMixer,
    ) -> Option<WaveManagerMessage> {
        self.internal_timer += dt;
        let state_command_optional = match &mut self.state {
            WaveManagerState::Spawning(game_state_spawning) => Self::update_state_spawning(
                game_state_spawning,
                dt,
                enemies,
                resources,
                sound_mixer,
            ),
            WaveManagerState::Battle => {
                Self::update_state_battle(dt, enemies, &self.internal_timer)
            }
        };

        if let Some(state_command) = state_command_optional {
            match state_command {
                WaveManagerCommand::ChangeState(target_state) => {
                    self.state = target_state;

                    let cleared_screen = variant_eq(
                        &self.state,
                        &WaveManagerState::Spawning(WaveManagerStateSpawning {
                            spawn_timer: 0f32,
                            enemies_left: 0,
                        }),
                    );
                    if cleared_screen {
                        return Some(WaveManagerMessage::LevelCleared);
                    }
                }
            }
        }
        None
    }

    fn update_state_battle(
        dt: f32,
        enemies: &mut Vec<Enemy>,
        internal_time: &f32,
    ) -> Option<WaveManagerCommand> {
        if enemies.len() <= 0 {
            let enemies_left = Self::get_enemy_spawn_count(internal_time);
            return Some(WaveManagerCommand::ChangeState(WaveManagerState::Spawning(
                WaveManagerStateSpawning {
                    enemies_left,
                    spawn_timer: 0f32,
                },
            )));
        }
        None
    }

    fn update_state_spawning(
        game_state_spawning: &mut WaveManagerStateSpawning,
        dt: f32,
        enemies: &mut Vec<Enemy>,
        resources: &Resources,
        sound_mixer: &mut SoundMixer,
    ) -> Option<WaveManagerCommand> {
        game_state_spawning.spawn_timer += dt;
        if game_state_spawning.spawn_timer > ENEMY_SPAWN_TIME {
            game_state_spawning.enemies_left -= 1;
            game_state_spawning.spawn_timer -= ENEMY_SPAWN_TIME;
            spawn_enemy(
                enemies,
                &resources,
                SpawnBlueprint::Normal,
                EnemyColor::random(),
            );
            resources.play_sound(SoundIdentifier::Spawn, sound_mixer, Volume(0.4f32));
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
pub fn spawn_enemy(
    enemies: &mut Vec<Enemy>,
    resources: &Resources,
    spawn_blueprint: SpawnBlueprint,
    enemy_color: EnemyColor,
) {
    let health = 1;
    let enemy = match spawn_blueprint {
        SpawnBlueprint::Normal => {
            let spawn_offset = vec2(
                rand::gen_range(-100f32, 100f32),
                rand::gen_range(-60f32, 10f32),
            );
            let spawn_pos = vec2(GAME_CENTER_X, GAME_CENTER_Y) + spawn_offset;
            let death_method = if rand::gen_range(0f32, 1f32) > 0.5f32 {
                let spawn_amount = rand::gen_range(1, 2 + 1);
                EnemyDeathMethod::SpawnChildren(spawn_amount)
            } else {
                EnemyDeathMethod::None
            };

            Enemy::new(
                spawn_pos,
                resources.rand_enemy_normal(enemy_color),
                health,
                death_method,
                EnemyType::Normal,
                enemy_color,
            )
        }
        SpawnBlueprint::Mini(pos) => Enemy::new(
            pos,
            resources.rand_enemy_mini(enemy_color),
            health,
            EnemyDeathMethod::None,
            EnemyType::Mini,
            enemy_color,
        ),
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
                .update(dt, &mut self.enemies, &resources, sound_mixer);
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
                &resources,
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
        for (i, bullet) in self
            .bullets
            .iter_mut()
            .filter(|b| b.hurt_type == BulletHurtType::Player)
            .enumerate()
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
        for (i, bullet) in self
            .bullets
            .iter_mut()
            .filter(|b| b.hurt_type == BulletHurtType::Enemy)
            .enumerate()
        {
            for (i, enemy) in self.enemies.iter_mut().enumerate() {
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
                            &resources,
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
            .update(dt, &mut self.bullets, &resources, sound_mixer);
        self.player.draw();
        None
    }

    fn draw(&self, resources: &Resources) {}

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
        sound_mixer: &mut SoundMixer,
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

    fn on_enter(&mut self, resources: &Resources, payload_optional: Option<ChangeStatePayload>) {
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

#[macroquad::main(window_conf)]
async fn main() {
    let game_render_target = render_target(GAME_SIZE_X as u32, GAME_SIZE_Y as u32);
    let resources = load_resources(game_render_target).await;
    let mixer = SoundMixer::new();

    let game_states: Vec<(GameStateIdentifier, Box<dyn GameState>)> = vec![
        (GameStateIdentifier::Menu, Box::new(GameStateMenu::new())),
        (
            GameStateIdentifier::Game,
            Box::new(GameStateGame::new(&resources)),
        ),
    ];
    let mut game_manager = GameManager::new(game_states, resources, mixer);

    loop {
        let dt = get_frame_time();

        set_camera(Camera2D {
            // I have no idea why the zoom is this way lmao
            zoom: vec2(1. / GAME_SIZE_X as f32 * 2., 1. / GAME_SIZE_Y as f32 * 2.),
            target: vec2(
                (GAME_SIZE_X as f32 * 0.5f32).floor(),
                (GAME_SIZE_Y as f32 * 0.5f32).floor(),
            ),
            render_target: Some(game_render_target),
            ..Default::default()
        });
        clear_background(BLACK);

        game_manager.update(dt);
        game_manager.draw();

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
                dest_size: Some(Vec2::new(scaled_game_size_w, scaled_game_size_h)),
                ..Default::default()
            },
        );

        game_manager.draw_unscaled();

        game_manager.frame_sounds();

        next_frame().await
    }
}
