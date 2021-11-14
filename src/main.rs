use macroquad::prelude::*;
use quad_snd::mixer::SoundMixer;

use constants::*;
use game::{GameManager, GameState, GameStateGame, GameStateIdentifier, GameStateMenu};
use resources::load_resources;

mod bullet;
mod constants;
mod enemy;
mod game;
mod player;
mod resources;
mod wave;

fn window_conf() -> Conf {
    Conf {
        window_title: "Demottack".to_owned(),
        window_width: GAME_SIZE_X,
        window_height: GAME_SIZE_Y,
        ..Default::default()
    }
}

// used to compare enums without having to match against it's values
// example what we avoid: emotion_enum == Emotion::Happy{happines_level: 0f32, visible_on_face: false,}
// the values needs to be constructed, but comparison is top-level
fn variant_eq<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
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
        let camera = Camera2D {
            // I have no idea why the zoom is this way lmao
            zoom: vec2(1. / GAME_SIZE_X as f32 * 2., 1. / GAME_SIZE_Y as f32 * 2.),
            target: vec2(
                (GAME_SIZE_X as f32 * 0.5f32).floor(),
                (GAME_SIZE_Y as f32 * 0.5f32).floor(),
            ),
            render_target: Some(game_render_target),
            ..Default::default()
        };
        set_camera(&camera);
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
