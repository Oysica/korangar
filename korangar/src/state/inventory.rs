use std::sync::Arc;

use korangar_interface::element::StateElement;
use korangar_networking::{InventoryItem, InventoryItemDetails, NoMetadata};
use ragnarok_packets::{EquipPosition, InventoryIndex, ItemId};
use rust_state::RustState;

use crate::graphics::Texture;
use crate::loaders::AsyncLoader;
use crate::world::ResourceMetadata;

/// Coarse-grained item buckets used to split the inventory window into tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventoryCategory {
    /// Consumables and time-delayed consumables.
    Consumable,
    /// Wearable gear, ammunition, pet eggs and pet equipment.
    Equipment,
    /// Etc. items and cards.
    Other,
    /// Cash-shop / account-bound items.
    Personal,
}

fn categorize(item_type: u8) -> InventoryCategory {
    match item_type {
        0 | 2 | 11 => InventoryCategory::Consumable,
        4 | 5 | 7 | 8 | 10 => InventoryCategory::Equipment,
        18 => InventoryCategory::Personal,
        _ => InventoryCategory::Other,
    }
}

#[derive(Default, RustState, StateElement)]
pub struct Inventory {
    #[hidden_element]
    items: Vec<InventoryItem<ResourceMetadata>>,
    #[hidden_element]
    consumable_items: Vec<InventoryItem<ResourceMetadata>>,
    #[hidden_element]
    equipment_items: Vec<InventoryItem<ResourceMetadata>>,
    #[hidden_element]
    other_items: Vec<InventoryItem<ResourceMetadata>>,
    #[hidden_element]
    personal_items: Vec<InventoryItem<ResourceMetadata>>,
    #[hidden_element]
    selected_tab: usize,
}

impl Inventory {
    fn recompute_filtered(&mut self) {
        self.consumable_items.clear();
        self.equipment_items.clear();
        self.other_items.clear();
        self.personal_items.clear();
        for item in &self.items {
            // Hide items that are currently equipped — they live in the
            // equipment window instead.
            if let InventoryItemDetails::Equippable { equipped_position, .. } = &item.details
                && !equipped_position.is_empty()
            {
                continue;
            }
            match categorize(item.item_type) {
                InventoryCategory::Consumable => self.consumable_items.push(item.clone()),
                InventoryCategory::Equipment => self.equipment_items.push(item.clone()),
                InventoryCategory::Other => self.other_items.push(item.clone()),
                InventoryCategory::Personal => self.personal_items.push(item.clone()),
            }
        }
    }

    pub fn fill(&mut self, async_loader: &AsyncLoader, items: Vec<InventoryItem<NoMetadata>>) {
        self.items = items
            .into_iter()
            .map(|item| async_loader.request_inventory_item_metadata_load(item))
            .collect();
        self.recompute_filtered();
    }

    pub fn add_item(&mut self, async_loader: &AsyncLoader, item: InventoryItem<NoMetadata>) {
        if let Some(found_item) = self.items.iter_mut().find(|inventory_item| inventory_item.index == item.index) {
            let InventoryItemDetails::Regular { amount, .. } = &mut found_item.details else {
                panic!();
            };

            let InventoryItemDetails::Regular { amount: added_amount, .. } = item.details else {
                panic!();
            };

            *amount += added_amount;
        } else {
            let item = async_loader.request_inventory_item_metadata_load(item);

            self.items.push(item);
        }
        self.recompute_filtered();
    }

    pub fn update_item_sprite(&mut self, item_id: ItemId, texture: Arc<Texture>) {
        self.items.iter_mut().filter(|item| item.item_id == item_id).for_each(|item| {
            item.metadata.texture = Some(texture.clone());
        });
        self.recompute_filtered();
    }

    pub fn remove_item(&mut self, index: InventoryIndex, remove_amount: u16) {
        let position = self
            .items
            .iter()
            .position(|item| item.index == index)
            .expect("item not in inventory");

        if let InventoryItemDetails::Regular { amount, .. } = &mut self.items[position].details
            && *amount > remove_amount
        {
            *amount -= remove_amount;
        } else {
            self.items.remove(position);
        }
        self.recompute_filtered();
    }

    pub fn update_equipped_position(&mut self, index: InventoryIndex, new_equipped_position: EquipPosition) {
        let item = self.items.iter_mut().find(|item| item.index == index).unwrap();

        let InventoryItemDetails::Equippable { equipped_position, .. } = &mut item.details else {
            // This can happen for ammunition for example.
            return;
        };

        *equipped_position = new_equipped_position;
        self.recompute_filtered();
    }
}
