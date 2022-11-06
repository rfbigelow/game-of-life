use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};

use rand::prelude::*;

use std::collections::HashMap;
use std::collections::HashSet;

const CELL_SIZE: i16 = 1;
const INITIAL_GRID_DIM: i16 = 512;
const WORLD_RADIUS: f32 = 1_000_000.0;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Init,
    Running,
    Paused
}

struct GameRules {
    lower: u8,
    upper: u8,
}

enum Direction {
    NorthWest,
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Component)]
#[component(storage = "SparseSet")]
struct GridPosition {
    x: i16,
    y: i16,
}

#[derive(Default)]
struct GridState {
    neighbors: HashMap<GridPosition, u8>,
    cells: HashSet<GridPosition>,
}

impl GridPosition {
    fn offset(&self, direction: &Direction) -> GridPosition {
        match direction {
            Direction::NorthWest => GridPosition { x: self.x - CELL_SIZE, y: self.y + CELL_SIZE },
            Direction::North => GridPosition { x: self.x, y: self.y + CELL_SIZE },
            Direction::NorthEast => GridPosition { x: self.x + CELL_SIZE, y: self.y + CELL_SIZE },
            Direction::East => GridPosition { x: self.x + CELL_SIZE, y: self.y },
            Direction::SouthEast => GridPosition { x: self.x + CELL_SIZE, y: self.y - CELL_SIZE },
            Direction::South => GridPosition { x: self.x, y: self.y - CELL_SIZE },
            Direction::SouthWest => GridPosition { x: self.x - CELL_SIZE, y: self.y - CELL_SIZE },
            Direction::West => GridPosition { x: self.x - CELL_SIZE, y: self.y },
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

fn dead(pos: &GridPosition, neighbors: &HashMap<GridPosition, u8>, lower: u8, upper: u8) -> bool {
    match neighbors.get(pos) {
        Some(count) => *count < lower || *count > upper,
        None => true,
    }
}

fn increment_neighbor_count(neighbors: &mut HashMap<GridPosition, u8>, pos: &GridPosition, dir: Direction) {
    let neighbor_pos = pos.offset(&dir);
    let count = neighbors.entry(neighbor_pos).or_insert(0);
    *count += 1;
}

fn count_neighbors_system(
    mut game_state: ResMut<GridState>,
    query: Query<&GridPosition>,
) {
    game_state.neighbors.clear();
    game_state.cells.clear();
    for pos in query.iter() {
        increment_neighbor_count(&mut game_state.neighbors, pos, Direction::NorthWest);
        increment_neighbor_count(&mut game_state.neighbors, pos, Direction::North);
        increment_neighbor_count(&mut game_state.neighbors, pos, Direction::NorthEast);
        increment_neighbor_count(&mut game_state.neighbors, pos, Direction::East);
        increment_neighbor_count(&mut game_state.neighbors, pos, Direction::SouthEast);
        increment_neighbor_count(&mut game_state.neighbors, pos, Direction::South);
        increment_neighbor_count(&mut game_state.neighbors, pos, Direction::SouthWest);
        increment_neighbor_count(&mut game_state.neighbors, pos, Direction::West);
        game_state.cells.insert(*pos);
    }
}

fn spawn_system(
    mut commands: Commands, 
    game_rules: Res<GameRules>, 
    game_state: Res<GridState>,
) {
    for (pos, count) in game_state.neighbors.iter() {
        if *count == game_rules.upper && !game_state.cells.contains(pos) {
            commands.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(1.0, 1.0) * CELL_SIZE as f32),
                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(pos.x as f32, pos.y as f32, 0.0)),
                ..Default::default()
            })
            .insert(pos.clone());
        }
    }
}

fn despawn_system(
    mut commands: Commands,
    game_rules: Res<GameRules>,
    game_state: Res<GridState>,
    query: Query<(Entity, &GridPosition)>
) {
    for (id, pos) in query.iter() {
        let vec = Vec2::new(pos.x as f32, pos.y as f32);
        if vec.length() > WORLD_RADIUS || dead(pos, &game_state.neighbors, game_rules.lower, game_rules.upper) {
            commands.entity(id).despawn();
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn init_system(mut commands: Commands, mut state: ResMut<State<AppState>>, mut game_state: ResMut<GridState>, query: Query<Entity, With<GridPosition>> ) {
    let cell_size = CELL_SIZE;
    let dim = INITIAL_GRID_DIM;
    let half_width = dim * cell_size / 2;
    let half_height = dim * cell_size / 2;

    game_state.neighbors.clear();
    game_state.cells.clear();   

    for id in query.iter() {
        commands.entity(id).despawn();
    }

    for i in 0..dim {
        for j in 0..dim {

            if random() { continue; }

            let x = (cell_size * i - half_width) as f32;
            let y = (cell_size * j - half_height) as f32;

            commands.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(1.0, 1.0) * cell_size as f32),
                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
                ..Default::default()
            })
            .insert(GridPosition { x: x as i16, y: y as i16});
        }
    }

    state.overwrite_set(AppState::Running).unwrap();
}

fn zoom_system(mut query: Query<&mut OrthographicProjection>, keyboard_input: Res<Input<KeyCode>>) {
    for mut ortho in query.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::Left) {
            ortho.scale *= 2.0;
        } else if keyboard_input.just_pressed(KeyCode::Right) {
            ortho.scale /= 2.0;
        }
    }
}

fn camera_move_system(mut query: Query<&mut Transform, With<OrthographicProjection>>, keyboard_input: Res<Input<KeyCode>>) {
    const MOVE_DELTA: f32 = 10.0;
    for mut transform in query.iter_mut() {
        let mut delta = Vec3::default();

        if keyboard_input.pressed(KeyCode::W) {
            delta.y += MOVE_DELTA;
        }
        if keyboard_input.pressed(KeyCode::A) {
            delta.x -= MOVE_DELTA;
        }
        if keyboard_input.pressed(KeyCode::S) {
            delta.y -= MOVE_DELTA;
        }
        if keyboard_input.pressed(KeyCode::D) {
            delta.x += MOVE_DELTA;
        }

        transform.translation += delta;
    }
}

fn pause_system(mut state: ResMut<State<AppState>>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        match state.current() {
            &AppState::Running => { state.set(AppState::Paused).unwrap(); }
            &AppState::Paused => { state.set(AppState::Running).unwrap(); }
            _ => {}
        }
    }
}

fn reset_system(mut state: ResMut<State<AppState>>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::R) {
        state.set(AppState::Init).unwrap();
    }
}

fn main() {
    App::new()
        .add_state(AppState::Init)
        .insert_resource(ClearColor(Color::WHITE))
        .insert_resource(GameRules {
            lower: 2,
            upper: 3,
        })
        .insert_resource(GridState {
            neighbors: HashMap::new(),
            cells: HashSet::new(),
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EntityCountDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .add_system(zoom_system)
        .add_system(camera_move_system)
        .add_system(pause_system)
        .add_system(reset_system)
        .add_system_set(SystemSet::on_update(AppState::Running)
            .with_system(count_neighbors_system.label("count"))
            .with_system(spawn_system.after("count"))
            .with_system(despawn_system.after("count"))
        )
        .add_system_set(SystemSet::on_enter(AppState::Init)
            .with_system(init_system)
        )       
        .run();
}