use bevy::math::Vec4Swizzles;
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::hex_grid::neighbors::{HexDirection, HexNeighbors};
use bevy_ecs_tilemap::prelude::*;
mod helpers;
use bevy_inspector_egui::WorldInspectorPlugin;
use helpers::camera::movement as camera_movement;

// Press SPACE to change map type. Hover over a tile to highlight its label (red) and those of its
// neighbors (blue). Press and hold one of keys 0-5 to mark the neighbor in that direction (green).

// You can increase the MAP_SIDE_LENGTH, in order to test that mouse picking works for larger maps,
// but just make sure that you run in release mode (`cargo run --release --example mouse_to_tile`)
// otherwise things might be too slow.
const MAP_SIDE_LENGTH_X: u32 = 4;
const MAP_SIDE_LENGTH_Y: u32 = 4;

const TILE_SIZE_HEX_ROW: TilemapTileSize = TilemapTileSize { x: 50.0, y: 58.0 };
const GRID_SIZE_HEX_ROW: TilemapGridSize = TilemapGridSize { x: 50.0, y: 58.0 };

#[derive(Deref, Resource)]
pub struct TileHandleHex(Handle<Image>);

impl FromWorld for TileHandleHex {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self(asset_server.load("hex.png"))
    }
}

#[derive(Deref, Resource)]
pub struct FontHandle(Handle<Font>);
impl FromWorld for FontHandle {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self(asset_server.load("fonts/FiraSans-Bold.ttf"))
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

#[derive(Deref, Resource)]
pub struct AxeHandle(Handle<Image>);
impl FromWorld for AxeHandle {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self(asset_server.load("axe.png"))
    }
}

// Add a single axe icon over the top of the tile at 0,0
fn add_wood_cutter(
    mut commands: Commands,
    _tilemap_q: Query<(&Transform, &TilemapType, &TilemapGridSize, &TileStorage)>,
    _tile_q: Query<&mut TilePos>,
    axe_handle: Res<AxeHandle>,
) {
    let scale = Vec3::new(0.7, 0.7, 1.0);
    let translation = Vec3::new(0.0, 0.0, 1.1);
    commands.spawn(SpriteBundle {
        texture: axe_handle.clone(),
        transform: Transform {
            translation: translation,
            scale: scale,
            ..Default::default()
        },
        ..Default::default()
    });
}

// Generates the initial tilemap, which is a hex grid.
fn spawn_tilemap(mut commands: Commands, tile_handle_hex: Res<TileHandleHex>) {
    commands.spawn(Camera2dBundle::default());

    let map_size = TilemapSize {
        x: MAP_SIDE_LENGTH_X,
        y: MAP_SIDE_LENGTH_Y,
    };

    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    fill_tilemap(
        TileTextureIndex(0),
        map_size,
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    let tile_size = TILE_SIZE_HEX_ROW;
    let grid_size = GRID_SIZE_HEX_ROW;
    let map_type = TilemapType::Hexagon(HexCoordSystem::Row);

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(tile_handle_hex.clone()),
        tile_size,
        map_type,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    });
}

#[derive(Component)]
struct TileLabel(Entity);

// Generates tile position labels of the form: `(tile_pos.x, tile_pos.y)`
fn spawn_tile_labels(
    mut commands: Commands,
    tilemap_q: Query<(&Transform, &TilemapType, &TilemapGridSize, &TileStorage)>,
    tile_q: Query<&mut TilePos>,
    font_handle: Res<FontHandle>,
) {
    let text_style = TextStyle {
        font: font_handle.clone(),
        font_size: 20.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment::CENTER;
    for (map_transform, map_type, grid_size, tilemap_storage) in tilemap_q.iter() {
        for tile_entity in tilemap_storage.iter().flatten() {
            let tile_pos = tile_q.get(*tile_entity).unwrap();
            let tile_center = tile_pos.center_in_world(grid_size, map_type).extend(1.0);
            let transform = *map_transform * Transform::from_translation(tile_center);

            let label_entity = commands
                .spawn(Text2dBundle {
                    text: Text::from_section(
                        format!("{}, {}", tile_pos.x, tile_pos.y),
                        text_style.clone(),
                    )
                    .with_alignment(text_alignment),
                    transform,
                    ..default()
                })
                .id();
            commands
                .entity(*tile_entity)
                .insert(TileLabel(label_entity));
        }
    }
}

#[derive(Component)]
struct Hovered;

#[derive(Component)]
struct HoverCursor {
    tile_pos: Option<TilePos>,
}

// Add a the hover cursor (currently not on a tile)
fn add_hover_cursor(mut commands: Commands, hex_tile_hover: Res<TileHandleHexHover>) {
    commands.spawn((
        HoverCursor { tile_pos: None },
        SpriteBundle {
            texture: hex_tile_hover.clone(),
            visibility: Visibility { is_visible: false },
            transform: Transform {
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}
// Converts the cursor position into a world position, taking into account any transforms applied
// the camera.
pub fn cursor_pos_in_world(
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

#[derive(Resource)]
pub struct CursorPos(Vec3);
impl Default for CursorPos {
    fn default() -> Self {
        // Initialize the cursor pos at some far away place. It will get updated
        // correctly when the cursor moves.
        Self(Vec3::new(-1000.0, -1000.0, 0.0))
    }
}

// We need to keep the cursor position updated based on any `CursorMoved` events.
pub fn update_cursor_pos(
    windows: Res<Windows>,
    camera_q: Query<(&Transform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    for cursor_moved in cursor_moved_events.iter() {
        // To get the mouse's world position, we have to transform its window position by
        // any transforms on the camera. This is done by projecting the cursor position into
        // camera space (world space).
        for (cam_t, cam) in camera_q.iter() {
            *cursor_pos = CursorPos(cursor_pos_in_world(
                &windows,
                cursor_moved.position,
                cam_t,
                cam,
            ));
        }
    }
}

// This is where we check which tile the cursor is hovered over.
// We need:
// timemap
// cursor_pos so we know what the mouse is pointing to
// current highlighted tile
fn update_hover_cursor(
    mut commands: Commands,
    cursor_pos: Res<CursorPos>,
    tilemap_q: Query<
        (
            &TilemapSize,
            &TilemapGridSize,
            &TilemapType,
            &TileStorage,
            &Transform,
        ),
        (Without<HoverCursor>,),
    >,
    mut hover_cursor_q: Query<(Entity, &mut HoverCursor, &mut Visibility, &mut Transform)>,
    tile_pos_q: Query<&TilePos>,
) {
    for (map_size, grid_size, map_type, tile_storage, map_transform) in tilemap_q.iter() {
        // Grab the cursor position from the `Res<CursorPos>`
        let cursor_pos: Vec3 = cursor_pos.0;
        // We need to make sure that the cursor's world position is correct relative to the map
        // due to any map transformation.
        let cursor_in_map_pos: Vec2 = {
            // Extend the cursor_pos vec3 by 1.0
            let cursor_pos = Vec4::from((cursor_pos, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };
        // Once we have a world position we can transform it into a possible tile position.
        if let Some(tile_pos) =
            TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
        {
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                for (_, hc, mut vis, mut trans) in hover_cursor_q.iter_mut() {
                    // The mouse cursor is over a tile, so we need to set the hover_cursor
                    // position and make the sprite visible
                    let tile_pos = tile_pos_q.get(tile_entity).unwrap();
                    let tile_center = tile_pos.center_in_world(grid_size, map_type).extend(1.1);
                    let hover_location = *map_transform * Transform::from_translation(tile_center);
                    trans.translation = hover_location.translation;
                    vis.is_visible = true;
                }
            }
        } else {
            // The cursor has moved off the tiles so hide the cursor
            // for (_, hc, mut vis, mut trans) in hover_cursor_q.iter_mut() {
            //     vis.is_visible = false;
            // }
        }
    }
}

fn hover_highlight(
    mut commands: Commands,
    cursor_pos: Res<CursorPos>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Transform,
    )>,
    hovered_tiles_q: Query<&TilePos, With<Hovered>>,
    hex_tile_hover: Res<TileHandleHexHover>,
) {
    for (map_size, grid_size, map_type, tile_storage, map_transform) in tilemap_q.iter() {
        // Grab the cursor position from the `Res<CursorPos>`
        let cursor_pos: Vec3 = cursor_pos.0;
        // We need to make sure that the cursor's world position is correct relative to the map
        // due to any map transformation.
        let cursor_in_map_pos: Vec2 = {
            // Extend the cursor_pos vec3 by 1.0
            let cursor_pos = Vec4::from((cursor_pos, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };
        // Once we have a world position we can transform it into a possible tile position.
        if let Some(tile_pos) =
            TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
        {
            // Highlight the relevant tile's label
            // if let Some(tile_entity) = tile_storage.get(&tile_pos) {
            //     let tile_pos = tile_q.get(tile_entity).unwrap();
            //     let tile_center = tile_pos.center_in_world(grid_size, map_type).extend(1.1);
            //     let hover_location = *map_transform * Transform::from_translation(tile_center);

            //     commands.entity(tile_entity).insert(Hovered);

            //     let hover_cursor = commands.spawn(SpriteBundle{
            //         texture: hex_tile_hover.clone(),
            //         transform: hover_location,
            //         ..Default::default()
            //     }).id();
            //     commands.entity(tile_entity).push_children(&[hover_cursor]);

            // }
        }
    }
}

#[derive(Component)]
struct NeighborHighlight;

// Highlight neigbours
#[allow(clippy::too_many_arguments)]
fn highlight_neighbor_label(
    mut commands: Commands,
    tilemap_query: Query<(&TilemapType, &TilemapSize, &TileStorage)>,
    keyboard_input: Res<Input<KeyCode>>,
    highlighted_tiles_q: Query<Entity, With<NeighborHighlight>>,
    hovered_tiles_q: Query<&TilePos, With<Hovered>>,
    tile_label_q: Query<&TileLabel>,
    mut text_q: Query<&mut Text>,
) {
    // Un-highlight any previously highlighted tile labels.
    for highlighted_tile_entity in highlighted_tiles_q.iter() {
        if let Ok(label) = tile_label_q.get(highlighted_tile_entity) {
            if let Ok(mut tile_text) = text_q.get_mut(label.0) {
                for mut section in tile_text.sections.iter_mut() {
                    section.style.color = Color::BLACK;
                }
                commands
                    .entity(highlighted_tile_entity)
                    .remove::<NeighborHighlight>();
            }
        }
    }

    for (map_type, map_size, tile_storage) in tilemap_query.iter() {
        let hex_coord_sys = if let TilemapType::Hexagon(hex_coord_sys) = map_type {
            hex_coord_sys
        } else {
            continue;
        };

        for hovered_tile_pos in hovered_tiles_q.iter() {
            let neighboring_positions =
                HexNeighbors::get_neighboring_positions(hovered_tile_pos, map_size, hex_coord_sys);

            for neighbor_pos in neighboring_positions.iter() {
                // We want to ensure that the tile position lies within the tile map, so we do a
                // `checked_get`.
                if let Some(tile_entity) = tile_storage.checked_get(neighbor_pos) {
                    if let Ok(label) = tile_label_q.get(tile_entity) {
                        if let Ok(mut tile_text) = text_q.get_mut(label.0) {
                            for mut section in tile_text.sections.iter_mut() {
                                section.style.color = Color::BLUE;
                            }
                            commands.entity(tile_entity).insert(NeighborHighlight);
                        }
                    }
                }
            }

            let selected_hex_direction = if keyboard_input.pressed(KeyCode::Key0) {
                Some(HexDirection::Zero)
            } else if keyboard_input.pressed(KeyCode::Key1) {
                Some(HexDirection::One)
            } else if keyboard_input.pressed(KeyCode::Key2) {
                Some(HexDirection::Two)
            } else if keyboard_input.pressed(KeyCode::Key3) {
                Some(HexDirection::Three)
            } else if keyboard_input.pressed(KeyCode::Key4) {
                Some(HexDirection::Four)
            } else if keyboard_input.pressed(KeyCode::Key5) {
                Some(HexDirection::Five)
            } else {
                None
            };

            if let Some(hex_direction) = selected_hex_direction {
                let tile_pos = match map_type {
                    TilemapType::Hexagon(hex_coord_sys) => {
                        // Get the neighbor in a particular direction.
                        // This function does not check to see if the calculated neighbor lies
                        // within the tile map.
                        hex_direction.offset(hovered_tile_pos, *hex_coord_sys)
                    }
                    _ => unreachable!(),
                };

                // We want to ensure that the tile position lies within the tile map, so we do a
                // `checked_get`.
                if let Some(tile_entity) = tile_storage.checked_get(&tile_pos) {
                    if let Ok(label) = tile_label_q.get(tile_entity) {
                        if let Ok(mut tile_text) = text_q.get_mut(label.0) {
                            for mut section in tile_text.sections.iter_mut() {
                                section.style.color = Color::GREEN;
                            }
                            commands.entity(tile_entity).insert(NeighborHighlight);
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: String::from(
                            "Hexagon Neighbors - Hover over a tile, and then press 0-5 to mark neighbors",
                        ),
                        ..Default::default()
                    },
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(TilemapPlugin)
        .init_resource::<CursorPos>()
        .init_resource::<TileHandleHexHover>()
        .init_resource::<TileHandleHex>()
        .init_resource::<FontHandle>()
        .init_resource::<AxeHandle>()
        .add_startup_system(spawn_tilemap)
        // .add_startup_system_to_stage(StartupStage::PostStartup, spawn_tile_labels)
        .add_startup_system_to_stage(StartupStage::PostStartup, add_wood_cutter)
        .add_startup_system_to_stage(StartupStage::PostStartup, add_hover_cursor)
        .add_system_to_stage(CoreStage::First, camera_movement)
        .add_system_to_stage(CoreStage::First, update_cursor_pos.after(camera_movement))
        .add_system(hover_highlight)
        .add_system(update_hover_cursor)
        .run();
}
