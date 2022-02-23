use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};

use rand::prelude::*;

use std::collections::HashMap;
use std::collections::HashSet;

const CELL_SIZE: i32 = 4;
const INITIAL_GRID_DIM: i32 = 64;
const WORLD_RADIUS: f32 = 1000.0;

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
    x: i32,
    y: i32,
}

#[derive(Default)]
struct GridState {
    neighbors: HashMap<GridPosition, u8>,
    cells: HashSet<GridPosition>,
}

impl GridPosition {
    fn offset(&self, direction: Direction) -> GridPosition {
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

fn increment_neighbor_count(neighbors: &mut HashMap<GridPosition, u8>, pos: &GridPosition, dir: Direction) {
    let neighbor_pos = pos.offset(dir);
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
        let mut despawn = false;
        let vec = Vec2::new(pos.x as f32, pos.y as f32);
        if vec.length() > WORLD_RADIUS {
            despawn = true;
        }
        else if let Some(count) = game_state.neighbors.get(pos) {
            if *count < game_rules.lower || *count > game_rules.upper {
                despawn = true;
            }
        }

        if despawn {
            commands.entity(id).despawn();
        }
    }
}

fn setup(
    mut commands: Commands,
) {
    let cell_size = CELL_SIZE;
    let dim = INITIAL_GRID_DIM;
    let half_width = dim * cell_size / 2;
    let half_height = dim * cell_size / 2;
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.insert_resource(GameRules {
        lower: 2,
        upper: 3,
    });

    commands.insert_resource(GridState {
        neighbors: HashMap::new(),
        cells: HashSet::new(),
    });

    for i in 0..dim {
        for j in 0..dim {
            let spawn_chance: f32 = random();

            if spawn_chance < 0.95 { continue; }

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
            .insert(GridPosition { x: x as i32, y: y as i32});
        }
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .add_plugins(DefaultPlugins)
        .add_plugin(EntityCountDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .add_system(count_neighbors_system.label("count"))
        .add_system(spawn_system.after("count"))
        .add_system(despawn_system.after("count"))
        .run();
}