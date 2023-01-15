use bevy::prelude::*;
mod helpers;
use bevy_inspector_egui::{Inspectable, WorldInspectorPlugin};
use helpers::camera::movement as camera_movement;

mod map;
use map::{create_map, hover_on_tile, HoveredTile, TileHandleHex, TileHandleHexHover};

mod resources;
use resources::{ItemType, PlayerResources};

#[derive(Resource, Default, Debug)]
pub struct TurnCounter {
    turn: u32,
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
pub struct AxeHandle(Handle<Image>);
impl FromWorld for AxeHandle {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self(asset_server.load("axe.png"))
    }
}

#[derive(Component, Inspectable)]
pub struct Building;

/// Productions parameters common to anything that produces items
#[derive(Component, Inspectable)]
struct Producer {
    /// Current number of turns until we next produce
    turns_remaining: u32,
    /// This is how many units are produced in each batch
    number_produced: u32,
    /// This is how many turns it takes to produce a batch
    production_turns: u32,
    /// The type of item produced
    produces_item_type: ItemType,
}

impl Producer {
    /// Try to produce items.
    pub fn produce(&mut self, player_resources: &mut PlayerResources) {
        if self.turns_remaining == 1 {
            // Add a batch of resources to the player's inventory
            player_resources.add_item(&self.produces_item_type, self.number_produced);
            // Reset the turn counter
            self.turns_remaining = self.production_turns;
        } else {
            self.turns_remaining -= 1;
        }
    }
}

#[derive(Component)]
pub struct WoodCutter;

#[derive(Resource)]
pub enum SelectedAction {
    DemolishBuilding,
    BuildWoodCutter,
}

impl Default for SelectedAction {
    fn default() -> Self {
        SelectedAction::BuildWoodCutter
    }
}

/// For all producers, make stuff!
fn produce_items_for_a_turn(
    mut resources: ResMut<PlayerResources>,
    mut producers: Query<&mut Producer>,
    key: Res<Input<KeyCode>>,
    mut turn_counter: ResMut<TurnCounter>,
) {
    if key.just_pressed(KeyCode::Space) {
        for mut p in producers.iter_mut() {
            p.produce(&mut resources);
        }
        turn_counter.turn +=1;
        println!("Current Player Resources: {resources:?}, current turn now: {turn_counter:?}");
    }
}

fn add_wood_cutter_on_click(
    mut commands: Commands,
    tile_q: Query<(Entity, Option<&Children>), (With<HoveredTile>,)>,
    existing_wood_cutters_q: Query<Entity, (With<WoodCutter>,)>,
    axe_handle: Res<AxeHandle>,
    buttons: Res<Input<MouseButton>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Some((tile, None)) = tile_q.iter().last() {
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
                    Producer {
                        turns_remaining: 1,
                        number_produced: 1,
                        production_turns: 1,
                        produces_item_type: ItemType::Wood,
                    },
                    SpriteBundle {
                        texture: axe_handle.clone(),
                        transform,
                        ..Default::default()
                    },
                ))
                .id();
            commands.entity(tile).add_child(wood_cutter_id);
        }
    }
    if buttons.just_pressed(MouseButton::Right) {
        // Right mouse will destroy the wood cutter
        if let Some((tile, Some(children))) = tile_q.iter().last() {
            for &possible_wood_cutter in children.iter() {
                if existing_wood_cutters_q.get(possible_wood_cutter).is_ok() {
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
        .init_resource::<SelectedAction>()
        .init_resource::<PlayerResources>()
        .init_resource::<TurnCounter>()
        .add_startup_system(setup_camera)
        .add_startup_system(create_map)
        .add_startup_system(setup_menu)
        .add_system_to_stage(CoreStage::First, camera_movement)
        .add_system(add_wood_cutter_on_click)
        .add_system(hover_on_tile)
        .add_system(produce_items_for_a_turn)
        .run();
}
