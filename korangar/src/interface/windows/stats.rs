use std::sync::Arc;

use korangar_interface::element::store::{ElementStore, ElementStoreMut};
use korangar_interface::element::{BaseLayoutInfo, Element};
use korangar_interface::event::{ClickHandler, EventQueue};
use korangar_interface::layout::area::Area;
use korangar_interface::layout::{MouseButton, Resolvers, WindowLayout, with_single_resolver};
use korangar_interface::prelude::{HorizontalAlignment, VerticalAlignment};
use korangar_interface::window::{CustomWindow, Window};
use ragnarok_packets::StatUpType;
use rust_state::{Path, State};

use crate::graphics::{Color, Texture};
use crate::input::InputEvent;
use crate::interface::windows::WindowClass;
use crate::loaders::{FontSize, OverflowBehavior};
use crate::renderer::LayoutExt;
use crate::state::ClientState;
use crate::state::theme::InterfaceThemeType;
use crate::world::Player;

const TITLEBAR_HEIGHT: f32 = 17.0;
const PANEL_WIDTH: f32 = 280.0;
const PANEL_HEIGHT: f32 = 103.0;
const ROW_HEIGHT: f32 = PANEL_HEIGHT / 6.0;
/// Vertical distance between row centers in the bg image. Slightly smaller
/// than ROW_HEIGHT to keep later rows from drifting below their box.
const ROW_STRIDE: f32 = 16.2;
/// Vertical offset applied to all rows. Positive shifts text downward.
const ROWS_Y_OFFSET: f32 = 3.0;
const TEXT_FONT_SIZE: f32 = 11.0;

/// Pixel layout of the empty value boxes inside `statwin_bg.png`. These are
/// estimates measured against the image — tune if values don't sit perfectly.
const STAT_VALUE_LEFT: f32 = 34.0;
const STAT_VALUE_WIDTH: f32 = 18.0;
/// Bonus (`+N`) cell — sits flush to the right of the base value.
const STAT_BONUS_TEXT_LEFT: f32 = 52.0;
const STAT_BONUS_TEXT_WIDTH: f32 = 22.0;
/// Cost (icon + number) cell — further right with a clear gap from bonus.
const STAT_BONUS_LEFT: f32 = 80.0;
const STAT_BONUS_WIDTH: f32 = 28.0;
const INFO1_VALUE_LEFT: f32 = 130.0;
const INFO1_VALUE_WIDTH: f32 = 60.0;
const INFO2_VALUE_LEFT: f32 = 215.0;
const INFO2_VALUE_WIDTH: f32 = 55.0;
/// Right edge used by single-value rows (Status Point / Guild) — value text is
/// right-aligned to here so it sits flush against the panel right.
const INFO_FULL_VALUE_LEFT: f32 = 170.0;
const INFO_FULL_VALUE_WIDTH: f32 = 100.0;

/// 11×11 close button placed at the top-right of the titlebar.
const CLOSE_BUTTON_SIZE: f32 = 11.0;
const CLOSE_BUTTON_RIGHT_MARGIN: f32 = 3.0;
const CLOSE_BUTTON_TOP_MARGIN: f32 = 3.0;

/// 8×8 stat-up arrow icon.
const ADD_ICON_SIZE: f32 = 8.0;

/// Y position of a row's center (0-indexed). Add panel top + this for absolute
/// vertical position.
fn row_center_y(row: usize) -> f32 {
    ROW_STRIDE * (row as f32 + 0.5)
}

/// Right-click style click handler — increments a single stat point.
struct StatUpHandler {
    stat_type: StatUpType,
}

impl ClickHandler<ClientState> for StatUpHandler {
    fn handle_click(&self, _: &State<ClientState>, queue: &mut EventQueue<ClientState>) {
        queue.queue(InputEvent::StatUp { stat_type: self.stat_type.clone() });
    }
}

/// Closes the stats window. Reuses the existing toggle event, which closes
/// the window when it's currently open.
struct CloseStatsHandler;

impl ClickHandler<ClientState> for CloseStatsHandler {
    fn handle_click(&self, _: &State<ClientState>, queue: &mut EventQueue<ClientState>) {
        queue.queue(InputEvent::ToggleStatsWindow);
    }
}

/// The full stats panel — one custom element renders titlebar + bg images and
/// overlays all dynamic text + buttons at fixed pixel offsets.
pub struct StatsPanel<P> {
    player_path: P,
    titlebar_texture: Arc<Texture>,
    statwin_texture: Arc<Texture>,
    close_off_texture: Arc<Texture>,
    close_on_texture: Arc<Texture>,
    add_texture: Arc<Texture>,
    handlers: [StatUpHandler; 6],
    close_handler: CloseStatsHandler,
}

impl<P> StatsPanel<P> {
    pub fn new(
        player_path: P,
        titlebar_texture: Arc<Texture>,
        statwin_texture: Arc<Texture>,
        close_off_texture: Arc<Texture>,
        close_on_texture: Arc<Texture>,
        add_texture: Arc<Texture>,
    ) -> Self {
        Self {
            player_path,
            titlebar_texture,
            statwin_texture,
            close_off_texture,
            close_on_texture,
            add_texture,
            handlers: [
                StatUpHandler { stat_type: StatUpType::Strength { amount: 1 } },
                StatUpHandler { stat_type: StatUpType::Agility { amount: 1 } },
                StatUpHandler { stat_type: StatUpType::Vitality { amount: 1 } },
                StatUpHandler { stat_type: StatUpType::Intelligence { amount: 1 } },
                StatUpHandler { stat_type: StatUpType::Dexterity { amount: 1 } },
                StatUpHandler { stat_type: StatUpType::Luck { amount: 1 } },
            ],
            close_handler: CloseStatsHandler,
        }
    }
}

impl<P> Element<ClientState> for StatsPanel<P>
where
    P: Path<ClientState, Player>,
{
    type LayoutInfo = BaseLayoutInfo;

    fn create_layout_info(
        &mut self,
        _: &State<ClientState>,
        _: ElementStoreMut,
        resolvers: &mut dyn Resolvers<ClientState>,
    ) -> Self::LayoutInfo {
        with_single_resolver(resolvers, |resolver| Self::LayoutInfo {
            area: resolver.with_height(PANEL_HEIGHT),
        })
    }

    fn lay_out<'a>(
        &'a self,
        state: &'a State<ClientState>,
        _: ElementStore<'a>,
        layout_info: &'a Self::LayoutInfo,
        layout: &mut WindowLayout<'a, ClientState>,
    ) {
        let origin_x = layout_info.area.left;
        // Our element's area starts BELOW the window title (because the window
        // reserves title_height pixels at the top). Draw the titlebar texture
        // by extending upward into that reserved space — the children layer
        // is clipped to the full window area so this is allowed.
        let panel_top = layout_info.area.top;
        let titlebar_top = panel_top - TITLEBAR_HEIGHT;

        // Titlebar background.
        let titlebar_area = Area {
            left: origin_x,
            top: titlebar_top,
            width: PANEL_WIDTH,
            height: TITLEBAR_HEIGHT,
        };
        layout.add_texture(titlebar_area, self.titlebar_texture.clone(), Color::WHITE, false);

        // Render the title text ourselves over the texture. Fake-bold via
        // double-draw with a 1px x-offset to keep antialiased edges from
        // looking grey.
        let title_shifted = Area {
            left: titlebar_area.left + 1.0,
            ..titlebar_area
        };
        for area in [title_shifted, titlebar_area] {
            layout.add_text(
                area,
                "人物能力屬性狀態",
                FontSize(TEXT_FONT_SIZE),
                Color::rgba_u8(0, 0, 0, 255),
                Color::TRANSPARENT,
                HorizontalAlignment::Left { offset: 10.0, border: 0.0 },
                VerticalAlignment::Center { offset: 0.0 },
                OverflowBehavior::Shrink,
            );
        }

        // Close button on the top-right of the titlebar. Swap textures while
        // the mouse is hovering and register a click that closes the window.
        let close_area = Area {
            left: origin_x + PANEL_WIDTH - CLOSE_BUTTON_SIZE - CLOSE_BUTTON_RIGHT_MARGIN,
            top: titlebar_top + CLOSE_BUTTON_TOP_MARGIN,
            width: CLOSE_BUTTON_SIZE,
            height: CLOSE_BUTTON_SIZE,
        };
        let close_hovered = close_area.check().run(layout);
        let close_texture = if close_hovered {
            self.close_on_texture.clone()
        } else {
            self.close_off_texture.clone()
        };
        layout.add_texture(close_area, close_texture, Color::WHITE, false);
        if close_hovered {
            layout.register_click_handler(MouseButton::Left, &self.close_handler);
        }

        // Stats background (labels and boxes are baked into the image).
        let panel_area = Area {
            left: origin_x,
            top: panel_top,
            width: PANEL_WIDTH,
            height: PANEL_HEIGHT,
        };
        layout.add_texture(panel_area, self.statwin_texture.clone(), Color::WHITE, false);

        let player = state.get(&self.player_path);

        // Stat rows: [value][bonus][+1 click area]
        let stat_data: [(i32, i32, u8, usize); 6] = [
            (player.strength, player.bonus_strength, player.strength_stat_points_cost, 0),
            (player.agility, player.bonus_agility, player.agility_stat_points_cost, 1),
            (player.vitality, player.bonus_vitality, player.vitality_stat_points_cost, 2),
            (player.intelligence, player.bonus_intelligence, player.intelligence_stat_points_cost, 3),
            (player.dexterity, player.bonus_dexterity, player.dexterity_stat_points_cost, 4),
            (player.luck, player.bonus_luck, player.luck_stat_points_cost, 5),
        ];

        for (value, bonus, cost, row) in stat_data {
            let cy = panel_top + row_center_y(row) + ROWS_Y_OFFSET;

            // Cell 1: base value + bonus, packed flush-left.
            let value_area = Area {
                left: origin_x + STAT_VALUE_LEFT,
                top: cy - ROW_HEIGHT * 0.5,
                width: STAT_VALUE_WIDTH,
                height: ROW_HEIGHT,
            };
            self.draw_text(
                layout,
                value_area,
                value.to_string(),
                HorizontalAlignment::Left { offset: 4.0, border: 0.0 },
            );
            // Bonus right after the base value with no overlap on the cost
            // cell.
            let bonus_area = Area {
                left: origin_x + STAT_BONUS_TEXT_LEFT,
                top: cy - ROW_HEIGHT * 0.5,
                width: STAT_BONUS_TEXT_WIDTH,
                height: ROW_HEIGHT,
            };
            self.draw_text(
                layout,
                bonus_area,
                format!("{:+}", bonus),
                HorizontalAlignment::Left { offset: 0.0, border: 0.0 },
            );

            // Cell 2: sys_add icon (only if affordable) + cost number.
            let can_up = cost > 0 && player.stat_points >= cost as u32;
            let cost_area = Area {
                left: origin_x + STAT_BONUS_LEFT,
                top: cy - ROW_HEIGHT * 0.5,
                width: STAT_BONUS_WIDTH,
                height: ROW_HEIGHT,
            };
            if can_up {
                let icon_area = Area {
                    left: origin_x + STAT_BONUS_LEFT - 3.0,
                    top: cy - ADD_ICON_SIZE * 0.5,
                    width: ADD_ICON_SIZE,
                    height: ADD_ICON_SIZE,
                };
                layout.add_texture(icon_area, self.add_texture.clone(), Color::WHITE, false);
            }
            let cost_text = if cost == 0 {
                String::new()
            } else {
                cost.to_string()
            };
            // Place the cost text to the right of the icon slot. The extra
            // +3px shift keeps the digit away from the right separator line.
            let cost_text_area = Area {
                left: cost_area.left + ADD_ICON_SIZE + 5.0,
                top: cost_area.top,
                width: (cost_area.width - ADD_ICON_SIZE - 5.0).max(0.0),
                height: cost_area.height,
            };
            self.draw_text(
                layout,
                cost_text_area,
                cost_text,
                HorizontalAlignment::Left { offset: 0.0, border: 0.0 },
            );
            if cost_area.check().run(layout) && can_up {
                layout.register_click_handler(MouseButton::Left, &self.handlers[row]);
            }
        }

        // Right side combat-stat values. Layout per row (info1, info2):
        // 0 Atk(base+bonus) / Def(base+bonus)
        // 1 Matk(min~max)   / Mdef(base+bonus)
        // 2 Hit(value)      / Flee(flee+perfect_dodge)
        // 3 Critical(value) / Aspd(value)
        // 4 StatusPoint     / (empty)
        // 5 Guild           / (empty)
        let info1: [Option<String>; 6] = [
            Some(format!("{} + {}", player.attack_base, player.attack_bonus)),
            Some(format!("{} ~ {}", player.magic_attack_min, player.magic_attack_max)),
            Some(player.hit.to_string()),
            Some(player.critical.to_string()),
            Some(player.stat_points.to_string()),
            None,
        ];
        let info2: [Option<String>; 6] = [
            Some(format!("{} + {}", player.defense_base, player.defense_bonus)),
            Some(format!("{} + {}", player.magic_defense_base, player.magic_defense_bonus)),
            Some(format!("{} + {}", player.flee, player.perfect_dodge)),
            Some(player.attack_speed.to_string()),
            None,
            None,
        ];

        for row in 0..6 {
            let cy = panel_top + row_center_y(row) + ROWS_Y_OFFSET;
            if let Some(text) = &info1[row] {
                // Status Point / Guild rows have no info2 — use the wider
                // full-width area so the value sits flush against the right.
                let single_column = info2[row].is_none();
                let (left, width) = if single_column {
                    (INFO_FULL_VALUE_LEFT, INFO_FULL_VALUE_WIDTH)
                } else {
                    (INFO1_VALUE_LEFT, INFO1_VALUE_WIDTH)
                };
                let area = Area {
                    left: origin_x + left,
                    top: cy - ROW_HEIGHT * 0.5,
                    width,
                    height: ROW_HEIGHT,
                };
                self.draw_text(
                    layout,
                    area,
                    text.clone(),
                    HorizontalAlignment::Right { offset: 0.0, border: 6.0 },
                );
            }
            if let Some(text) = &info2[row] {
                let area = Area {
                    left: origin_x + INFO2_VALUE_LEFT,
                    top: cy - ROW_HEIGHT * 0.5,
                    width: INFO2_VALUE_WIDTH,
                    height: ROW_HEIGHT,
                };
                self.draw_text(
                    layout,
                    area,
                    text.clone(),
                    HorizontalAlignment::Right { offset: 0.0, border: 6.0 },
                );
            }
        }
    }
}

impl<P> StatsPanel<P> {
    fn draw_text<'a>(&'a self, layout: &mut WindowLayout<'a, ClientState>, area: Area, text: String, align: HorizontalAlignment) {
        let leaked: &'static str = Box::leak(text.into_boxed_str());
        // Fake-bold: draw the same text twice with a 1px horizontal offset so
        // antialiased edges add up to look fully black instead of grey.
        let shifted = Area {
            left: area.left + 1.0,
            ..area
        };
        layout.add_text(
            shifted,
            leaked,
            FontSize(TEXT_FONT_SIZE),
            Color::rgba_u8(0, 0, 0, 255),
            Color::TRANSPARENT,
            align,
            VerticalAlignment::Center { offset: 0.0 },
            OverflowBehavior::Shrink,
        );
        layout.add_text(
            area,
            leaked,
            FontSize(TEXT_FONT_SIZE),
            Color::rgba_u8(0, 0, 0, 255),
            Color::TRANSPARENT,
            align,
            VerticalAlignment::Center { offset: 0.0 },
            OverflowBehavior::Shrink,
        );
    }
}

#[derive(Default)]
pub struct StatsWindow<A> {
    player_path: A,
    titlebar_texture: Option<Arc<Texture>>,
    statwin_texture: Option<Arc<Texture>>,
    close_off_texture: Option<Arc<Texture>>,
    close_on_texture: Option<Arc<Texture>>,
    add_texture: Option<Arc<Texture>>,
}

impl<A> StatsWindow<A> {
    pub fn new(
        player_path: A,
        titlebar_texture: Arc<Texture>,
        statwin_texture: Arc<Texture>,
        close_off_texture: Arc<Texture>,
        close_on_texture: Arc<Texture>,
        add_texture: Arc<Texture>,
    ) -> Self {
        Self {
            player_path,
            titlebar_texture: Some(titlebar_texture),
            statwin_texture: Some(statwin_texture),
            close_off_texture: Some(close_off_texture),
            close_on_texture: Some(close_on_texture),
            add_texture: Some(add_texture),
        }
    }
}

impl<A> CustomWindow<ClientState> for StatsWindow<A>
where
    A: Path<ClientState, Player>,
{
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::Stats)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        let titlebar_texture = self.titlebar_texture.expect("titlebar texture must be provided");
        let statwin_texture = self.statwin_texture.expect("statwin texture must be provided");
        let close_off_texture = self.close_off_texture.expect("close-off texture must be provided");
        let close_on_texture = self.close_on_texture.expect("close-on texture must be provided");
        let add_texture = self.add_texture.expect("add texture must be provided");

        let panel = StatsPanel::new(
            self.player_path,
            titlebar_texture,
            statwin_texture,
            close_off_texture,
            close_on_texture,
            add_texture,
        );

        window! {
            // The panel itself paints the title text — leave this empty so the
            // theme's title renderer doesn't duplicate it at the wrong size.
            title: "",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            // title_height > 0 enables the built-in drag handler on the top
            // 17px strip; we still draw the titlebar texture ourselves from
            // within the custom panel.
            title_height: TITLEBAR_HEIGHT,
            border: 0.0,
            background_color: Color::TRANSPARENT,
            // We render our own close button overlay.
            closable: false,
            minimum_width: PANEL_WIDTH,
            maximum_width: PANEL_WIDTH,
            elements: (
                panel,
            ),
        }
    }
}
