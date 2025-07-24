use bevy::prelude::*;
use bevy::ecs::system::SystemId;

use crate::engine;

const EMPTY_BACKGROUND_COLOR: bevy::ui::BackgroundColor = BackgroundColor(Color::LinearRgba(LinearRgba { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0 }));
const PLAYFIELD_WIDTH_IN_PX: f32 = 400.0;

pub struct MyUiPlugin;

impl Plugin for MyUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(engine::scene::ScenePlugin);
        app.add_systems(Startup, setup);
        app.add_systems(Update, update_horizontal_pad_window_width);
        app.add_systems(Update, update_score);
        app.add_systems(Update, update_level);
        app.add_systems(Update, new_game_button_listener);
    }
}

fn setup(
    mut commands: Commands, 
    _asset_server: Res<AssetServer>
) {
    //test window
    commands.spawn(generate_screen_window());


    let game_over = commands.register_system(spawn_game_over_screen);
    commands.insert_resource(SpawnGameOverSystem(game_over));

    let pause = commands.register_system(spawn_pause_screen);
    commands.insert_resource(SpawnPauseSystem(pause));
}

fn spawn_game_over_screen(
    mut commands: Commands,
) {
    commands.spawn(generate_game_over_screen());
}

fn spawn_pause_screen(
    mut commands: Commands,
) {
    commands.spawn(generate_pause_screen());
}

#[derive(Resource)]
pub(crate) struct SpawnGameOverSystem(pub SystemId);

#[derive(Resource)]
pub struct SpawnPauseSystem(pub SystemId);

fn generate_screen_window() -> impl Bundle + use<> {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column, 

            ..Default::default()
        },
        EMPTY_BACKGROUND_COLOR,
        children![
            generate_pad_window(Val::Percent(100.0), Val::Percent(55.0)),
            generate_centered_strip_window(),
        ],
    )
}

fn generate_pad_window(width: Val, height: Val) -> impl Bundle + use<> {
    (
        Node {
            width,
            height,

            ..Default::default()
        },
        EMPTY_BACKGROUND_COLOR,
    )
}

fn generate_horizontal_pad_window() -> impl Bundle + use<> {
    (
        Node {
            width: Val::Percent(65.0),
            height: Val::Percent(100.0),

            ..Default::default()
        },
        EMPTY_BACKGROUND_COLOR,
        HorizontalPadMarker,
    )
}

#[derive(Component)]
struct HorizontalPadMarker;

fn generate_centered_strip_window() -> impl Bundle + use<> {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(10.0),
            //border: UiRect::all(Val::Px(2.0)),

            ..Default::default()
        },
        EMPTY_BACKGROUND_COLOR,
        BorderColor(Color::LinearRgba(LinearRgba::GREEN)),
        children![
            generate_horizontal_pad_window(),
            main_content_window(),
        ],
    )
}

fn main_content_window() -> impl Bundle + use<> {
    (
        Node {
            width: Val::Px(200.0),
            min_width: Val::Px(200.0),
            height: Val::Auto,
            //border: UiRect::all(Val::Px(2.0)),
            flex_direction: FlexDirection::Column, 

            ..Default::default()
        },
        EMPTY_BACKGROUND_COLOR,
        BorderColor(Color::LinearRgba(LinearRgba::GREEN)),
        children![
            generate_text_window("Score: ", ScoreTextMarker),
            generate_text_window("Level: ", LevelTextMarker),
        ],
    )
}

fn update_horizontal_pad_window_width(
    window_query: Query<&Window>,
    mut node_query: Query<&mut Node, With<HorizontalPadMarker>>,
) {
    let Ok(window) = window_query.single() else {return;};
    let window_width = window.physical_width() as f32;
    let window_height = window.physical_height() as f32;
    let window_height_factor = 700.0 / window_height;
    let playfield_width = PLAYFIELD_WIDTH_IN_PX / window_height_factor;

    let Ok(mut node) = node_query.single_mut() else {return;};
    let value = Val::Px((window_width - playfield_width) / 2.0 + playfield_width);
    node.min_width = value;
    node.max_width = value;
}

fn generate_text_window<T>(text: &str, text_marker: T) -> impl Bundle + use<T> 
where T: Component {
    (
        Node::DEFAULT,
        children![
            (
                Node::DEFAULT,
                Text::new(text),
                text_marker
            )
        ],
    )
}

#[derive(Component)]
struct ScoreTextMarker;

fn update_score(
    score: Res<engine::scene::GameScore>,
    text_query: Query<&mut Text, With<ScoreTextMarker>>,
) {
    for mut text in text_query {
        *text = Text::new(format!("Score: {}", score.score));
    }
}

#[derive(Component)]
struct LevelTextMarker;

fn update_level(
    score: Res<engine::scene::GameScore>,
    text_query: Query<&mut Text, With<LevelTextMarker>>,
) {
    for mut text in text_query {
        *text = Text::new(format!("Level: {}", score.level));
    }
}

fn generate_game_over_screen() -> impl Bundle + use<> {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column, 

            ..Default::default()
        },
        EMPTY_BACKGROUND_COLOR,
        NewGameTopDiv,
        children![
            (   //Game Over Text
                Node {
                    ..Default::default()
                },
                children![
                    (
                        Node::DEFAULT,
                        Text::new("GAME OVER!"),
                        TextFont {
                            font_size: 100.0,
                            ..Default::default()
                        },
                        TextColor(Color::srgb(1.0, 0.0, 0.0)),
                    )
                ],
            ),
            (   //Button
                Node::DEFAULT,
                children![
                    (
                        Node::DEFAULT,
                        Text::new(" New Game "),    //spaces because I cant be arsed to configure the node
                        TextFont {
                            font_size: 50.0,
                            ..Default::default()
                        },
                        Button,
                        NewGameButton,
                        EMPTY_BACKGROUND_COLOR,
                    )
                ]
            )
        ],
    )
}

#[derive(Component)]
struct NewGameButton;

#[derive(Component)]
struct NewGameTopDiv;

fn generate_pause_screen() -> impl Bundle + use<> {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 0.5)),
        PausedTopDiv,
        children![
            (   //Paused Text Screen
                Node::DEFAULT,
                Text::new("Game Paused"),
                TextFont {
                    font_size: 50.0,
                    ..Default::default()
                },
            ),
            (   //Settings

            )   //TODO
        ],
    )
}

#[derive(Component)]
pub struct PausedTopDiv;

fn new_game_button_listener(
    mut button_query: Query<(&Interaction, &mut bevy::ui::BackgroundColor), (Changed<Interaction>, With<Button>, With<NewGameButton>)>, 
    mut game_query: Query<&mut engine::scene::Game>,
    mut game_score: ResMut<engine::scene::GameScore>,
    mut is_game_running: ResMut<engine::scene::IsAppRunning>,
    main_div_query: Query<Entity, With<NewGameTopDiv>>,
    mut commands: Commands, 
) {
    let Ok((interaction, mut background_color)) = button_query.single_mut() else {return;};

    match interaction {
        Interaction::Pressed => {
            info!("Starting new Game");

            //reset the playfield
            let Ok(mut game) = game_query.single_mut() else {return;};
            game.tetris = engine::model::Tetris::default();
            
            //reset the score
            *game_score = engine::scene::GameScore::default();

            //remove game over screen
            let Ok(main_div) = main_div_query.single() else {error!("Failed to remove New Game main div!"); return;};
            commands.entity(main_div).despawn();

            //set game to be running again
            is_game_running.0 = engine::scene::AppState::Running;
        }
        Interaction::Hovered => {
            *background_color = BackgroundColor(Color::Srgba(Srgba { red: 1.0, green: 1.0, blue: 1.0, alpha: 0.2 }));
            //*background_color = BackgroundColor(Color::Srgba(Srgba::WHITE));
        }
        Interaction::None => {
            *background_color = EMPTY_BACKGROUND_COLOR;
        }
    }
}
