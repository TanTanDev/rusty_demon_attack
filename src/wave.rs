use macroquad::prelude::*;
use quad_snd::mixer::{SoundMixer, Volume};

use crate::{
    constants::*,
    enemy::{Enemy, EnemyColor, EnemyDeathMethod, EnemyType},
    resources::{Resources, SoundIdentifier},
    variant_eq,
};

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
    pub state: WaveManagerState,
    pub last_enemy_death_reason: LastEnemyDeathReason,
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
            WaveManagerState::Battle => Self::update_state_battle(enemies, &self.internal_timer),
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
        enemies: &mut Vec<Enemy>,
        internal_time: &f32,
    ) -> Option<WaveManagerCommand> {
        if enemies.is_empty() {
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
                resources,
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
