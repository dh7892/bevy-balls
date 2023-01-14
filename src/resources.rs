use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

#[derive(Inspectable)]
pub enum ItemType {
    Wood,
    Stone,
}

#[derive(Resource, Debug)]
pub struct PlayerResources {
    pub wood: u32,
    pub stone: u32,
}
impl FromWorld for PlayerResources {
    fn from_world(_world: &mut World) -> Self {
        PlayerResources { wood: 0, stone: 0 }
    }
}

impl PlayerResources {
    /// Add `count` items of type `item` to the player's resources
    pub fn add_item(&mut self, item: &ItemType, count: u32) {
        match item {
            ItemType::Wood => self.wood += count,
            ItemType::Stone => self.stone += count,
        }

    }
}