/// This map module handles creating our map of tiles.
/// It also provides helpers for looking up tiles in our map by index,
/// and finding neighbours etc
///
use bevy::prelude::*;
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Deref, Resource)]
pub struct TileHandleHex(Handle<Image>);

impl FromWorld for TileHandleHex {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self(asset_server.load("hex.png"))
    }
}

#[derive(Component)]
struct IsTile;

#[derive(Debug, Eq, PartialEq, Hash)]
struct TileIndex {
    row: usize,
    col: usize,
}

#[derive(Resource)]
struct TileMap {
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

    let delta = Vec2::new(tilesize.x, tilesize.y * 3_f64.sqrt() as f32/2.0);

    let origin = Transform::from_xyz(0.0, 0.0, 0.0);
    for (i, j) in (0..rows).cartesian_product(0..cols) {
        // TODO: scale j direction for hexness
        let stagger = if j%2 ==0  {0.0} else {delta.x/2.0};
        let translation = Vec3::new(i as f32 * delta.x + stagger, j as f32 * delta.y, 1.0);
        let mut tile_transform = Transform {
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
        tile_entities.insert(TileIndex { row: i, col: j }, tile_entity);
    }

    let tile_map = TileMap {
        rows,
        cols,
        tilesize,
        tile_entities,
    };
    commands.insert_resource(tile_map);
}
