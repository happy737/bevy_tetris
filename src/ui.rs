use bevy::prelude::*;
use bevy::ecs::system::SystemId;
use serde::{Serialize, Deserialize};

use crate::engine;

const EMPTY_BACKGROUND_COLOR: bevy::ui::BackgroundColor = BackgroundColor(Color::LinearRgba(LinearRgba { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0 }));
const HOVERED_BUTTON_BACKGROUND_COLOR: bevy::ui::BackgroundColor = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
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
        app.add_systems(Update, audio_button_listener);
        app.add_systems(Update, key_mapping_button_listener);
        app.add_systems(Update, display_music_volume_settings);
        app.add_systems(Update, update_music_volume_settings);
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

    commands.insert_resource(Settings::from_serialized_or_default());
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
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                children![
                    generate_top_level_settings_line_element("Audio", AudioButton),
                    generate_top_level_settings_line_element("Key Mapping", KeyMappingButton), 
                ],
            ),
        ],
    )
}

fn generate_top_level_settings_line_element<T>(text: &str, button_marker: T) -> impl Bundle + use<T> 
where T: Component {
    (
        Node {
            padding: UiRect::all(Val::Px(5.0)),
            margin: UiRect::all(Val::Px(15.0)),
                            
            ..Default::default()
        },
        Button, 
        button_marker,
        children![
            (
                Text::new(text),
                TextFont {
                    font_size: 30.0,
                    ..Default::default()
                },
            ),
        ],
    )
}

fn generate_audio_settings() -> impl Bundle + use<> {
    (
        Node {
            flex_direction: FlexDirection::Row,
            align_content: AlignContent::Center, 
            justify_content: JustifyContent::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..Default::default()
        },
        BorderColor(Color::srgb(0.0, 1.0, 0.0)),
        children![
            (
                Node {
                    height: Val::Percent(100.0),
                    align_content: AlignContent::Center, 
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                children![
                    Node {
                        align_content: AlignContent::Center,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    Text::new("Music Volume: "),
                ]
            ),
            (
                Node::DEFAULT,
                Button,
                DecreaseMusicVolumeButton,
                BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                children![
                    Text::new(" - "),
                ],
            ),
            (
                Node::DEFAULT,
                Text::new(" ??% "),
                MusicVolumeTextMarker,
            ),
            (
                Node::DEFAULT,
                Button,
                Text::new(" + "),
                IncreaseMusicVolumeButton,
                BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
            )
        ]
    )   //TODO
}

#[derive(Component)]
pub struct PausedTopDiv;

#[derive(Component)]
pub struct MusicVolumeTextMarker;

#[derive(Component)]
pub struct DecreaseMusicVolumeButton;

#[derive(Component)]
pub struct IncreaseMusicVolumeButton;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsTab {
    Audio, 
    KeyMapping,
}

#[derive(Component)]
pub struct AudioButton;

#[derive(Component)]
pub struct KeyMappingButton;

fn audio_button_listener(
    mut button_query: Query<(&Interaction, &mut bevy::ui::BackgroundColor), (Changed<Interaction>, With<Button>, With<AudioButton>)>,
    mut commands: Commands,
    mut settings_tab_query: Query<&mut SettingsTab>,
    paused_top_div_query: Query<Entity, With<PausedTopDiv>>,
    mut settings: ResMut<Settings>,
) {
    let Ok((interaction, mut backgroud_color)) = button_query.single_mut() else {return;};

    match interaction {
        Interaction::Pressed => {
            if let Ok(mut settings_tab) = settings_tab_query.single_mut() {
                if *settings_tab == SettingsTab::Audio {
                    return;
                }
                *settings_tab = SettingsTab::Audio;
            } else {
                commands.spawn(SettingsTab::Audio);
            }

            let Ok(entity) = paused_top_div_query.single() else {return;};

            commands.entity(entity).with_child(generate_audio_settings());
        }
        Interaction::Hovered => {
            *backgroud_color = HOVERED_BUTTON_BACKGROUND_COLOR;
        }
        Interaction::None => {
            *backgroud_color = EMPTY_BACKGROUND_COLOR;
        }
    }
}

fn key_mapping_button_listener(
    mut button_query: Query<(&Interaction, &mut bevy::ui::BackgroundColor), (Changed<Interaction>, With<Button>, With<KeyMappingButton>)>,
) {
    let Ok((interaction, mut backgroud_color)) = button_query.single_mut() else {return;};

    match interaction {
        Interaction::Pressed => {
            info!("Key Mapping Button has been pressed!");
        }
        Interaction::Hovered => {
            *backgroud_color = HOVERED_BUTTON_BACKGROUND_COLOR;
        }
        Interaction::None => {
            *backgroud_color = EMPTY_BACKGROUND_COLOR;
        }
    }
}

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

#[derive(Resource, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub music_volume: f32,
}

impl Settings {
    fn write_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = std::fs::create_dir("./data");

        //get a file 
        let file = std::fs::File::create("data/settings.dat")?;

        //serialize and write to file
        serde_cbor::to_writer(file, &self)?;

        Ok(())
    }

    fn new_from_serialized() -> Result<Self, Box<dyn std::error::Error>> {
        //open the file
        let file = std::fs::File::open("data/settings.dat")?;

        //deserialze 
        let settings = serde_cbor::from_reader(file)?;

        Ok(settings)
    }

    fn from_serialized_or_default() -> Self {
        //try to 
        let attempt = Settings::new_from_serialized();
        match attempt {
            Ok(settings) => {
                return settings;
            }
            Err(err) => {
                warn!("Could not load user settings. Using default instead. Error: {}", err);
            }
        }

        Settings::default()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self { 
            music_volume: 0.5,
        }
    }
}

fn display_music_volume_settings(
    settings: Res<Settings>,
    mut text_query: Query<&mut Text, With<MusicVolumeTextMarker>>,
) {
    let Ok(mut text) = text_query.single_mut() else {return;};

    *text = Text::new(format!(" {:.0} ", settings.music_volume * 100.0));
}

fn update_music_volume_settings(
    mut settings: ResMut<Settings>,
    decrease_query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<DecreaseMusicVolumeButton>)>,
    increase_query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<IncreaseMusicVolumeButton>)>,
) {
    let mut changed = false;

    'decrease: {
        let Ok(interaction) = decrease_query.single() else {break 'decrease};

        if *interaction == Interaction::Pressed {
            settings.music_volume = (settings.music_volume - 0.1).max(0.0);
            changed = true;
        }
    }

    'increase: {
        let Ok(interaction) = increase_query.single() else {break 'increase};

        if *interaction == Interaction::Pressed {
            settings.music_volume = (settings.music_volume + 0.1).min(1.0);
            changed = true;
        }
    }

    if changed {
        if let Err(err) = settings.write_to_file() {
            error!("Could not save current settings. Error: {}", err);
        }
    }
}
