use bevy::{prelude::*, transform, utils::info};
use std::fs;
use crate::pacman::PacMan;


// Assuming TileType and main function are defined as before
#[derive(PartialEq,Debug)]
pub enum TileType {
    Wall,
    Path,
    Dot,
    Cherry,
}

pub struct MazePlugin;

impl Plugin for MazePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Maze::new(load_maze("assets/maze.txt")));
        app.add_systems(Startup, spawn_maze);
        app.add_systems(Update, pacman_eat_dots);
        app.add_systems(Update, refresh_map_display);
    }
}

#[derive(Resource)]
pub struct Maze {
    // the maze is 13 * 38
    cells: Vec<Vec<TileType>>
}

#[derive(Component)]
pub struct Dot;

#[derive(Component)]
struct MapTileEntity;

impl Maze {
    fn new(cells: Vec<Vec<TileType>>) -> Self {
        Self { cells }
    }
    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        match self.cells.get(y).and_then(|row| row.get(x)) {
            Some(&TileType::Path) => true,
            Some(&TileType::Dot) => true,
            Some(&TileType::Cherry) => true,
            Some(&TileType::Wall) => false,
            _ => false,
        }
    }
}

fn spawn_maze(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    let maze = load_maze("assets/maze.txt"); // Make sure to implement load_maze appropriately

    // Assuming you have separate images for each type
    let wall_texture_handle = asset_server.load("wall.png");
    let dot_texture_handle = asset_server.load("dot.png");
    let cherry_texture_handle = asset_server.load("cherry.png");

    for (y, row) in maze.iter().enumerate() {
        for (x, ref tile) in row.iter().enumerate() {
            let position = Vec3::new(x as f32 * 32.0 - 600.0, y as f32 * -32.0 + 300.0, 0.0); // Example tile size of 32x32
            match tile {
                TileType::Wall => {
                    commands.spawn(SpriteBundle {
                        texture: wall_texture_handle.clone(),
                        transform: Transform::from_translation(position),
                        ..default()
                    });
                }
                TileType::Dot => {
                    let scale_factor = 32.0 / 30.0;
                    commands.spawn(SpriteBundle {
                        texture: dot_texture_handle.clone(),
                        transform: Transform::from_translation(position).with_scale(Vec3::splat(scale_factor)),
                        ..default()
                    }).insert(Dot);
                }
                // Repeat for Cherry, Player, etc., adjusting texture_handle as needed
                TileType::Cherry => {
                    commands.spawn(SpriteBundle {
                        texture: cherry_texture_handle.clone(),
                        transform: Transform::from_translation(position),
                        ..default()
                    });
                }
                _ => ()
            }
        }
    }
}

fn load_maze(file_path: &str) -> Vec<Vec<TileType>> {
    let content = fs::read_to_string(file_path).expect("Failed to load maze file");
    content
        .lines()
        .map(|line| {
            line.chars()
                .map(|c| match c {
                    'W' => TileType::Wall,
                    'D' => TileType::Dot,
                    'C' => TileType::Cherry,
                    _ => TileType::Path,
                })
                .collect()
        })
        .collect()
}

fn pacman_eat_dots(
    pacman_query: Query<&Transform, With<PacMan>>, // Assuming you have a Pacman component
    mut maze: ResMut<Maze>,
    mut commands: Commands,
    dot_query: Query<(Entity, &Transform), With<Dot>>, // Assuming you have a Dot component
) {
    if let Some(pacman_transform) = pacman_query.iter().next() {
        // Convert Pac-Man's position to map grid coordinates
        let map_x = ((pacman_transform.translation.x + 615.0) / 32.0) as usize; // Adjust according to your scale
        let map_y = (13.0 - (pacman_transform.translation.y + 100.0) / 32.0 )as usize;
        if maze.cells[map_y][map_x] == TileType::Dot {
            maze.cells[map_y][map_x] = TileType::Path;
            // Despawn all dot entities (and potentially other tile types)
            for (entity, _) in dot_query.iter() {
                commands.entity(entity).despawn();
            }

            // Logic to respawn the map can be triggered here or in another system
            // depending on your game's architecture
        }
    }
}

fn refresh_map_display(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    maze: Res<Maze>,
    existing_tiles_query: Query<Entity, With<MapTileEntity>>, // Assuming MapTileEntity marker component
) {
    // Despawn existing map tiles
    for entity in existing_tiles_query.iter() {
        commands.entity(entity).despawn();
    }

    // Respawn map tiles according to the `map` state
    for (y, row) in maze.cells.iter().enumerate() {
        for (x, ref tile) in row.iter().enumerate()  {
            let position = Vec3::new(x as f32 * 32.0 - 600.0, y as f32 * -32.0 + 300.0, 0.0); 
            if let Some(texture) = match tile {
                TileType::Wall => Some("wall.png"),
                TileType::Dot => Some("dot.png"),
                TileType::Cherry => Some("cherry.png"),
                _ => None, // Skip spawning for empty tiles
            } {
                let texture_handle = asset_server.load(texture);
                commands.spawn(SpriteBundle {
                    texture: texture_handle.clone(),
                    transform: Transform::from_translation(position),
                    ..Default::default()
                }).insert(MapTileEntity); // Mark it as a map tile entity
            }
        }
    }
}

fn world_to_grid(world_position: Vec3, cell_size: f32) -> (usize, usize) {
    let grid_x = ((world_position.x + 615.0) / cell_size).floor() as usize;
    let grid_y = (13.0 - (world_position.y + 100.0) / cell_size).floor() as usize;
    (grid_x, grid_y)
}