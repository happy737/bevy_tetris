use bevy::{ecs::system::entity_command::clear, prelude::*};
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

#[derive(Resource)]
pub struct ClearKeybindClicks(pub SystemId);

#[derive(Resource)]
pub struct HighlghtClickedKeybind(pub SystemId);

fn clear_keybind_clicks(
    backgrounds_query: Query<&mut BackgroundColor, (With<TetrisInstruction>, With<Button>)>,
) {
    for mut background in backgrounds_query {
        *background = EMPTY_BACKGROUND_COLOR;
    }
}

fn highlight_clicked_keybind(
    backgrounds_query: Query<(&mut BackgroundColor, &TetrisInstruction, &KeyBindClickArea), With<Button>>,
    waiting_query: Query<(&WaitingForNewKeyBind, Entity)>,
    mut commands: Commands,
) {
    let Ok((waiting, entity)) = waiting_query.single() else {
        error!("Called 'highlight_clicked_keybind' with no or too many selected clicked keybinds!");
        return;
    };

    for (mut background, instruction, number) in backgrounds_query {
        if *instruction == waiting.selected_tetris_instruction && number.0 == waiting.primary {
            *background = BackgroundColor(Color::srgb(0.5, 0.5, 0.5));
        }
    }

    commands.entity(entity).despawn();
}

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

#[derive(Component)]
pub struct PausedTopDiv;

#[derive(Component)]
pub struct KeyBindClickArea(u32);

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

#[derive(Component)]
pub struct PauseMenuRemovableChildren;

#[derive(Component)]
pub struct FirstKeyBindTextMarker;

#[derive(Component)]
pub struct SecondKeyBindTextMarker;

fn remove_settings_children(
    query: Query<Entity, With<PauseMenuRemovableChildren>>,
    commands: &mut Commands,
) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}

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

fn individual_keybind_button_listener(
    mut button_query: Query<(&TetrisInstruction, &mut BackgroundColor, &Interaction, &KeyBindClickArea), (Changed<Interaction>, With<Button>)>,
    mut commands: Commands,
    waiting_query: Query<Entity, With<WaitingForNewKeyBind>>,
    clear_all: Res<ClearKeybindClicks>,
    highlight_one: Res<HighlghtClickedKeybind>,
) {
    for (instruction, mut background_color, interaction, keybind_click_area) in &mut button_query {
        if *interaction == Interaction::Pressed{
            info!("Clicked!");

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

#[derive(Resource, Clone, Debug)]
pub struct Settings {
    pub music_volume: f32,
    pub key_binds: KeyBinds,
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

fn update_primary_key_bind_settings(
    primary_text_query: Query<(&TetrisInstruction, &mut Text), With<FirstKeyBindTextMarker>>,
    //secondary_text_query: Query<(&TetrisInstruction, &mut Text), (With<SecondKeyBindTextMarker>, Without<FirstKeyBindTextMarker>)>,
    settings: Res<Settings>,
) {
    for (action, mut text) in primary_text_query {
        let instruction_key_bind = settings.key_binds.get(action);
        *text = Text::new(key_code_to_str(instruction_key_bind.primary_key));
    }
}

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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct InstructionKeyBind {
    action: TetrisInstruction, 
    primary_key: KeyCode,
    secondary_key: Option<KeyCode>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Component)]
enum TetrisInstruction {
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

#[derive(Clone, Debug)]
pub struct KeyBinds {
    key_binds: Vec<InstructionKeyBind>,
}

impl KeyBinds {
    pub fn get(&self, instruction: &TetrisInstruction) -> InstructionKeyBind {
        for instruction_key_bind in &self.key_binds {
            if instruction_key_bind.action == *instruction {
                return *instruction_key_bind;
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

#[derive(Component, Clone, Copy, Debug)]
struct WaitingForNewKeyBind {
    primary: u32,
    selected_tetris_instruction: TetrisInstruction,
}
