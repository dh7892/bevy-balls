/// This map module handles creating our map of tiles.
/// It also provides helpers for looking up tiles in our map by index,
/// and finding neighbours etc
///
use bevy::{math::Vec3Swizzles, prelude::*};
use itertools::Itertools;
use std::collections::HashMap;
use tiled::Tile;

#[derive(Deref, Resource)]
pub struct TileHandleHex(Handle<Image>);

impl FromWorld for TileHandleHex {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self(asset_server.load("hex.png"))
    }
}

#[derive(Deref, Resource)]
pub struct TileHandleHexHover(Handle<Image>);

impl FromWorld for TileHandleHexHover {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self(asset_server.load("hex-hovered.png"))
    }
}

#[derive(Component)]
pub struct IsTile;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct TileIndex {
    row: usize,
    col: usize,
}
#[derive(Resource)]
pub struct TileMap {
    rows: usize,
    cols: usize,
    tilesize: Vec2,
    tile_entities: HashMap<TileIndex, Entity>,
}

/// Add entities to represent each tile and a resource to hold some meta-data about the map
pub fn create_map(mut commands: Commands, tile_image: Res<TileHandleHex>) {
    let rows: usize = 5;
    let cols: usize = 5;
    // commands.init_resource::<TileHandleHex>();
    let tilesize = Vec2::new(50.0, 50.0);
    let mut tile_entities: HashMap<TileIndex, Entity> = HashMap::new();

    let delta = Vec2::new(tilesize.x, tilesize.y * 3_f64.sqrt() as f32 / 2.0);

    // let origin = Transform::from_xyz(0.0, 0.0, 0.0);
    for (i, j) in (0..rows).cartesian_product(0..cols) {
        let stagger = if j % 2 == 0 { 0.0 } else { delta.x / 2.0 };
        let translation = Vec3::new(i as f32 * delta.x + stagger, j as f32 * delta.y, 1.0);
        let _tile_transform = Transform {
            translation: translation,
            ..Default::default()
        };

        // Insert the tile components, and save the entity for later
        // TODO add sprite
        let tile_entity = commands
            .spawn((
                IsTile,
                SpriteBundle {
                    transform: Transform {
                        translation,
                        ..Default::default()
                    },
                    texture: tile_image.clone(),
                    ..Default::default()
                },
            ))
            .id();
        tile_entities.insert(TileIndex { row: j, col: i }, tile_entity);
    }

    let tile_map = TileMap {
        rows,
        cols,
        tilesize,
        tile_entities,
    };
    commands.insert_resource(tile_map);
}

// Converts the cursor position into a world position, taking into account any transforms applied
// the camera.
fn cursor_pos_in_world(
    windows: &Windows,
    cursor_pos: Vec2,
    cam_t: &Transform,
    cam: &Camera,
) -> Vec3 {
    let window = windows.primary();

    let window_size = Vec2::new(window.width(), window.height());

    // Convert screen position [0..resolution] to ndc [-1..1]
    // (ndc = normalized device coordinates)
    let ndc_to_world = cam_t.compute_matrix() * cam.projection_matrix().inverse();
    let ndc = (cursor_pos / window_size) * 2.0 - Vec2::ONE;
    ndc_to_world.project_point3(ndc.extend(0.0))
}

// Given a point in x,y space and a map, return the index of the tile at the point,
// or None if the point is off the edge of the map
fn tile_index_from_xy_coord(point: Vec2, map: &TileMap) -> Option<TileIndex> {
    let delta = Vec2::new(map.tilesize.x, map.tilesize.y * 3_f64.sqrt() as f32 / 2.0);

    let row = (delta.y / 2_f32 + point.y) / delta.y;
    if row < 0_f32 || row > map.rows as f32 {
        // Pointer is outside y-range of map
        return None;
    }
    let row_index = row as usize;
    let offset = if row_index % 2 == 0 {
        0.0
    } else {
        delta.x / 2_f32
    };
    let col = (delta.x / 2_f32 - offset + point.x) / delta.x;
    if col < 0_f32 || col > map.cols as f32 {
        // Pointer is outside x-range of map
        return None;
    }
    let col_index = col as usize;
    Some(TileIndex {
        row: row_index,
        col: col_index,
    })
}

#[test]
fn test_tile_index_from_xy() {
    let map = TileMap {
        rows: 5,
        cols: 5,
        tilesize: Vec2 { x: 50.0, y: 50.0},
        tile_entities: HashMap::new(),
    };

    let mut point = Vec2::new(0.0, 0.0);
    assert_eq!(tile_index_from_xy_coord(point, &map), Some(TileIndex{row: 0, col: 0}));
    point.x = -60.0;
    assert_eq!(tile_index_from_xy_coord(point, &map), None);



}

#[derive(Component)]
pub struct HoveredTile;

/// When we move the cursor
/// Check update which tile we are hovering on
pub fn hover_on_tile(
    mut commands: Commands,
    mut all_tiles_q: Query<&mut Sprite, (With<IsTile>,)>,
    mut hovered_tiles_q: Query<(Entity), (With<HoveredTile>,)>,
    windows: Res<Windows>,
    cam_q: Query<(&Transform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    tile_map: Res<TileMap>,
) {
    // If we have multiple movements, we only care about the last one
    if let Some(cursor_moved) = cursor_moved_events.iter().last() {
        // First, remove hovered status from any tiles that have that status. 
        for (entity) in hovered_tiles_q.iter_mut() {
            commands.entity(entity).remove::<HoveredTile>();
            if let Ok(mut sprite) = all_tiles_q.get_mut(entity){
                sprite.color = Color::WHITE;
            }
        }
        
        for (cam_t, cam) in cam_q.iter() {
            let cursor_pos = cursor_pos_in_world(&windows, cursor_moved.position, cam_t, cam).xy();
            if let Some(tile_index) = tile_index_from_xy_coord(cursor_pos, &tile_map) {
                // If we are over a tile, highlght it
                if let Some(tile_entity) = tile_map.tile_entities.get(&tile_index){
                    if let Ok(mut sprite) = all_tiles_q.get_mut(*tile_entity) {
                        sprite.color = Color::RED;
                        commands.entity(*tile_entity).insert(HoveredTile);
                    }
                }
            }
        }
    }
}
