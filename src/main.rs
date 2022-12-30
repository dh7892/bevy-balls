use bevy::math::Vec4Swizzles;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
mod helpers;
use bevy_inspector_egui::{Inspectable, WorldInspectorPlugin};
use helpers::camera::movement as camera_movement;

mod map;
use map::map::{create_map, TileHandleHex};

// Press SPACE to change map type. Hover over a tile to highlight its label (red) and those of its
// neighbors (blue). Press and hold one of keys 0-5 to mark the neighbor in that direction (green).

// You can increase the MAP_SIDE_LENGTH, in order to test that mouse picking works for larger maps,
// but just make sure that you run in release mode (`cargo run --release --example mouse_to_tile`)
// otherwise things might be too slow.
const MAP_SIDE_LENGTH_X: u32 = 4;
const MAP_SIDE_LENGTH_Y: u32 = 4;

const TILE_SIZE_HEX_ROW: TilemapTileSize = TilemapTileSize { x: 50.0, y: 58.0 };
const GRID_SIZE_HEX_ROW: TilemapGridSize = TilemapGridSize { x: 50.0, y: 58.0 };

// #[derive(Deref, Resource)]
// pub struct TileHandleHex(Handle<Image>);

// impl FromWorld for TileHandleHex {
//     fn from_world(world: &mut World) -> Self {
//         let asset_server = world.resource::<AssetServer>();
//         Self(asset_server.load("hex.png"))
//     }
// }

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

// Given some map data and a position in 3d,
// Return the (TilePos, Transform) of the tile that is near that point
// Or None if there is no tile under the point of interest
fn tile_pos_from_cursor(
    map_size: &TilemapSize,
    grid_size: &TilemapGridSize,
    map_type: &TilemapType,
    map_transform: &Transform,
    cursor_pos: Vec3,
) -> Option<(TilePos, Transform)> {
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
        let tile_center = tile_pos.center_in_world(grid_size, map_type).extend(1.0);
        let tile_world_location = *map_transform * Transform::from_translation(tile_center);
        return Some((tile_pos, tile_world_location));
    }

    None
}

#[derive(Component, Inspectable)]
pub struct Building;

#[derive(Component)]
pub struct WoodCutter {
    pos: TilePos,
}

fn add_wood_cutter_to_tile(
    mut commands: Commands,
    meshes_q: Query<
        (&TilemapSize, &TilemapGridSize, &TilemapType, &Transform),
        (Without<HoverCursor>,),
    >,
    cutters_q: Query<&WoodCutter>,
    axe_handle: Res<AxeHandle>,
    cursor_pos: Res<CursorPos>,
    buttons: Res<Input<MouseButton>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        // Left click adds wood cutter
        if let Ok((map_size, grid_size, map_type, map_transform)) = meshes_q.get_single() {
            if let Some((tile_pos, tile_centre)) =
                tile_pos_from_cursor(map_size, grid_size, map_type, map_transform, cursor_pos.0)
            {
                if !cutters_q.iter().any(|item| item.pos == tile_pos) {
                    let mut transform = tile_centre;
                    transform.translation.z += 0.1;
                    transform.scale = Vec3::new(0.8, 0.8, 0.8);
                    commands.spawn((
                        WoodCutter { pos: tile_pos },
                        SpriteBundle {
                            texture: axe_handle.clone(),
                            transform: transform,
                            ..Default::default()
                        },
                    ));
                }
            }
        }
    }
}

fn setup_menu(mut commands: Commands, axe_handle: Res<AxeHandle>) {
    commands
        .spawn(ButtonBundle {
            style: Style {
                align_self: AlignSelf::Center,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                size: Size::new(Val::Percent(20.0), Val::Percent(10.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                style: Style {
                    size: Size::new(Val::Px(50.0), Val::Auto),
                    ..default()
                },
                image: axe_handle.clone().into(),
                ..default()
            });
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
    cursor_pos: Res<CursorPos>,
    tilemap_q: Query<
        (&TilemapSize, &TilemapGridSize, &TilemapType, &Transform),
        (Without<HoverCursor>,),
    >,
    mut hover_cursor_q: Query<(&mut HoverCursor, &mut Visibility, &mut Transform)>,
) {
    for (map_size, grid_size, map_type, map_transform) in tilemap_q.iter() {
        if let Some((tile_pos, tile_centre)) =
            tile_pos_from_cursor(map_size, grid_size, map_type, map_transform, cursor_pos.0)
        {
            for (mut hc, mut vis, mut trans) in hover_cursor_q.iter_mut() {
                // The mouse cursor is over a tile, so we need to set the hover_cursor
                // position and make the sprite visible
                trans.translation = tile_centre.translation;
                // Increase z a little so it's in top of the tile
                trans.translation.z += 0.1;

                vis.is_visible = true;
                // Update the cursor's knowledge of the tile it's highlighting
                hc.tile_pos = Some(tile_pos);
            }
        } else {
            // The cursor has moved off the tiles so hide the cursor
            for (mut hc, mut vis, _) in hover_cursor_q.iter_mut() {
                vis.is_visible = false;
                hc.tile_pos = None;
            }
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
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
        .add_startup_system(setup_camera)
        .add_startup_system(create_map)
        // .add_startup_system(spawn_tilemap)
        // .add_startup_system(setup_menu)
        // .add_startup_system_to_stage(StartupStage::PostStartup, add_hover_cursor)
        .add_system_to_stage(CoreStage::First, camera_movement)
        // .add_system_to_stage(CoreStage::First, update_cursor_pos.after(camera_movement))
        // .add_system(add_wood_cutter_to_tile)
        // .add_system(update_hover_cursor)
        .run();
}
