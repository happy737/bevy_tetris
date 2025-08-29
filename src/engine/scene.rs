use std::collections::HashMap;
use std::time::Duration;

use bevy::audio::Volume;
use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;

use crate::engine;
use crate::engine::line_stuff::LineListIndex;
use crate::engine::line_stuff::LineMaterial;
use crate::engine::model::CellStatus;
use crate::ui::Settings;
use crate::ui::TetrisInstruction;
use crate::ui::WaitingForNewKeyBind;

const ONE_LINE_SCORE: u32 = 100;
const TWO_LINE_SCORE: u32 = 300;
const THREE_LINE_SCORE: u32 = 500;
const FOUR_LINE_SCORE: u32 = 800;
const SLOW_DROP_SCORE: u32 = 1;
const FAST_DROP_SCORE: u32 = 2;

const BASE_DROP_DURATION_SECS: f64 = 1.0;
const DIFFICULTY: u32 = 1;  //TODO should always be 1

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(engine::line_stuff::LineStuffPlugin);
        app.add_systems(Startup, setup);
        app.add_systems(Update, display_game_state);
        app.add_systems(Update, update_game_state);
        app.add_systems(Update, display_next_piece);
        app.add_systems(Update, display_stored_piece);
        app.add_systems(Update, display_ghost_piece);
        app.add_systems(Update, update_audio);
        app.add_systems(Update, manage_pause);
    }
}

fn setup(mut commands: Commands, 
         mut meshes: ResMut<Assets<Mesh>>, 
         mut materials: ResMut<Assets<StandardMaterial>>, 
         mut materials_line: ResMut<Assets<LineMaterial>>,
         mut clear_color: ResMut<ClearColor>,
         asset_server: Res<AssetServer>
        ) {
    //colorful materials for each block color
    let mut material_map = HashMap::new();
    material_map.insert(CellStatus::Cyan, materials.add(Color::srgb(0.0, 1.0, 1.0)));
    material_map.insert(CellStatus::Yellow, materials.add(Color::srgb(1.0, 1.0, 0.0)));
    material_map.insert(CellStatus::Purple, materials.add(Color::srgb(1.0, 0.0, 1.0)));
    material_map.insert(CellStatus::Green, materials.add(Color::srgb(0.0, 1.0, 0.0)));
    material_map.insert(CellStatus::Red, materials.add(Color::srgb(1.0, 0.0, 0.0)));
    material_map.insert(CellStatus::Blue, materials.add(Color::srgb(0.0, 0.0, 1.0)));
    material_map.insert(CellStatus::Orange, materials.add(Color::srgb(1.0, 0.5, 0.0)));

    commands.insert_resource(MaterialsHandle(material_map));

    //line material
    commands.insert_resource(LineMaterialHandle(materials_line.add(LineMaterial {color: LinearRgba::WHITE})));

    //load mesh of a cube
    commands.insert_resource(CubeHandle(meshes.add(Cuboid::new(1.0, 1.0, 1.0))));

    let line_cube_handle = meshes.add(LineListIndex::cube());

    //load mesh of line cube
    commands.insert_resource(LineCubeHandle(line_cube_handle.clone()));

    //tetris model
    commands.spawn((
        Game::default(),
    ));

    //score of the game
    commands.insert_resource(GameScore::default());

    //timer that repeatedly drops active tetromino
    commands.insert_resource(DropTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));

    //flag that is set to recolor existing cubes
    commands.insert_resource(RecolorCubes(false));

    //camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    //make background black
    clear_color.0 = BLACK.into();

    //light source
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 6.0),
    ));

    //line cube
    commands.spawn((
        Mesh3d(line_cube_handle.clone()),
        MeshMaterial3d(materials_line.add(LineMaterial{color: LinearRgba::WHITE})),
        Transform::from_scale(Vec3::new(5.0, 10.0, 0.5)),
    ));

    //next piece line cube
    commands.spawn((
        Mesh3d(line_cube_handle.clone()),
        MeshMaterial3d(materials_line.add(LineMaterial{color: LinearRgba::WHITE})),
        Transform::from_scale(Vec3::new(2.0, 1.0, 0.5)).with_translation(Vec3::new(12.0, 16.0, 0.0) - Vec3::new(4.0, 10.5, 0.0)),
    ));

    //stored piece line cube
    commands.spawn((
        Mesh3d(line_cube_handle.clone()),
        MeshMaterial3d(materials_line.add(LineMaterial{color: LinearRgba::WHITE})),
        Transform::from_scale(Vec3::new(2.0, 1.0, 0.5)).with_translation(Vec3::new(12.0, 12.0, 0.0) - Vec3::new(4.0, 10.5, 0.0)),
    ));

    //start music
    commands.spawn((
        AudioPlayer::new(asset_server.load("music/Tetris.ogg")),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            ..Default::default()
        },
    ));

    //add a struct which will prematurely end all functions working with a game overed game
    commands.insert_resource(IsAppRunning(AppState::Running));

    //center dividing line
    // commands.spawn((
    //     Mesh3d(meshes.add(LineListIndex{
    //         points: vec![Vec3::new(0.0, 50.0, 0.0), Vec3::new(0.0, -50.0, 0.0)],
    //         indices: vec![0, 1],
    //     })),
    //     MeshMaterial3d(materials_line.add(LineMaterial{color: LinearRgba::WHITE})),
    //     Transform::IDENTITY,
    // ));
}

#[derive(Clone, Component, Debug, Default)]
pub struct Game {
    pub tetris: engine::model::Tetris<rand::rngs::OsRng>,
}

fn display_game_state(
        mut commands: Commands, 
        game_query: Query<&Game>, 
        mut cubes_query: Query<(Entity, &CellPosition, &mut MeshMaterial3d<StandardMaterial>), With<MainPixelMarker>>,
        cube_handle: Res<CubeHandle>,
        material_handles: Res<MaterialsHandle>,
        mut update_cube_color: ResMut<RecolorCubes>,
        running: Res<IsAppRunning>,
    ) {
    if !(running.0 == AppState::Running) {
        return;
    }
    if game_query.is_empty() {
        error!("Game is missing!");
        return;
    }
    let game = game_query.into_iter().next().unwrap();
    
    //main cubes

    //get all the cubes the system is currently displaying
    let mut existing_cubes = cubes_query
        .iter_mut()
        .fold(HashMap::new(), |mut map, (entity, pos, material)| {map.insert(pos, (entity, material)); map});

    //get all the positions where cubes should be. If one is missing, spawn it
    for (cell, x, y) in game.tetris.get_block_list() {
        let pos = CellPosition::new(x as i32, y as i32);

        if let Some((_, material)) = &mut existing_cubes.remove(&pos) {
            //if necessary flag is set, re assign every material. 
            // this prevents a bug where immediately dropping a piece will mis-color some cubes of the following piece
            if update_cube_color.0 {
                material.0 = material_handles.0[&cell].clone();
            }
        } else {
            //spawn new cube
            let material_handle = &material_handles.0[&cell];

            commands.spawn((
                Mesh3d(cube_handle.0.clone()),
                MeshMaterial3d(material_handle.clone()),
                Transform::from_translation(Vec3::from(pos) - Vec3::new(4.5, 9.5, 0.0)),
                pos,
                MainPixelMarker,
            ));
        }
    }

    //all remaining cubes are at positions where nothing should be, remove them
    for (_, (entity, _)) in existing_cubes.into_iter() {
        commands.entity(entity).despawn();
    }

    update_cube_color.0 = false;
}

fn display_next_piece(
    mut commands: Commands, 
    game_query: Query<&Game>, 
    mut next_cubes_query: Query<(Entity, &CellPosition, &mut MeshMaterial3d<StandardMaterial>), With<NextPixelMarker>>,
    cube_handle: Res<CubeHandle>,
    material_handles: Res<MaterialsHandle>,
    running: Res<IsAppRunning>,
) {
    if !(running.0 == AppState::Running) {
        return;
    }

    if game_query.is_empty() {
        error!("Game is missing!");
        return;
    }
    let game = game_query.into_iter().next().unwrap();
    
    //next piece cubes

    //get all the cubes the system is currently displaying
    let mut existing_next_cubes = next_cubes_query
        .iter_mut()
        .fold(HashMap::new(), |mut map, (entity, pos, material)| {map.insert(pos, (entity, material)); map});

    //get all the positions where cubes should be. If one is missing, spawn it
    for (cell, x, y) in game.tetris.get_next_block_list() {
        let pos = CellPosition::new(x as i32, y as i32);

        if let Some((_, material)) = &mut existing_next_cubes.remove(&pos) {
            //if necessary flag is set, re assign every material. 
            material.0 = material_handles.0[&cell].clone();
        } else {
            //spawn new cube
            let material_handle = &material_handles.0[&cell];

            commands.spawn((
                Mesh3d(cube_handle.0.clone()),
                MeshMaterial3d(material_handle.clone()),
                Transform::from_translation(Vec3::from(pos) - Vec3::new(4.5, 10.0, 0.0) + Vec3::new(11.0, 15.0, 0.0)),
                pos,
                NextPixelMarker,
            ));
        }
    }
    //all remaining cubes are at positions where nothing should be, remove them
    for (_, (entity, _)) in existing_next_cubes.into_iter() {
        commands.entity(entity).despawn();
    }
}

fn display_stored_piece(
    mut commands: Commands, 
    game_query: Query<&Game>, 
    mut stored_cubes_query: Query<(Entity, &CellPosition, &mut MeshMaterial3d<StandardMaterial>), With<StoredPixelMarker>>,
    cube_handle: Res<CubeHandle>,
    material_handles: Res<MaterialsHandle>,
    running: Res<IsAppRunning>,
) {
    if !(running.0 == AppState::Running) {
        return;
    }

    if game_query.is_empty() {
        error!("Game is missing!");
        return;
    }
    let game = game_query.into_iter().next().unwrap();
    
    //next piece cubes

    //get all the cubes the system is currently displaying
    let mut existing_stored_cubes = stored_cubes_query
        .iter_mut()
        .fold(HashMap::new(), |mut map, (entity, pos, material)| {map.insert(pos, (entity, material)); map});

    //get all the positions where cubes should be. If one is missing, spawn it
    for (cell, x, y) in game.tetris.get_stored_block_list() {
        let pos = CellPosition::new(x as i32, y as i32);

        if let Some((_, material)) = &mut existing_stored_cubes.remove(&pos) {
            //if necessary flag is set, re assign every material. 
            material.0 = material_handles.0[&cell].clone();
        } else {
            //spawn new cube
            let material_handle = &material_handles.0[&cell];

            commands.spawn((
                Mesh3d(cube_handle.0.clone()),
                MeshMaterial3d(material_handle.clone()),
                Transform::from_translation(Vec3::from(pos) - Vec3::new(4.5, 10.0, 0.0) + Vec3::new(11.0, 11.0, 0.0)),
                pos,
                StoredPixelMarker,
            ));
        }
    }
    //all remaining cubes are at positions where nothing should be, remove them
    for (_, (entity, _)) in existing_stored_cubes.into_iter() {
        commands.entity(entity).despawn();
    }
}

fn display_ghost_piece(
    mut commands: Commands, 
    game_query: Query<&Game>, 
    mut ghost_cubes_query: Query<(Entity, &CellPosition, &mut MeshMaterial3d<LineMaterial>), With<GhostPixelMarker>>,
    mut line_cube_handle: ResMut<LineCubeHandle>,
    line_material_handle: Res<LineMaterialHandle>,
    running: Res<IsAppRunning>,

    mut meshes: ResMut<Assets<Mesh>>, 
) {
    if !(running.0 == AppState::Running) {
        return;
    }

    let game = game_query.into_iter().next().unwrap();
    
    //next piece cubes

    //get all the cubes the system is currently displaying
    let mut existing_ghost_cubes = ghost_cubes_query
        .iter_mut()
        .fold(HashMap::new(), |mut map, (entity, pos, material)| {map.insert(pos, (entity, material)); map});

    //get all the positions where cubes should be. If one is missing, spawn it
    for (x, y) in game.tetris.get_ghost_piece_list() {
        let pos = CellPosition::new(x as i32, y as i32);

        if let Some((_, material)) = &mut existing_ghost_cubes.remove(&pos) {
            material.0 = line_material_handle.0.clone();
        } else {
            //spawn new cube
            let material_handle = &line_material_handle.0;

            if meshes.get(&line_cube_handle.0.clone()).is_none() {  //TODO THIS BUG IS SHIT
                warn!("Line Cube Mesh has been unloaded. Reloading it");
                line_cube_handle.0 = meshes.add(LineListIndex::cube());
            }

            commands.spawn((
                Mesh3d(line_cube_handle.0.clone()),
                //Mesh3d(meshes.add(LineListIndex::cube())),
                MeshMaterial3d(material_handle.clone()),
                Transform::from_scale(Vec3::new(0.5, 0.5, 0.5))
                    .with_translation(Vec3::from(pos) - Vec3::new(4.5, 9.5, 0.0)),
                pos,
                GhostPixelMarker,
            ));
        }
    }
    //all remaining cubes are at positions where nothing should be, remove them
    for (_, (entity, _)) in existing_ghost_cubes.into_iter() {
        commands.entity(entity).despawn();
    }
}

fn manage_pause(
    mut app_state: ResMut<IsAppRunning>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    show_paused_menu: Res<crate::ui::SpawnPauseSystem>,
    mut commands: Commands,
    previous_state_query: Query<(&GamePausedPreviousState, Entity)>,
    paused_top_div_query: Query<Entity, With<crate::ui::PausedTopDiv>>,
    settings_tap_query: Query<Entity, With<crate::ui::SettingsTab>>,
) {
    let state = app_state.0;
    //Game is currently not paused
    if state == AppState::Running {
        if keyboard_input.just_pressed(KeyCode::Escape) {
            app_state.0 = AppState::Paused;
            commands.run_system(show_paused_menu.0);
            commands.spawn(GamePausedPreviousState(state));
        }
        return;
    }
    if state == AppState::GameOver {
        return;
    }

    //game is currently paused

    //end pause
    if keyboard_input.just_pressed(KeyCode::Escape) {
        let Ok((previous_state, entity)) = previous_state_query.single() else {return;};
        app_state.0 = previous_state.0;
        commands.entity(entity).despawn();

        let Ok(top_div_entity) = paused_top_div_query.single() else {
            error!("Couldn't find and remove the top div of the paused screen!");
            return;
        };
        commands.entity(top_div_entity).despawn();

        for entity in settings_tap_query {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component)]
struct GamePausedPreviousState(AppState);

fn update_game_state(
        game_query: Query<&mut Game>, 
        time: Res<Time>, 
        mut timer: ResMut<DropTimer>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut update_cube_color: ResMut<RecolorCubes>,
        mut game_score: ResMut<GameScore>,
        mut running: ResMut<IsAppRunning>,
        mut settings: ResMut<Settings>,
        assigning_keybind_query: Query<(Entity, &WaitingForNewKeyBind)>,

        mut commands: Commands, 
        show_game_over: Res<crate::ui::SpawnGameOverSystem>,
        clear_keybind_clicks: Res<crate::ui::ClearKeybindClicks>,
    ) {
    let key_binds = &mut settings.key_binds;
    if let Ok((entity, waiting)) = assigning_keybind_query.single() {
        if let Some(key) = keyboard_input.get_just_pressed().next() {
            if *key != KeyCode::Escape {
                match waiting.primary {
                    1 => {
                        key_binds.get_mut(&waiting.selected_tetris_instruction).primary_key = *key;
                    }
                    2 => {
                        key_binds.get_mut(&waiting.selected_tetris_instruction).secondary_key = Some(*key);
                    }
                    x => {
                        error!("Illegal state, no more than two keys per action allows. Expected value 1 or 2, got: {x}!");
                        panic!();
                    }
                }
            }

            commands.entity(entity).despawn();
            commands.run_system(clear_keybind_clicks.0);
        }

        return;
    }
    
    if !(running.0 == AppState::Running) {
        return;
    }

    if game_query.is_empty() {
        error!("Game is missing!");
        return;
    }
    let mut game = game_query.into_iter().next().unwrap();

    //check if piece drops automatically
    if timer.0.tick(time.delta()).just_finished() {
        let result = game.tetris.drop();
        if let Ok((false, Some(nbr_of_lines))) = result {
            let add_score = match nbr_of_lines {
                0 => {0}
                1 => {ONE_LINE_SCORE}
                2 => {TWO_LINE_SCORE}
                3 => {THREE_LINE_SCORE}
                4 => {FOUR_LINE_SCORE}
                _ => {
                    error!("Unexpected number of lines cleared after drop to bottom. Expected range: [0; 4], Actual: {nbr_of_lines}");
                    panic!();
                }
            };
            game_score.change(add_score, nbr_of_lines);
        } else if result.is_err() {
            running.0 = AppState::GameOver;
            commands.run_system(show_game_over.0);
        }
        update_cube_color.0 = true;

        timer.0.set_duration(Duration::from_secs_f64(level_to_drop_duration(game_score.level)));
    }

    //check for moving left
    if keyboard_input.just_pressed(key_binds.get(&TetrisInstruction::Left).primary_key) ||
            key_binds.get(&TetrisInstruction::Left).secondary_key.is_some_and(|k| keyboard_input.just_pressed(k)) {
        let _ = game.tetris.try_left();
    }

    //check for moving right
    if keyboard_input.just_pressed(key_binds.get(&TetrisInstruction::Right).primary_key) ||
            key_binds.get(&TetrisInstruction::Right).secondary_key.is_some_and(|k| keyboard_input.just_pressed(k)) {
        let _ = game.tetris.try_right();
    }

    //drop one level
    if keyboard_input.just_pressed(key_binds.get(&TetrisInstruction::Drop).primary_key) ||
            key_binds.get(&TetrisInstruction::Drop).secondary_key.is_some_and(|k| keyboard_input.just_pressed(k)) {
        let result = game.tetris.drop();
        if let Ok((false, Some(nbr_of_lines))) = result {
            let add_score = match nbr_of_lines {
                0 => {0}
                1 => {ONE_LINE_SCORE}
                2 => {TWO_LINE_SCORE}
                3 => {THREE_LINE_SCORE}
                4 => {FOUR_LINE_SCORE}
                _ => {
                    error!("Unexpected number of lines cleared after drop to bottom. Expected range: [0; 4], Actual: {nbr_of_lines}");
                    panic!();
                }
            };
            game_score.change(add_score, nbr_of_lines);
        } else if result.is_err() {
            running.0 = AppState::GameOver;
            commands.run_system(show_game_over.0);
        }
        game_score.change(SLOW_DROP_SCORE, 0);

        timer.0.reset();
        update_cube_color.0 = true;
    }

    //drop all the way down 
    if keyboard_input.just_pressed(key_binds.get(&TetrisInstruction::FullDrop).primary_key) ||
            key_binds.get(&TetrisInstruction::FullDrop).secondary_key.is_some_and(|k| keyboard_input.just_pressed(k)) {
        let result = game.tetris.drop_completely_down();
        if result.is_err() {
            running.0 = AppState::GameOver;
            commands.run_system(show_game_over.0);
        } else {
            let (nbr_of_dropped_cells, nbr_of_cleared_lines) = result.unwrap();
            let mut add_score = match nbr_of_cleared_lines {
                0 => {0}
                1 => {ONE_LINE_SCORE}
                2 => {TWO_LINE_SCORE}
                3 => {THREE_LINE_SCORE}
                4 => {FOUR_LINE_SCORE}
                _ => {
                    error!("Unexpected number of lines cleared after drop to bottom. Expected range: [0; 4], Actual: {nbr_of_cleared_lines}");
                    panic!();
                }
            };
            add_score += nbr_of_dropped_cells * FAST_DROP_SCORE;
            game_score.change(add_score, nbr_of_cleared_lines);
        }

        update_cube_color.0 = true;
    }

    //spin active piece counterclockwise
    if keyboard_input.just_pressed(key_binds.get(&TetrisInstruction::RotateCounter).primary_key) ||
            key_binds.get(&TetrisInstruction::RotateCounter).secondary_key.is_some_and(|k| keyboard_input.just_pressed(k)) {
        game.tetris.spin_counter_90();
    }

    //spin active piece clockwise
    if keyboard_input.just_pressed(key_binds.get(&TetrisInstruction::RotateClock).primary_key) ||
            key_binds.get(&TetrisInstruction::RotateClock).secondary_key.is_some_and(|k| keyboard_input.just_pressed(k)) {
        game.tetris.spin_clock_90();
    }

    //switch active peace with stored piece
    if keyboard_input.just_pressed(key_binds.get(&TetrisInstruction::Store).primary_key) ||
            key_binds.get(&TetrisInstruction::Store).secondary_key.is_some_and(|k| keyboard_input.just_pressed(k)) {
        let _ = game.tetris.try_switch_active_piece();
        update_cube_color.0 = true;
    }
}

fn update_audio(
    mut audio_query: Query<&mut AudioSink>,
    score: Res<GameScore>,
    running: Res<IsAppRunning>,
    settings: Res<crate::ui::Settings>,
) {
    match running.0 {
        AppState::Running => {
            let Ok(mut sink) = audio_query.single_mut() else {return};

            let mut speed = 1.0;
            if score.level >= 10 {
                speed = 1.2;
            }
            if score.level >= 15 {
                speed = (1.4 + 0.2 * (score.level - 15) as f32).max(2.4);
            } 

            sink.set_speed(speed);
            sink.set_volume(Volume::Linear(settings.music_volume));
        }
        AppState::Paused => {
            let Ok(mut sink) = audio_query.single_mut() else {return};
            sink.set_speed(0.5);
            sink.set_volume(Volume::Linear(settings.music_volume));
        }
        _ => {}
    }    
}

/////////////////// HERE THE HELPER FUNCTIONS AND STRUCTS START /////////////////////////

fn level_to_drop_duration(level: u32) -> f64 {
    let level = level as f64;

    (BASE_DROP_DURATION_SECS - level * 0.05).max(0.05)
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct CellPosition {
    x: i32, 
    y: i32,
}

impl CellPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x, 
            y,
        }
    }
}

impl From<CellPosition> for Vec3 {
    fn from(value: CellPosition) -> Self {
        Vec3 { 
            x: value.x as f32, 
            y: value.y as f32, 
            z: 0.0,
        }
    }
}

#[derive(Resource)]
struct CubeHandle(Handle<Mesh>);

#[derive(Resource)]
struct LineCubeHandle(Handle<Mesh>);

#[derive(Resource)]
struct MaterialsHandle(HashMap<engine::model::CellStatus, Handle<StandardMaterial>>);

#[derive(Resource)]
struct LineMaterialHandle(Handle<LineMaterial>);

#[derive(Resource)]
struct DropTimer(Timer);

#[derive(Resource)]
struct RecolorCubes(bool);

#[derive(Component)]
struct MainPixelMarker;

#[derive(Component)]
struct NextPixelMarker;

#[derive(Component)]
struct StoredPixelMarker;

#[derive(Component)]
struct GhostPixelMarker;

#[derive(Debug, Default, Resource)]
pub struct GameScore {
    pub score: u32,
    pub level: u32,
    cleared_lines: u32,
}

impl GameScore {
    fn change(&mut self, add_score: u32, add_lines: u32) {
        self.score += add_score;
        self.cleared_lines += add_lines;
        self.level = (self.cleared_lines * DIFFICULTY) / 10;
    }
}

#[derive(Resource)]
pub(crate) struct IsAppRunning(pub AppState);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppState {
    Running, 
    GameOver, 
    Paused,
}
