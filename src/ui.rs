use bevy::prelude::*;
use bevy::ecs::system::SystemId;
use serde::{ser::SerializeStruct, Deserialize, Serialize};

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
        app.add_systems(Update, update_primary_key_bind_settings);
        app.add_systems(Update, update_secondary_key_bind_settings);
        app.add_systems(Update, individual_keybind_button_listener);
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

    let clear_keybind_clicks = commands.register_system(clear_keybind_clicks);
    commands.insert_resource(ClearKeybindClicks(clear_keybind_clicks));

    let highlight_clicked_keybinds = commands.register_system(highlight_clicked_keybind);
    commands.insert_resource(HighlghtClickedKeybind(highlight_clicked_keybinds));

    commands.insert_resource(Settings::from_serialized_or_default());
}

/// One shot function which spawns the game over screen. 
fn spawn_game_over_screen(
    mut commands: Commands,
) {
    commands.spawn(generate_game_over_screen());
}

/// One shot function which spawns the pause game screen. 
fn spawn_pause_screen(
    mut commands: Commands,
) {
    commands.spawn(generate_pause_screen());
}

/// A resource which holds the id of the spawn_game_over_screen function. 
#[derive(Resource)]
pub(crate) struct SpawnGameOverSystem(pub SystemId);

/// A resource which holds the id of the spawn_pause_screen function. 
#[derive(Resource)]
pub struct SpawnPauseSystem(pub SystemId);

/// A resource which holds the id of the clear_keybind_click function. 
#[derive(Resource)]
pub struct ClearKeybindClicks(pub SystemId);

/// A resource which holds the id of the highlight_clicked_keybind function. 
#[derive(Resource)]
pub struct HighlghtClickedKeybind(pub SystemId);

/// One shot function which clears all backgrounds of the individual key bind settings fields. 
fn clear_keybind_clicks(
    backgrounds_query: Query<&mut BackgroundColor, (With<TetrisInstruction>, With<Button>)>,
) {
    for mut background in backgrounds_query {
        *background = EMPTY_BACKGROUND_COLOR;
    }
}

/// One shot function which changes the background of the clicked key bind settings field. 
fn highlight_clicked_keybind(
    backgrounds_query: Query<(&mut BackgroundColor, &TetrisInstruction, &KeyBindClickArea), With<Button>>,
    waiting_query: Query<&WaitingForNewKeyBind>,
) {
    let Ok(waiting) = waiting_query.single() else {
        error!("Called 'highlight_clicked_keybind' with no or too many selected clicked keybinds!");
        return;
    };

    for (mut background, instruction, number) in backgrounds_query {
        if *instruction == waiting.selected_tetris_instruction && number.0 == waiting.primary {
            *background = BackgroundColor(Color::srgb(0.5, 0.5, 0.5));
        }
    }
}

/// Generates the UI setup for the general in game UI. 
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
            //empty padding on top
            generate_pad_window(Val::Percent(100.0), Val::Percent(55.0)),
            
            //scoreboard content which is pushed down to about halfway
            generate_centered_strip_window(),
        ],
    )
}

/// Creates an empty UI "div" with the specified width and height to serve as padding. 
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

/// Creates an empty UI "div" specifically for the scoreboard ui. It contains a special 
/// marker component [HorizontalPadMarker] for continuous width adjustment when resizing
/// the window. 
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

/// The specialized marker to continuously update the position of the scoreboard when 
/// resizing the window. 
#[derive(Component)]
struct HorizontalPadMarker;

/// Creates the second horizontal UI component.  
fn generate_centered_strip_window() -> impl Bundle + use<> {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(10.0),
            //border: UiRect::all(Val::Px(2.0)),

            ..Default::default()
        },
        EMPTY_BACKGROUND_COLOR,
        //BorderColor(Color::LinearRgba(LinearRgba::GREEN)),
        children![
            //padding for the main content
            generate_horizontal_pad_window(),
            //the small "scoreboard" 
            main_content_window(),
        ],
    )
}

/// Creates the "scoreboard" UI "div". 
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

/// Continuously updates the horizontal pad for the main content div to account for 
/// window resizing. 
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

/// Creates a UI component containing some text and its respective marker, allowing 
/// it to be updated.  
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

/// The marker to change the players score. 
#[derive(Component)]
struct ScoreTextMarker;

/// Continuously updates the users score with the score from the [engine::scene::GameScore] 
/// entity. 
fn update_score(
    score: Res<engine::scene::GameScore>,
    text_query: Query<&mut Text, With<ScoreTextMarker>>,
) {
    for mut text in text_query {
        *text = Text::new(format!("Score: {}", score.score));
    }
}

/// The marker to change the players level. 
#[derive(Component)]
struct LevelTextMarker;

/// Continuously updates the users level with the level from the [engine::scene::GameScore] 
/// entity. 
fn update_level(
    score: Res<engine::scene::GameScore>,
    text_query: Query<&mut Text, With<LevelTextMarker>>,
) {
    for mut text in text_query {
        *text = Text::new(format!("Level: {}", score.level));
    }
}

/// Creates the entire screen spanning game over screen UI component. 
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

/// The marker to mark the new game button. 
#[derive(Component)]
struct NewGameButton;

/// The marker for the screen spanning div of the game over screen. 
#[derive(Component)]
struct NewGameTopDiv;

/// Creates the pause menu screen UI component. 
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

/// Creates the UI buttons which will select which settings will be shown in the pause game
/// menu. 
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

/// Creates the UI components which will form the audio settings. 
fn generate_audio_settings() -> impl Bundle + use<> {
    (
        Node {
            flex_direction: FlexDirection::Row,
            align_content: AlignContent::Center, 
            justify_content: JustifyContent::Center,

            margin: UiRect::all(Val::Px(5.0)),
            padding: UiRect::all(Val::Px(5.0)),

            ..Default::default()
        },
        PauseMenuRemovableChildren,
        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
        children![
            (
                Node {
                    height: Val::Percent(100.0),
                    align_content: AlignContent::Center, 
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                children![
                    (
                        Node {
                            align_content: AlignContent::Center,
                            justify_content: JustifyContent::Center,
                            ..Default::default()
                        },
                        Text::new("Music Volume: "),
                    ),
                ],
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

/// Creates the UI components which will form the settings for a single action keybind. 
fn generate_single_key_bind_entry(action_description: &str, instruction: TetrisInstruction) -> impl Bundle + use<> {
    (
        Node {
            flex_direction: FlexDirection::Row,
            margin: UiRect::all(Val::Px(5.0)),
            padding: UiRect::all(Val::Px(5.0)),
            ..Default::default()
        },
        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
        children![
            (   //action descriptor
                Node {
                    align_content: AlignContent::Center,
                    ..Default::default()
                },
                children![
                    (
                        Node {
                            align_self: AlignSelf::Center,
                            ..Default::default()
                        },
                        Text::new(action_description),
                    ),
                ],
            ),
            (   //primary key bind
                Node {
                    margin: UiRect::horizontal(Val::Px(10.0)),  //middle div is responsible for spacing
                    border: UiRect::all(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(4.0)),
                    ..Default::default()
                },
                BorderColor(Color::Srgba(Srgba::BLACK)),
                Button,
                EMPTY_BACKGROUND_COLOR,
                KeyBindClickArea(1),
                instruction,
                children![
                    (
                        instruction,
                        FirstKeyBindTextMarker,
                        Text::new("PRIMARY KEY BIND"),
                    )
                ],
            ),
            (   //secondary key bind
                Node {
                    border: UiRect::all(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(4.0)),
                    ..Default::default()
                },
                BorderColor(Color::Srgba(Srgba::BLACK)),
                Button,
                EMPTY_BACKGROUND_COLOR,
                KeyBindClickArea(2),
                instruction,
                children![
                    (
                        instruction,
                        SecondKeyBindTextMarker,
                        Text::new("SECONDARY KEY BIND"),
                    )
                ],
            ),
        ],
    )
}

/// Creates the UI component of the entire key bind settings. 
fn generate_key_bind_menu() -> impl Bundle + use<> {
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_content: AlignContent::FlexStart,
            ..Default::default()
        },
        PauseMenuRemovableChildren,
        children![
            generate_single_key_bind_entry("Drop", TetrisInstruction::Drop),
            generate_single_key_bind_entry("Full Drop", TetrisInstruction::FullDrop),
            generate_single_key_bind_entry("Left", TetrisInstruction::Left),
            generate_single_key_bind_entry("Right", TetrisInstruction::Right),
            generate_single_key_bind_entry("Rotate Counter Clock Wise", TetrisInstruction::RotateCounter),
            generate_single_key_bind_entry("Rotate Clock Wise", TetrisInstruction::RotateClock),
            generate_single_key_bind_entry("Store Active Piece", TetrisInstruction::Store),
        ],
    )
}

/// A marker which marks the paused screen top div. 
#[derive(Component)]
pub struct PausedTopDiv;

/// Differentiates which key for a specific action is meant. The value it holds theoretically 
/// suppoers u32 many different keybinds but in practice only two are used. This is simply
/// future proofing for when unlimited keys for a single action are allowed. 
#[derive(Component)]
pub struct KeyBindClickArea(u32);

/// A marker which marks the text of the music volume. 
#[derive(Component)]
pub struct MusicVolumeTextMarker;

/// A marker which marks the button to decrease the music volume. 
#[derive(Component)]
pub struct DecreaseMusicVolumeButton;

/// A marker which marks the button to increase the music volume. 
#[derive(Component)]
pub struct IncreaseMusicVolumeButton;

/// Differentiates the existing settings categories into their tabs.  
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsTab {
    Audio, 
    KeyMapping,
}

/// A marker which marks the audio settings button. 
#[derive(Component)]
pub struct AudioButton;

/// A marker which marks the keybind settings button. 
#[derive(Component)]
pub struct KeyMappingButton;

/// A marker which marks which part of the pause menu are children that can be removed 
/// when switchting the active settings tab. 
#[derive(Component)]
pub struct PauseMenuRemovableChildren;

/// A marker which marks the text in the primary keybind button. 
#[derive(Component)]
pub struct FirstKeyBindTextMarker;

/// A marker which marks the text in the secondary keybind button. 
#[derive(Component)]
pub struct SecondKeyBindTextMarker;

/// Removes the currently selected settings tab via [PauseMenuRemovableChildren]. 
fn remove_settings_children(
    query: Query<Entity, With<PauseMenuRemovableChildren>>,
    commands: &mut Commands,
) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}

/// Implements the button functionality for selecting the audio settings tab. 
fn audio_button_listener(
    mut button_query: Query<(&Interaction, &mut bevy::ui::BackgroundColor), (Changed<Interaction>, With<Button>, With<AudioButton>)>,
    mut commands: Commands,
    mut settings_tab_query: Query<&mut SettingsTab>,
    paused_top_div_query: Query<Entity, With<PausedTopDiv>>,
    remove_settings_query: Query<Entity, With<PauseMenuRemovableChildren>>,
) {
    let Ok((interaction, mut backgroud_color)) = button_query.single_mut() else {return;};

    match interaction {
        Interaction::Pressed => {
            if let Ok(mut settings_tab) = settings_tab_query.single_mut() {
                if *settings_tab == SettingsTab::Audio {
                    return;
                }
                *settings_tab = SettingsTab::Audio;
                remove_settings_children(remove_settings_query, &mut commands);
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

/// Implements the button functionality for selecting the key mapping settings tab.  
fn key_mapping_button_listener(
    mut button_query: Query<(&Interaction, &mut bevy::ui::BackgroundColor), (Changed<Interaction>, With<Button>, With<KeyMappingButton>)>,
    mut commands: Commands,
    mut settings_tab_query: Query<&mut SettingsTab>,
    paused_top_div_query: Query<Entity, With<PausedTopDiv>>,
    remove_settings_query: Query<Entity, With<PauseMenuRemovableChildren>>,
) {
    let Ok((interaction, mut backgroud_color)) = button_query.single_mut() else {return;};

    match interaction {
        Interaction::Pressed => {
            if let Ok(mut settings_tab) = settings_tab_query.single_mut() {
                if *settings_tab == SettingsTab::KeyMapping {
                    return;
                }
                *settings_tab = SettingsTab::KeyMapping;
                remove_settings_children(remove_settings_query, &mut commands);
            } else {
                commands.spawn(SettingsTab::KeyMapping);
            }

            let Ok(entity) = paused_top_div_query.single() else {return;};

            commands.entity(entity).with_child(generate_key_bind_menu());
        }
        Interaction::Hovered => {
            *backgroud_color = HOVERED_BUTTON_BACKGROUND_COLOR;
        }
        Interaction::None => {
            *backgroud_color = EMPTY_BACKGROUND_COLOR;
        }
    }
}

/// Implements the button functionality for starting a new game. 
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

/// Implements the button functionality for a single keybind area. 
fn individual_keybind_button_listener(
    mut button_query: Query<(&TetrisInstruction, &mut BackgroundColor, &Interaction, &KeyBindClickArea), (Changed<Interaction>, With<Button>)>,
    mut commands: Commands,
    waiting_query: Query<Entity, With<WaitingForNewKeyBind>>,
    clear_all: Res<ClearKeybindClicks>,
    highlight_one: Res<HighlghtClickedKeybind>,
) {
    for (instruction, mut background_color, interaction, keybind_click_area) in &mut button_query {
        if *interaction == Interaction::Pressed{
            //remove previous waiting component
            if let Ok(entity) = waiting_query.single() {    //theoretically, no more than one should ever exist concurrently
                commands.entity(entity).despawn();
            }

            //change the color to signal to the user which field is active
            background_color.0 = Color::srgb(0.5, 0.5, 0.5);

            //spawn a struct which signals not to use button inputs
            commands.spawn(WaitingForNewKeyBind {
                primary: keybind_click_area.0,
                selected_tetris_instruction: *instruction,
            });
            commands.run_system(clear_all.0);
            commands.run_system(highlight_one.0);
        }
    }
}

/// The struct that holds the general settings of the bevy engine game: audio and 
/// keybinds. 
#[derive(Resource, Clone, Debug)]
pub struct Settings {
    pub music_volume: f32,
    pub key_binds: KeyBinds,
}

impl Settings {
    /// Tries to save the settings to the data/settings.dat file. Returns Err 
    /// if any problems appear. 
    fn write_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = std::fs::create_dir("./data");

        //get a file 
        let file = std::fs::File::create("data/settings.dat")?;

        //serialize and write to file
        serde_cbor::to_writer(file, &self)?;

        Ok(())
    }

    /// Reads the settings from the data/settings.dat file. Returns Err if
    /// any problems appear. 
    fn new_from_serialized() -> Result<Self, Box<dyn std::error::Error>> {
        //open the file
        let file = std::fs::File::open("data/settings.dat")?;

        //deserialze 
        let settings = serde_cbor::from_reader(file)?;

        Ok(settings)
    }

    /// Reads the settings from the file. If this fails (as it does when first
    /// starting the game), uses default values instead. 
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

impl Serialize for Settings {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where S: serde::Serializer {
        let mut state = serializer.serialize_struct("Settings", 2)?;    //len = nbr of fields to be serialized

        state.serialize_field("music_volume", &self.music_volume)?;
        
        let serializable_key_binds = KeyBindsSerialized::from(self.key_binds.clone());
        state.serialize_field("key_binds", &serializable_key_binds)?;

        state.end()
    }
}

impl<'de> Deserialize<'de> for Settings {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        #[derive(Deserialize, Debug)]
        struct Helper {
            music_volume: f32,
            key_binds: KeyBindsSerialized,
        }

        let helper = Helper::deserialize(deserializer)?;

        Ok(Settings {
            music_volume: helper.music_volume,
            key_binds: helper.key_binds.into(),
        })
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self { 
            music_volume: 0.5,
            key_binds: KeyBinds::default(),
        }
    }
}

/// Updates the displayed volume text. 
fn display_music_volume_settings(
    settings: Res<Settings>,
    mut text_query: Query<&mut Text, With<MusicVolumeTextMarker>>,
) {
    let Ok(mut text) = text_query.single_mut() else {return;};

    *text = Text::new(format!(" {:.0} ", settings.music_volume * 100.0));
}

/// Implements the button functionalities for the decrease and increase music volume button. s
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

/// Updates the primary key bind settings button texts with the values found in the settings. 
fn update_primary_key_bind_settings(
    primary_text_query: Query<(&TetrisInstruction, &mut Text), With<FirstKeyBindTextMarker>>,
    settings: Res<Settings>,
) {
    for (action, mut text) in primary_text_query {
        let instruction_key_bind = settings.key_binds.get(action);
        *text = Text::new(key_code_to_str(instruction_key_bind.primary_key));
    }
}

/// Updates the secondary key bind settings button texts with the values found in the settings. 
fn update_secondary_key_bind_settings(
    secondary_text_query: Query<(&TetrisInstruction, &mut Text), With<SecondKeyBindTextMarker>>,
    settings: Res<Settings>,
) {
    for (action, mut text) in secondary_text_query {
        let instruction_key_bind =  settings.key_binds.get(action);
        if let Some(secondary_key) = instruction_key_bind.secondary_key {
            *text = Text::new(key_code_to_str(secondary_key));
        } else {
            *text = Text::new("   ");
        }
    }
}

/// A struct that contains all keybinds for a specific Instruction. 
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct InstructionKeyBind {
    action: TetrisInstruction, 
    pub primary_key: KeyCode,
    pub secondary_key: Option<KeyCode>,
}

/// Differentiates which kind of instructions specific to a tetris game can give. 
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Component)]
pub enum TetrisInstruction {
    Drop, 
    FullDrop, 
    Left, 
    Right, 
    RotateCounter,
    RotateClock, 
    Store, 
}

impl InstructionKeyBind {
    fn new(instruction: TetrisInstruction, primary: KeyCode, secondary: Option<KeyCode>) -> Self {
        Self {
            action: instruction,
            primary_key: primary,
            secondary_key: secondary,
        }
    }
}

/// A struct that holds the keybinds for ALL [TetrisInstruction]s. 
#[derive(Clone, Debug)]
pub struct KeyBinds {
    key_binds: Vec<InstructionKeyBind>,
}

impl KeyBinds {
    /// Returns a copy of the [InstructionKeyBind] for the given [TetrisInstruction]. 
    pub fn get(&self, instruction: &TetrisInstruction) -> InstructionKeyBind {
        for instruction_key_bind in &self.key_binds {
            if instruction_key_bind.action == *instruction {
                return *instruction_key_bind;
            }
        }

        panic!("Illegal State. The KeyBinds struct must always feature an entry for every instruction.");
    }

    /// Returns a mutable referente of the [InstructionKeyBind] for the given [TetrisInstruction].
    pub fn get_mut(&mut self, instruction: &TetrisInstruction) -> &mut InstructionKeyBind {
        for instruction_key_bind in &mut self.key_binds {
            if instruction_key_bind.action == *instruction {
                return instruction_key_bind;
            }
        }

        panic!("Illegal State. The KeyBinds struct must always feature an entry for every instruction.");
    }
}

impl PartialEq for InstructionKeyBind {
    fn eq(&self, other: &Self) -> bool {
        self.action == other.action
    }
}

impl Default for KeyBinds {
    fn default() -> Self {
        let key_binds = vec![
            InstructionKeyBind::new(TetrisInstruction::Drop, KeyCode::KeyS, None),
            InstructionKeyBind::new(TetrisInstruction::FullDrop, KeyCode::Space, None),
            InstructionKeyBind::new(TetrisInstruction::Left, KeyCode::KeyA, None),
            InstructionKeyBind::new(TetrisInstruction::Right, KeyCode::KeyD, None),
            InstructionKeyBind::new(TetrisInstruction::RotateCounter, KeyCode::KeyQ, None),
            InstructionKeyBind::new(TetrisInstruction::RotateClock, KeyCode::KeyE, None),
            InstructionKeyBind::new(TetrisInstruction::Store, KeyCode::KeyW, None),
        ];

        Self { 
            key_binds,
        }
    }
}

/// A helper struct for deserializing [KeyBinds]. This struct can at points not have all
/// [TetrisInstruction]s logged. 
#[derive(Clone, Debug, Serialize, Deserialize)]
struct KeyBindsSerialized {
    key_binds: Vec<InstructionKeyBind>,
}

impl From<KeyBinds> for KeyBindsSerialized {
    fn from(value: KeyBinds) -> Self {
        Self {
            key_binds: value.key_binds,
        }
    }
}

impl From<KeyBindsSerialized> for KeyBinds {
    fn from(value: KeyBindsSerialized) -> Self {
        let mut list = value.key_binds;
        let mut default = KeyBinds::default();

        for elem in default.key_binds {
            if !list.contains(&elem) {
                list.push(elem);
            }
        }

        default.key_binds = list;

        default
    }
}

/// Maps a [KeyCode] to a string slice. Substitutes the Display function as it is not 
/// implemented and the debug format is not user readable. 
fn key_code_to_str(key_code: KeyCode) -> &'static str {
    match key_code {
        KeyCode::KeyA => "A",
        KeyCode::KeyB => "B",
        KeyCode::KeyC => "C",
        KeyCode::KeyD => "D",
        KeyCode::KeyE => "E",
        KeyCode::KeyF => "F",
        KeyCode::KeyG => "G",
        KeyCode::KeyH => "H",
        KeyCode::KeyI => "I",
        KeyCode::KeyJ => "J",
        KeyCode::KeyK => "K",
        KeyCode::KeyL => "L",
        KeyCode::KeyM => "M",
        KeyCode::KeyN => "N",
        KeyCode::KeyO => "O",
        KeyCode::KeyP => "P",
        KeyCode::KeyQ => "Q",
        KeyCode::KeyR => "R",
        KeyCode::KeyS => "S",
        KeyCode::KeyT => "T",
        KeyCode::KeyU => "U",
        KeyCode::KeyV => "V",
        KeyCode::KeyW => "W",
        KeyCode::KeyX => "X",
        KeyCode::KeyY => "Y",
        KeyCode::KeyZ => "Z",

        KeyCode::Digit0 => "0",
        KeyCode::Digit1 => "1",
        KeyCode::Digit2 => "2",
        KeyCode::Digit3 => "3",
        KeyCode::Digit4 => "4",
        KeyCode::Digit5 => "5",
        KeyCode::Digit6 => "6",
        KeyCode::Digit7 => "7",
        KeyCode::Digit8 => "8",
        KeyCode::Digit9 => "9",

        KeyCode::Escape => "Esc",
        KeyCode::Tab => "Tab",
        KeyCode::CapsLock => "Caps",
        KeyCode::ShiftLeft => "L Shift",
        KeyCode::ControlLeft => "L Ctrl",
        KeyCode::AltLeft => "L Alt",
        KeyCode::Space => "Space",
        KeyCode::ContextMenu => "Context Menu",
        KeyCode::ControlRight => "R Control",
        KeyCode::ShiftRight => "R Shift",
        KeyCode::Enter => "Enter",
        KeyCode::Backspace => "Backspace",
        //TODO more missing KeyCodes

        KeyCode::ArrowUp => "Arrow Up",
        KeyCode::ArrowDown => "Arrow Down",
        KeyCode::ArrowLeft => "Arrow Left",
        KeyCode::ArrowRight => "Arrow Right",
        KeyCode::Delete => "Delete",
        KeyCode::End => "End",
        KeyCode::PageDown => "Page Down",
        KeyCode::PageUp => "Page Up",
        KeyCode::Insert => "Insert",
        KeyCode::Home => "Home",
        KeyCode::PrintScreen => "Print",
        KeyCode::ScrollLock => "Scroll Lock",
        KeyCode::Pause => "Pause",

        KeyCode::F1 => "F1",
        KeyCode::F2 => "F2",
        KeyCode::F3 => "F3",
        KeyCode::F4 => "F4",
        KeyCode::F5 => "F5",
        KeyCode::F6 => "F6",
        KeyCode::F7 => "F7",
        KeyCode::F8 => "F8",
        KeyCode::F9 => "F9",
        KeyCode::F10 => "F10",
        KeyCode::F11 => "F11",
        KeyCode::F12 => "F12",
        KeyCode::F13 => "F13",
        KeyCode::F14 => "F14",
        KeyCode::F15 => "F15",
        KeyCode::F16 => "F16",
        KeyCode::F17 => "F17",
        KeyCode::F18 => "F18",
        KeyCode::F19 => "F19",
        KeyCode::F20 => "F20",
        KeyCode::F21 => "F21",
        KeyCode::F22 => "F22",
        KeyCode::F23 => "F23",
        KeyCode::F24 => "F24",
        KeyCode::F25 => "F25",
        KeyCode::F26 => "F26",
        KeyCode::F27 => "F27",
        KeyCode::F28 => "F28",
        KeyCode::F29 => "F28",
        KeyCode::F30 => "F30",
        KeyCode::F31 => "F31",
        KeyCode::F32 => "F32",
        KeyCode::F33 => "F33",
        KeyCode::F34 => "F34",
        KeyCode::F35 => "F35",

        KeyCode::Numpad0 => "Num 0",
        KeyCode::Numpad1 => "Num 1",
        KeyCode::Numpad2 => "Num 2",
        KeyCode::Numpad3 => "Num 3",
        KeyCode::Numpad4 => "Num 4",
        KeyCode::Numpad5 => "Num 5",
        KeyCode::Numpad6 => "Num 6",
        KeyCode::Numpad7 => "Num 7",
        KeyCode::Numpad8 => "Num 8",
        KeyCode::Numpad9 => "Num 9",
        KeyCode::NumpadAdd => "Numpad Add",
        KeyCode::NumLock => "Numpad Lock",
        KeyCode::NumpadSubtract => "Numpad Minus",
        KeyCode::NumpadComma => "Numpad Comma",
        KeyCode::NumpadMultiply => "Numpad Multiply",
        KeyCode::NumpadDivide => "Numpad Divide",

        _ => "Unknown Key",
    }
}

/// An entity which signals that the game is currently waiting for a button input from 
/// the user to save as a new keybind. 
#[derive(Component, Clone, Copy, Debug)]
pub struct WaitingForNewKeyBind {
    pub primary: u32,
    pub selected_tetris_instruction: TetrisInstruction,
}
