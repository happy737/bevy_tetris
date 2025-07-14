use std::collections::HashMap;

use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;

use crate::engine;
use crate::engine::line_stuff::LineListIndex;
use crate::engine::line_stuff::LineMaterial;
use crate::engine::model::CellStatus;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(engine::line_stuff::LineStuffPlugin);
        app.add_systems(Startup, setup);
        app.add_systems(Update, display_game_state);
        app.add_systems(Update, update_game_state);
    }
}

fn setup(mut commands: Commands, 
         mut meshes: ResMut<Assets<Mesh>>, 
         mut materials: ResMut<Assets<StandardMaterial>>, 
         mut materials_line: ResMut<Assets<LineMaterial>>,
         mut clear_color: ResMut<ClearColor>,
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

    //load mesh of a cube
    commands.insert_resource(CubeHandle(meshes.add(Cuboid::new(1.0, 1.0, 1.0))));

    //tetris model
    commands.spawn((
        Game::default(),
    ));

    //timer that repeatedly drops active tetromino
    commands.insert_resource(DropTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));

    //camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    clear_color.0 = BLACK.into();

    //light source
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -6.0),
    ));

    //line cube
    commands.spawn((
        Mesh3d(meshes.add(LineListIndex::cube())),
        MeshMaterial3d(materials_line.add(LineMaterial{color: LinearRgba::WHITE})),
        Transform::from_scale(Vec3::new(5.0, 10.0, 0.5)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(LineListIndex{
            points: vec![Vec3::new(0.0, 50.0, 0.0), Vec3::new(0.0, -50.0, 0.0)],
            indices: vec![0, 1],
        })),
        MeshMaterial3d(materials_line.add(LineMaterial{color: LinearRgba::WHITE})),
        Transform::IDENTITY,
    ));
}

#[derive(Clone, Component, Debug, Default)]
pub struct Game {
    pub tetris: engine::model::Tetris<rand::rngs::OsRng>,
}

fn display_game_state(
        mut commands: Commands, 
        game_query: Query<&Game>, 
        cubes_query: Query<(Entity, &CellPosition)>,
        cube_handle: Res<CubeHandle>,
        material_handles: Res<MaterialsHandle>,
    ) {
    if game_query.is_empty() {
        error!("Game is missing!");
        return;
    }
    let game = game_query.into_iter().next().unwrap();
    
    //get all the cubes the system is currently displaying
    let mut existing_cubes = cubes_query
        .iter()
        .fold(HashMap::new(), |mut map, (entity, pos)| {map.insert(pos, entity); map});

    //get all the positions where cubes should be. If one is missing, spawn it
    for (cell, x, y) in game.tetris.get_block_list() {
        let pos = CellPosition::new(x as i32, y as i32);

        if existing_cubes.remove(&pos).is_none() {
            //spawn new cube
            let material_handle = &material_handles.0[&cell];

            commands.spawn((
                Mesh3d(cube_handle.0.clone()),
                MeshMaterial3d(material_handle.clone()),
                Transform::from_translation(Vec3::from(pos) - Vec3::new(4.5, 10.0, 0.0)),
                pos,
            ));
        }
    }

    //all remaining cubes are at positions where nothing should be, remove them
    for (_, entity) in existing_cubes.into_iter() {
        commands.entity(entity).despawn();
    }
}

fn update_game_state(
        game_query: Query<&mut Game>, 
        time: Res<Time>, 
        mut timer: ResMut<DropTimer>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
    ) {
    if game_query.is_empty() {
        error!("Game is missing!");
        return;
    }
    let mut game = game_query.into_iter().next().unwrap();

    //check if piece draps automatically
    if timer.0.tick(time.delta()).just_finished() {
        game.tetris.drop();
    }

    //check for moving left
    if keyboard_input.just_pressed(KeyCode::KeyA) {
        let _ = game.tetris.try_left();
    }

    //check for moving right
    if keyboard_input.just_pressed(KeyCode::KeyD) {
        let _ = game.tetris.try_right();
    }

    //drop one level
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        game.tetris.drop();
    }

    //drop all the way down 
    if keyboard_input.just_pressed(KeyCode::Space) {
        game.tetris.drop_completely_down();
    }

    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        game.tetris.spin_counter_90();
    }

    if keyboard_input.just_pressed(KeyCode::KeyE) {
        game.tetris.spin_clock_90();
    }
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
struct MaterialsHandle(HashMap<engine::model::CellStatus, Handle<StandardMaterial>>);

#[derive(Resource)]
struct DropTimer(Timer);