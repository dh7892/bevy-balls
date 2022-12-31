use bevy::prelude::*;
mod helpers;
use bevy_inspector_egui::{Inspectable, WorldInspectorPlugin};
use helpers::camera::movement as camera_movement;

mod map;
use map::map::{
    create_map, hover_on_tile, HoveredTile, TileHandleHex, TileHandleHexHover,
};

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

#[derive(Component)]
pub struct WoodCutter;

fn add_wood_cutter_on_click(
    mut commands: Commands,
    tile_q: Query<(Entity, Option<&Children>), (With<HoveredTile>,)>,
    existing_wood_cutters_q: Query<Entity, (With<WoodCutter>,)>,
    axe_handle: Res<AxeHandle>,
    buttons: Res<Input<MouseButton>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Some((tile, children)) = tile_q.iter().last() {
            match children {
                None => {
                    // Only build if there isn't already anythign built here.
                    // Might need a more better test later than just looking at children
                    // might need to look specifically for buildings that are children

                    // The transform is relative to the parent so we just add a small amount to z
                    // So the building appears on top
                    let mut transform = Transform {
                        ..Default::default()
                    };
                    transform.translation.z += 0.1;
                    // And scale it down a little to fit in the hex
                    transform.scale = Vec3::new(0.8, 0.8, 0.8);
                    let wood_cutter_id = commands
                        .spawn((
                            WoodCutter,
                            SpriteBundle {
                                texture: axe_handle.clone(),
                                transform: transform,
                                ..Default::default()
                            },
                        ))
                        .id();
                    commands.entity(tile).add_child(wood_cutter_id);
                }
                _ => {}
            }
        }
    }
    if buttons.just_pressed(MouseButton::Right) {
        // Right mouse will destroy the wood cutter
        if let Some((tile, children)) = tile_q.iter().last() {
            if let Some(children) = children {
                for &possible_wood_cutter in children.iter() {
                    if let Ok(_) = existing_wood_cutters_q.get(possible_wood_cutter) {
                        // The child is in our list of existing wood cutters so we want to
                        // despawn it
                        commands
                            .entity(tile)
                            .remove_children(&[possible_wood_cutter]);
                        commands.entity(possible_wood_cutter).despawn();
                    }
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
        .add_system(add_wood_cutter_on_click)
        .add_system(hover_on_tile)
        .run();
}
