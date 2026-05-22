use std::cell::{Cell, UnsafeCell};

use korangar_interface::element::store::{ElementStore, ElementStoreMut};
use korangar_interface::element::{BaseLayoutInfo, Element};
use korangar_interface::layout::area::Area;
use korangar_interface::layout::{Resolvers, WindowLayout, with_single_resolver};
use korangar_interface::prelude::{HorizontalAlignment, VerticalAlignment};
use korangar_interface::window::{CustomWindow, Window};
use ragnarok_packets::JobId;
use rust_state::{Path, PathExt, Selector, State};

use crate::graphics::{Color, CornerDiameter, ShadowPadding};
use crate::input::InputEvent;
use crate::interface::windows::WindowClass;
use crate::loaders::{FontSize, OverflowBehavior};
use crate::state::localization::LocalizationPathExt;
use crate::state::theme::InterfaceThemeType;
use crate::state::{ClientState, ClientStatePathExt, client_state};
use crate::world::{CommonPathExt, Player, PlayerPathExt};

/// Horizontal progress bar with overlaid `current/max  percent%` text.
struct ProgressBar<C, M> {
    current_path: C,
    max_path: M,
    color: Color,
    cached_text: UnsafeCell<String>,
    cached: Cell<Option<(usize, usize)>>,
}

impl<C, M> ProgressBar<C, M> {
    fn new(current_path: C, max_path: M, color: Color) -> Self {
        Self {
            current_path,
            max_path,
            color,
            cached_text: UnsafeCell::default(),
            cached: Cell::default(),
        }
    }
}

impl<C, M> Element<ClientState> for ProgressBar<C, M>
where
    C: Path<ClientState, usize>,
    M: Path<ClientState, usize>,
{
    type LayoutInfo = BaseLayoutInfo;

    fn create_layout_info(
        &mut self,
        state: &State<ClientState>,
        _: ElementStoreMut,
        resolvers: &mut dyn Resolvers<ClientState>,
    ) -> Self::LayoutInfo {
        with_single_resolver(resolvers, |resolver| {
            let current = *state.get(&self.current_path);
            let max = *state.get(&self.max_path);
            if self.cached.get() != Some((current, max)) {
                let percent = if max == 0 {
                    0.0
                } else {
                    (current as f32 / max as f32 * 100.0).clamp(0.0, 100.0)
                };
                unsafe { *self.cached_text.get() = format!("{current}/{max}    {percent:.0}%") };
                self.cached.set(Some((current, max)));
            }
            Self::LayoutInfo {
                area: resolver.with_height(22.0),
            }
        })
    }

    fn lay_out<'a>(
        &'a self,
        state: &'a State<ClientState>,
        _: ElementStore<'a>,
        layout_info: &'a Self::LayoutInfo,
        layout: &mut WindowLayout<'a, ClientState>,
    ) {
        let current = *state.get(&self.current_path);
        let max = *state.get(&self.max_path);

        layout.add_rectangle(
            layout_info.area,
            CornerDiameter::uniform(2.0),
            Color::rgba_u8(15, 15, 15, 230),
            Color::rgba_u8(255, 255, 255, 60),
            ShadowPadding::diagonal(1.0, 1.0),
        );

        let fraction = if max == 0 {
            0.0
        } else {
            (current as f32 / max as f32).clamp(0.0, 1.0)
        };
        if fraction > 0.0 {
            let fill = Area {
                left: layout_info.area.left,
                top: layout_info.area.top,
                width: layout_info.area.width * fraction,
                height: layout_info.area.height,
            };
            layout.add_rectangle(fill, CornerDiameter::uniform(2.0), self.color, Color::TRANSPARENT, ShadowPadding::default());
        }

        let text: &'a str = unsafe { (*self.cached_text.get()).as_str() };
        layout.add_text(
            layout_info.area,
            text,
            FontSize(10.0),
            Color::WHITE,
            Color::rgba_u8(0, 0, 0, 200),
            HorizontalAlignment::Center { offset: 0.0, border: 0.0 },
            VerticalAlignment::Center { offset: 0.0 },
            OverflowBehavior::Shrink,
        );
    }
}

struct LevelTextSelector<A> {
    path: A,
    cached: UnsafeCell<String>,
    last: Cell<Option<usize>>,
}

impl<A> LevelTextSelector<A> {
    fn new(path: A) -> Self {
        Self {
            path,
            cached: UnsafeCell::default(),
            last: Cell::default(),
        }
    }
}

impl<A> Selector<ClientState, String> for LevelTextSelector<A>
where
    A: Path<ClientState, usize>,
{
    fn select<'a>(&'a self, state: &'a ClientState) -> Option<&'a String> {
        let value = *self.path.follow_safe(state);
        if self.last.get() != Some(value) {
            unsafe { *self.cached.get() = format!("Lv {value}") };
            self.last.set(Some(value));
        }
        Some(unsafe { &*self.cached.get() })
    }
}

struct JobTextSelector<A> {
    path: A,
    cached: UnsafeCell<String>,
    last: Cell<Option<u16>>,
}

impl<A> JobTextSelector<A> {
    fn new(path: A) -> Self {
        Self {
            path,
            cached: UnsafeCell::default(),
            last: Cell::default(),
        }
    }
}

impl<A> Selector<ClientState, String> for JobTextSelector<A>
where
    A: Path<ClientState, JobId>,
{
    fn select<'a>(&'a self, state: &'a ClientState) -> Option<&'a String> {
        let value = self.path.follow_safe(state).0;
        if self.last.get() != Some(value) {
            unsafe { *self.cached.get() = format!("Job {value}") };
            self.last.set(Some(value));
        }
        Some(unsafe { &*self.cached.get() })
    }
}

pub struct CharacterOverviewWindow<N, P> {
    player_name_path: N,
    player_path: P,
}

impl<N, P> CharacterOverviewWindow<N, P> {
    pub fn new(player_name_path: N, player_path: P) -> Self {
        Self {
            player_name_path,
            player_path,
        }
    }
}

impl<N, P> CustomWindow<ClientState> for CharacterOverviewWindow<N, P>
where
    N: Path<ClientState, String>,
    P: Path<ClientState, Player> + Copy,
{
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::CharacterOverview)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        let common_path = self.player_path.common();
        let hp_path = common_path.health_points();
        let max_hp_path = common_path.maximum_health_points();
        let sp_path = self.player_path.spell_points();
        let max_sp_path = self.player_path.maximum_spell_points();
        let base_level_path = self.player_path.base_level();
        let job_id_path = common_path.job_id();
        window! {
            title: "",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            title_height: 0.0,
            minimum_width: 320.0,
            maximum_width: 320.0,
            elements: (
                fragment! {
                    gaps: 6.0,
                    children: (
                        split! {
                            gaps: 6.0,
                            children: (
                                text! {
                                    text: LevelTextSelector::new(base_level_path),
                                    color: Color::rgb_u8(13, 231, 255),
                                    horizontal_alignment: HorizontalAlignment::Left { offset: 0.0, border: 4.0 },
                                    overflow_behavior: OverflowBehavior::Shrink,
                                },
                                text! {
                                    text: self.player_name_path,
                                    color: Color::rgb_u8(255, 144, 13),
                                    horizontal_alignment: HorizontalAlignment::Center { offset: 0.0, border: 0.0 },
                                    overflow_behavior: OverflowBehavior::Shrink,
                                },
                                text! {
                                    text: JobTextSelector::new(job_id_path),
                                    color: Color::rgb_u8(220, 220, 220),
                                    horizontal_alignment: HorizontalAlignment::Right { offset: 0.0, border: 0.0 },
                                    overflow_behavior: OverflowBehavior::Shrink,
                                },
                            ),
                        },
                        fragment! {
                            gaps: 4.0,
                            children: (
                                ProgressBar::new(hp_path, max_hp_path, Color::rgba_u8(220, 50, 50, 230)),
                                ProgressBar::new(sp_path, max_sp_path, Color::rgba_u8(50, 130, 220, 230)),
                            ),
                        },
                    ),
                },
                split! {
                    gaps: theme().window().gaps(),
                    children: (
                        button! {
                            text: client_state().localization().inventory_button_text(),
                            event: InputEvent::ToggleInventoryWindow,
                        },
                        button! {
                            text: client_state().localization().equipment_button_text(),
                            event: InputEvent::ToggleEquipmentWindow,
                        },
                        button! {
                            text: client_state().localization().skill_tree_button_text(),
                            event: InputEvent::ToggleSkillTreeWindow,
                        },
                        button! {
                            text: client_state().localization().stats_button_text(),
                            event: InputEvent::ToggleStatsWindow,
                        },
                    ),
                },
                split! {
                    gaps: theme().window().gaps(),
                    children: (
                        button! {
                            text: client_state().localization().friend_list_button_text(),
                            event: InputEvent::ToggleFriendListWindow,
                        },
                        button! {
                            text: client_state().localization().menu_button_text(),
                            event: InputEvent::ToggleMenuWindow,
                        },
                    ),
                },
            ),
        }
    }
}

