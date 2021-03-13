use macroquad::prelude::*;

//const GAME_SIZE_X: i32 = 160;
const GAME_SIZE_X: i32 = 240;
const GAME_SIZE_Y: i32 = 130;
const GAME_CENTER_X: f32 = GAME_SIZE_X as f32 * 0.5f32;
const GAME_CENTER_Y: f32 = GAME_SIZE_Y as f32 * 0.5f32;
const _ASPECT_RATIO: f32 = GAME_SIZE_X as f32 / GAME_SIZE_Y as f32;

const KEY_RIGHT: KeyCode = KeyCode::Right;
const KEY_LEFT: KeyCode = KeyCode::Left;
const KEY_SHOOT: KeyCode = KeyCode::Space;

const PLAYER_SPEED: f32 = 80f32;
const PLAYER_SHOOT_TIME: f32 = 0.8f32;
const PLAYER_BULLET_SPEED: f32 = 80f32;
const PLAYER_LIVES_START: i32 = 7i32;
const PLAYER_LIVES_MAX: i32 = 7i32;
const PLAYER_TIME_INVISBLE: f32 = 2f32;

const ENEMY_SPEED: f32 = 0.3f32;
const ENEMY_BULLET_SPEED: f32 = 80f32;
const ENEMY_SHOOT_TIME: f32 = 2f32;
const ENEMY_ANIM_TIME_SPAWN: f32 = 0.7f32;
const ENEMY_ANIM_TIME_FLAP: f32 = 0.12f32;
const ENEMY_ANIM_SPAWN_SCALE: f32 = 4.0f32;
// how far away the spawn animation starts
const ENEMY_ANIM_DISTANCE: f32 = 140f32;
// only one enemy should spawn at a time, this delay
const ENEMY_ENTRANCE_DELAY: f32 = 2f32;

// Enemy Spawn management
const ENEMY_MAX_COUNT: i32 = 5;
// spawn every x sec
const ENEMY_SPAWN_TIME: f32 = 1.0f32;


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

pub enum EnemyState {
    Spawning(EnemyStateSpawning),
    Normal(EnemyStateNormal),
}

pub enum EnemyDeathMethod {
    None,
    // count
    SpawnChildren(i32),
}

pub enum EnemyCommand {
    ChangeState(EnemyState),
}

pub struct EnemyStateShared {
    texture: Texture2D,
    pos: Vec2,
    collision_rect: Rect,
    health: i32,
    death_method: EnemyDeathMethod,
    animation_timer: f32,
}

pub struct EnemyStateNormal {
    shoot_timer: f32,
}

pub struct EnemyStateSpawning {
    spawn_timer: f32,
}

pub struct Enemy {
    state_shared: EnemyStateShared,
    state: EnemyState,
}

impl Enemy {
    pub fn new(pos: Vec2, texture: Texture2D, health: i32, death_method: EnemyDeathMethod) -> Self {
        Enemy {
            state_shared: EnemyStateShared {
                pos,
                texture,
                collision_rect: Rect::new(0f32, 0f32, texture.width(), texture.height()),
                health,
                death_method,
                animation_timer: 0f32,
            },
            state: EnemyState::Spawning(EnemyStateSpawning{
                spawn_timer: 0f32,
            }),
        }
    }

    pub fn update(&mut self, dt: f32, bullets: &mut Vec::<Bullet>, textures: &Textures) {
        let command_optional = match &mut self.state {
            EnemyState::Spawning(state_data) => Self::update_state_spawning(&mut self.state_shared, dt, state_data),
            EnemyState::Normal(state_data) => Self::update_state_normal(&mut self.state_shared, dt, bullets, textures, state_data),
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

    fn update_state_spawning(state_shared: &mut EnemyStateShared, dt: f32, state_data: &mut EnemyStateSpawning) -> Option<EnemyCommand> {
        // ANIMATE TIMER
        state_data.spawn_timer += dt;
        // fraction 
        let fraction = state_data.spawn_timer / ENEMY_ANIM_TIME_SPAWN;
        if fraction >= 1.0f32 {
            return Some(EnemyCommand::ChangeState(EnemyState::Normal(EnemyStateNormal {shoot_timer: 0f32})));
        }
        None
    }

    fn update_state_normal(state_shared: &mut EnemyStateShared, dt: f32, bullets: &mut Vec::<Bullet>, textures: &Textures, state_data: &mut EnemyStateNormal) -> Option<EnemyCommand> {
        state_shared.pos.x += rand::gen_range(-1f32, 1f32) * ENEMY_SPEED;
        state_shared.pos.y += rand::gen_range(-1f32, 1f32) * ENEMY_SPEED;
        state_data.shoot_timer += dt;
        state_shared.animation_timer += dt;
        if state_shared.animation_timer > ENEMY_ANIM_TIME_FLAP*4f32 {
            state_shared.animation_timer -= ENEMY_ANIM_TIME_FLAP*4f32;
        }
        if state_data.shoot_timer > ENEMY_SHOOT_TIME {
            let spawn_offset = vec2(0f32, 0f32);
            bullets.push(Bullet::new(state_shared.pos + spawn_offset, BulletHurtType::Player, &textures));
            state_data.shoot_timer -= ENEMY_SHOOT_TIME;
        }
        state_shared.collision_rect.x = state_shared.pos.x - state_shared.texture.width()*0.5f32;
        state_shared.collision_rect.y = state_shared.pos.y;
        None
    }

    fn draw_state_spawning(state_shared: &EnemyStateShared, state_data: &EnemyStateSpawning) {
        let rand_frame = rand::gen_range(0i32, 2i32);
        //draw_circle(self.pos.x, self.pos.y, 1.0f32, RED);
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
    pos: Vec2,
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
        }
        if is_key_down(KEY_RIGHT) {
            self.pos.x += PLAYER_SPEED * dt; 
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
    demon_1: Texture2D,
    demon_2: Texture2D,
    demon_missile: Texture2D,
    player_missile: Texture2D,
}

pub struct GameManager {
    spawn_timer: f32,
}

impl GameManager {
    pub fn update(&mut self, dt: f32, enemies: &mut Vec<Enemy>, textures: &Textures) {
        self.spawn_timer += dt;
        if self.spawn_timer > ENEMY_SPAWN_TIME { 
            if enemies.len() > 7 {
                enemies.remove(rand::gen_range(0, enemies.len()));
            }
            self.spawn_timer -= ENEMY_SPAWN_TIME;
            // spawn enemy
            let spawn_offset = vec2(rand::gen_range(-100f32, 100f32), rand::gen_range(-60f32, 10f32));
            let health = 1;
            let death_method = EnemyDeathMethod::None;
            let mut enemy = Enemy::new(vec2(GAME_CENTER_X, GAME_CENTER_Y) + spawn_offset, [textures.demon_1, textures.demon_2][rand::gen_range(0, 2)], health, death_method);
            enemies.push(enemy);
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let game_render_target = render_target(GAME_SIZE_X as u32, GAME_SIZE_Y as u32);
    let mut texture_demon_1: Texture2D = load_texture("resources/demon_1.png").await;
    let mut texture_demon_2: Texture2D = load_texture("resources/demon_2.png").await;
    let mut texture_player: Texture2D = load_texture("resources/player.png").await;
    let mut texture_player_explotion: Texture2D = load_texture("resources/player_explotion.png").await;
    let mut texture_player_missile: Texture2D = load_texture("resources/player_missile.png").await;
    let mut texture_demon_missile: Texture2D = load_texture("resources/demon_missile.png").await;
    let mut texture_ground_bg: Texture2D = load_texture("resources/ground_bg.png").await;
    let mut texture_life: Texture2D = load_texture("resources/life.png").await;
    set_texture_filter(game_render_target.texture, FilterMode::Nearest);
    set_texture_filter(texture_demon_1, FilterMode::Nearest);
    set_texture_filter(texture_demon_2, FilterMode::Nearest);
    set_texture_filter(texture_player, FilterMode::Nearest);
    set_texture_filter(texture_player_explotion, FilterMode::Nearest);
    set_texture_filter(texture_player_missile, FilterMode::Nearest);
    set_texture_filter(texture_demon_missile, FilterMode::Nearest);
    set_texture_filter(texture_ground_bg, FilterMode::Nearest);
    set_texture_filter(texture_life, FilterMode::Nearest);

    let font = load_ttf_font("resources/Kenney Pixel Square.ttf").await;
    let mut player_score: i32 = 0;
    let mut player_lives: i32 = PLAYER_LIVES_START;

    let textures = Textures {
        demon_1: texture_demon_1,
        demon_2: texture_demon_2,
        demon_missile: texture_demon_missile,
        player_missile: texture_player_missile,
    };

    let mut bullets = Vec::<Bullet>::new();
    let mut enemies = Vec::<Enemy>::new();

    let spawn_count = 2;
    for _ in 0..spawn_count {
        let health = 1;
        let death_method = EnemyDeathMethod::None;
        let mut enemy = Enemy::new(vec2(GAME_CENTER_X, GAME_CENTER_Y), [texture_demon_1, texture_demon_2][rand::gen_range(0, 2)], health, death_method);
        enemies.push(enemy);
    }

    let player_spawn_y = GAME_SIZE_Y as f32 - texture_ground_bg.height() - texture_player.height();
    let mut player = Player::new(vec2(GAME_CENTER_X, player_spawn_y), texture_player, texture_player_missile, texture_player_explotion);

    let mut game_manager = GameManager {
        spawn_timer: 0f32,
    };

    loop {
        let dt = get_frame_time();

        set_camera(Camera2D {
            //zoom: vec2(0.01, 0.01),
            // I have no idea why the zoom is this way lmao
            zoom: vec2(1./GAME_SIZE_X as f32*2., 1./GAME_SIZE_Y as f32*2.),
            target: vec2((GAME_SIZE_X as f32*0.5f32).floor(), (GAME_SIZE_Y as f32 * 0.5f32).floor()),
            render_target: Some(game_render_target),
            ..Default::default()
        });
        clear_background(BLACK);
        game_manager.update(dt, &mut enemies, &textures);

        for enemy in enemies.iter_mut() {
            enemy.update(dt, &mut bullets, &textures);
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

        // bullets hurting enemies
        for (i, bullet) in bullets.iter_mut()
            .filter(|b| b.hurt_type == BulletHurtType::Enemy)
            .enumerate()
        {
            for (i, enemy) in enemies.iter_mut().enumerate() {
                if enemy.overlaps(&bullet.collision_rect) {
                    enemy.state_shared.health -= 1;
                    // death
                    if enemy.state_shared.health <= 0 {
                        match enemy.state_shared.death_method {
                            EnemyDeathMethod::None => {},
                            EnemyDeathMethod::SpawnChildren(amount) => {
                                debug!("Supposed to spawn children here");
                            },
                        }
                    }
                    // can only hurt one enemy, flag for deletion
                    bullet.is_kill = true;
                    break;
                }
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

        // draw lives
        let lives_padding = 2f32;
        for i in 0..player_lives {
            draw_texture_ex(
                texture_life,
                5f32 + i as f32 * (texture_life.width() + lives_padding),
                GAME_SIZE_Y as f32 - texture_ground_bg.height() + 3f32,
                WHITE,
                DrawTextureParams {
                    //dest_size: Some(vec2(screen_width(), screen_height())),
                    //dest_size: Some(Vec2::new(GAME_SIZE_X as f32,  texture_ground_bg.height())),
                    ..Default::default()
                },
            );
        }

        player.update(dt, &mut bullets, &textures);
        player.draw();

        player_score +=1; 
        let score_text = format!("{}", player_score); 
        let mut font_center = GAME_SIZE_X as f32 * 0.5f32;
        font_center -= score_text.len() as f32 * 0.5f32 * 20f32 *0.6f32;
        draw_text_ex(score_text.as_ref(), font_center, 40.0
            , TextParams{
                font,
                font_size: 20u16,
                font_scale: 1f32,
                color: YELLOW,
            }
        );

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

        next_frame().await
    }
}