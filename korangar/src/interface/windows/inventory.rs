use std::cell::{Cell, UnsafeCell};

use korangar_components::item_box;
use korangar_interface::window::{CustomWindow, Window};
use rust_state::{Path, PathExt, Selector, State, VecIndexExt};

use crate::ItemSource;
use crate::graphics::Color;
use crate::interface::windows::WindowClass;
use crate::loaders::OverflowBehavior;
use crate::state::inventory::{Inventory, InventoryPathExt};
use crate::state::localization::LocalizationPathExt;
use crate::state::theme::InterfaceThemeType;
use crate::state::{ClientState, ClientStatePathExt, client_state};
use crate::world::{Player, PlayerPathExt};

const SECTION_ROWS: usize = 4;
const SECTION_COLUMNS: usize = 10;

/// "{label}: {current}/{max}" composite selector for the weight row.
struct WeightLabelSelector<C, M, L> {
    current_path: C,
    max_path: M,
    label_path: L,
    cached: UnsafeCell<String>,
    last: Cell<Option<(u32, u32, String)>>,
}

impl<C, M, L> WeightLabelSelector<C, M, L> {
    fn new(current_path: C, max_path: M, label_path: L) -> Self {
        Self {
            current_path,
            max_path,
            label_path,
            cached: UnsafeCell::default(),
            last: Cell::default(),
        }
    }
}

impl<C, M, L> Selector<ClientState, String> for WeightLabelSelector<C, M, L>
where
    C: Path<ClientState, u32>,
    M: Path<ClientState, u32>,
    L: Path<ClientState, String>,
{
    fn select<'a>(&'a self, state: &'a ClientState) -> Option<&'a String> {
        let current = *self.current_path.follow_safe(state);
        let max = *self.max_path.follow_safe(state);
        let label = self.label_path.follow_safe(state).clone();
        let key = (current, max, label.clone());
        if self.last.replace(Some(key.clone())) != Some(key) {
            unsafe { *self.cached.get() = format!("{label}: {current}/{max}") };
        }
        Some(unsafe { &*self.cached.get() })
    }
}

/// "{label}: {value-with-commas}" composite selector for the zeny row.
struct ZenyLabelSelector<V, L> {
    value_path: V,
    label_path: L,
    cached: UnsafeCell<String>,
    last: Cell<Option<(u32, String)>>,
}

impl<V, L> ZenyLabelSelector<V, L> {
    fn new(value_path: V, label_path: L) -> Self {
        Self {
            value_path,
            label_path,
            cached: UnsafeCell::default(),
            last: Cell::default(),
        }
    }
}

impl<V, L> Selector<ClientState, String> for ZenyLabelSelector<V, L>
where
    V: Path<ClientState, u32>,
    L: Path<ClientState, String>,
{
    fn select<'a>(&'a self, state: &'a ClientState) -> Option<&'a String> {
        let value = *self.value_path.follow_safe(state);
        let label = self.label_path.follow_safe(state).clone();
        let key = (value, label.clone());
        if self.last.replace(Some(key.clone())) != Some(key) {
            let raw = value.to_string();
            let bytes = raw.as_bytes();
            let len = bytes.len();
            let mut grouped = String::with_capacity(len + len / 3);
            for (i, &b) in bytes.iter().enumerate() {
                if i > 0 && (len - i) % 3 == 0 {
                    grouped.push(',');
                }
                grouped.push(b as char);
            }
            unsafe { *self.cached.get() = format!("{label}: {grouped}") };
        }
        Some(unsafe { &*self.cached.get() })
    }
}

pub struct InventoryWindow<I, Q> {
    inventory_path: I,
    player_path: Q,
}

impl<I, Q> InventoryWindow<I, Q> {
    pub fn new(inventory_path: I, player_path: Q) -> Self {
        Self {
            inventory_path,
            player_path,
        }
    }
}

impl<I, Q> CustomWindow<ClientState> for InventoryWindow<I, Q>
where
    I: Path<ClientState, Inventory> + Copy,
    Q: Path<ClientState, Player> + Copy,
{
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::Inventory)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        let weight_path = self.player_path.weight();
        let max_weight_path = self.player_path.maximum_weight();
        let zeny_path = self.player_path.zeny();

        let consumable_path = self.inventory_path.consumable_items();
        let equipment_path = self.inventory_path.equipment_items();
        let other_path = self.inventory_path.other_items();
        let personal_path = self.inventory_path.personal_items();
        let selected_tab = self.inventory_path.selected_tab();

        let consumable_grid = std::array::from_fn::<_, SECTION_ROWS, _>(|row| {
            split! {
                gaps: theme().window().gaps(),
                children: std::array::from_fn::<_, SECTION_COLUMNS, _>(|column| {
                    let path = consumable_path.index(row * SECTION_COLUMNS + column);
                    item_box! { item_path: path, source: ItemSource::Inventory }
                }),
            }
        });
        let equipment_grid = std::array::from_fn::<_, SECTION_ROWS, _>(|row| {
            split! {
                gaps: theme().window().gaps(),
                children: std::array::from_fn::<_, SECTION_COLUMNS, _>(|column| {
                    let path = equipment_path.index(row * SECTION_COLUMNS + column);
                    item_box! { item_path: path, source: ItemSource::Inventory }
                }),
            }
        });
        let other_grid = std::array::from_fn::<_, SECTION_ROWS, _>(|row| {
            split! {
                gaps: theme().window().gaps(),
                children: std::array::from_fn::<_, SECTION_COLUMNS, _>(|column| {
                    let path = other_path.index(row * SECTION_COLUMNS + column);
                    item_box! { item_path: path, source: ItemSource::Inventory }
                }),
            }
        });
        let personal_grid = std::array::from_fn::<_, SECTION_ROWS, _>(|row| {
            split! {
                gaps: theme().window().gaps(),
                children: std::array::from_fn::<_, SECTION_COLUMNS, _>(|column| {
                    let path = personal_path.index(row * SECTION_COLUMNS + column);
                    item_box! { item_path: path, source: ItemSource::Inventory }
                }),
            }
        });

        let tab_button = |index: usize, label: &'static str| {
            button! {
                text: label,
                event: move |state: &State<ClientState>, _: &mut EventQueue<ClientState>| {
                    state.update_value(selected_tab, index);
                },
                disabled: ComputedSelector::new_default(move |state: &ClientState| *selected_tab.follow_safe(state) == index),
            }
        };

        window! {
            title: client_state().localization().inventory_window_title(),
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            closable: true,
            elements: (
                split! {
                    gaps: theme().window().gaps(),
                    children: (
                        text! {
                            text: WeightLabelSelector::new(weight_path, max_weight_path, client_state().localization().weight_text()),
                            color: Color::rgb_u8(220, 220, 220),
                            overflow_behavior: OverflowBehavior::Shrink,
                        },
                        text! {
                            text: ZenyLabelSelector::new(zeny_path, client_state().localization().zeny_text()),
                            color: Color::rgb_u8(255, 215, 0),
                            horizontal_alignment: HorizontalAlignment::Right { offset: 0.0, border: 3.0 },
                            overflow_behavior: OverflowBehavior::Shrink,
                        },
                    ),
                },
                split! {
                    gaps: theme().window().gaps(),
                    children: (
                        tab_button(0, "道具類"),
                        tab_button(1, "裝備類"),
                        tab_button(2, "其他類"),
                        tab_button(3, "個人"),
                    ),
                },
                either! {
                    selector: ComputedSelector::new_default(move |state: &ClientState| *selected_tab.follow_safe(state) < 2),
                    on_true: either! {
                        selector: ComputedSelector::new_default(move |state: &ClientState| *selected_tab.follow_safe(state) == 0),
                        on_true: consumable_grid,
                        on_false: equipment_grid,
                    },
                    on_false: either! {
                        selector: ComputedSelector::new_default(move |state: &ClientState| *selected_tab.follow_safe(state) == 2),
                        on_true: other_grid,
                        on_false: personal_grid,
                    },
                },
            ),
        }
    }
}
