use bevy::prelude::*;
mod helpers;
use bevy_inspector_egui::{Inspectable, WorldInspectorPlugin};
use helpers::camera::movement as camera_movement;

mod map;
use map::map::{create_map, hover_on_tile, TileHandleHex, TileHandleHexHover};

#[derive(Deref, Resource)]
pub struct FontHandle(Handle<Font>);
impl FromWorld for FontHandle {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self(asset_server.load("fonts/FiraSans-Bold.ttf"))
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

#[derive(Component, Inspectable)]
pub struct Building;

// #[derive(Component)]
// pub struct WoodCutter {
//     pos: TilePos,
// }

// fn add_wood_cutter_to_tile(
//     mut commands: Commands,
//     meshes_q: Query<
//         (&TilemapSize, &TilemapGridSize, &TilemapType, &Transform),
//         (Without<HoverCursor>,),
//     >,
//     cutters_q: Query<&WoodCutter>,
//     axe_handle: Res<AxeHandle>,
//     cursor_pos: Res<CursorPos>,
//     buttons: Res<Input<MouseButton>>,
// ) {
//     if buttons.just_pressed(MouseButton::Left) {
//         // Left click adds wood cutter
//         if let Ok((map_size, grid_size, map_type, map_transform)) = meshes_q.get_single() {
//             if let Some((tile_pos, tile_centre)) =
//                 tile_pos_from_cursor(map_size, grid_size, map_type, map_transform, cursor_pos.0)
//             {
//                 if !cutters_q.iter().any(|item| item.pos == tile_pos) {
//                     let mut transform = tile_centre;
//                     transform.translation.z += 0.1;
//                     transform.scale = Vec3::new(0.8, 0.8, 0.8);
//                     commands.spawn((
//                         WoodCutter { pos: tile_pos },
//                         SpriteBundle {
//                             texture: axe_handle.clone(),
//                             transform: transform,
//                             ..Default::default()
//                         },
//                     ));
//                 }
//             }
//         }
//     }
// }

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

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: String::from("Dave Game!"),
                        ..Default::default()
                    },
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(WorldInspectorPlugin::new())
        .init_resource::<TileHandleHexHover>()
        .init_resource::<TileHandleHex>()
        .init_resource::<FontHandle>()
        .init_resource::<AxeHandle>()
        .add_startup_system(setup_camera)
        .add_startup_system(create_map)
        .add_startup_system(setup_menu)
        .add_system_to_stage(CoreStage::First, camera_movement)
        // .add_system(add_wood_cutter_to_tile)
        .add_system(hover_on_tile)
        .run();
}
