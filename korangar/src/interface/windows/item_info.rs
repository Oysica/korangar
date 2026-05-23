use korangar_interface::window::{CustomWindow, Window};
use korangar_networking::{InventoryItem, InventoryItemDetails};

use crate::graphics::Color;
use crate::interface::windows::WindowClass;
use crate::loaders::OverflowBehavior;
use crate::state::ClientState;
use crate::state::theme::InterfaceThemeType;
use crate::world::ResourceMetadata;

pub struct ItemInfoWindow {
    item: InventoryItem<ResourceMetadata>,
}

impl ItemInfoWindow {
    pub fn new(item: InventoryItem<ResourceMetadata>) -> Self {
        Self { item }
    }
}

impl CustomWindow<ClientState> for ItemInfoWindow {
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::ItemInfo)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        let name = if self.item.metadata.name.is_empty() {
            format!("道具 #{}", self.item.item_id.0)
        } else {
            self.item.metadata.name.clone()
        };

        let amount_text = match &self.item.details {
            InventoryItemDetails::Regular { amount, .. } => format!("數量: {}", amount),
            InventoryItemDetails::Equippable { .. } => "可裝備物品".to_string(),
        };

        window! {
            title: "道具說明",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            closable: true,
            elements: (
                text! {
                    text: name,
                    color: Color::rgb_u8(255, 230, 180),
                    overflow_behavior: OverflowBehavior::Shrink,
                },
                text! {
                    text: amount_text,
                    color: Color::rgb_u8(220, 220, 220),
                    overflow_behavior: OverflowBehavior::Shrink,
                },
                text! {
                    text: format!("物品 ID: {}", self.item.item_id.0),
                    color: Color::rgb_u8(160, 160, 160),
                    overflow_behavior: OverflowBehavior::Shrink,
                },
            ),
        }
    }
}
