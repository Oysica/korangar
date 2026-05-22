use std::cell::{Cell, UnsafeCell};

use korangar_interface::element::store::{ElementStore, ElementStoreMut};
use korangar_interface::element::{BaseLayoutInfo, Element};
use korangar_interface::layout::area::Area;
use korangar_interface::layout::{Resolvers, WindowLayout, with_single_resolver};
use korangar_interface::prelude::{HorizontalAlignment, VerticalAlignment};
use korangar_interface::window::{CustomWindow, Window};
use rust_state::{Path, State};

use crate::graphics::{Color, CornerDiameter, ShadowPadding};
use crate::interface::windows::WindowClass;
use crate::loaders::{FontSize, OverflowBehavior};
use crate::state::ClientState;
use crate::state::theme::InterfaceThemeType;
use crate::world::{Player, PlayerPathExt};

/// A single horizontal status bar with the player's base and job experience
/// joined at the center, level labels overlaid in the middle.
struct DualExperienceBar<BL, BE, NBE, JL, JE, NJE> {
    base_level_path: BL,
    base_exp_path: BE,
    next_base_exp_path: NBE,
    job_level_path: JL,
    job_exp_path: JE,
    next_job_exp_path: NJE,
    cached_text: UnsafeCell<String>,
    cached_levels: Cell<Option<(usize, usize)>>,
}

impl<BL, BE, NBE, JL, JE, NJE> DualExperienceBar<BL, BE, NBE, JL, JE, NJE> {
    fn new(
        base_level_path: BL,
        base_exp_path: BE,
        next_base_exp_path: NBE,
        job_level_path: JL,
        job_exp_path: JE,
        next_job_exp_path: NJE,
    ) -> Self {
        Self {
            base_level_path,
            base_exp_path,
            next_base_exp_path,
            job_level_path,
            job_exp_path,
            next_job_exp_path,
            cached_text: UnsafeCell::default(),
            cached_levels: Cell::default(),
        }
    }
}

impl<BL, BE, NBE, JL, JE, NJE> Element<ClientState> for DualExperienceBar<BL, BE, NBE, JL, JE, NJE>
where
    BL: Path<ClientState, usize>,
    BE: Path<ClientState, u64>,
    NBE: Path<ClientState, u64>,
    JL: Path<ClientState, usize>,
    JE: Path<ClientState, u64>,
    NJE: Path<ClientState, u64>,
{
    type LayoutInfo = BaseLayoutInfo;

    fn create_layout_info(
        &mut self,
        state: &State<ClientState>,
        _: ElementStoreMut,
        resolvers: &mut dyn Resolvers<ClientState>,
    ) -> Self::LayoutInfo {
        with_single_resolver(resolvers, |resolver| {
            let base_level = *state.get(&self.base_level_path);
            let job_level = *state.get(&self.job_level_path);
            let last = self.cached_levels.get();

            if last != Some((base_level, job_level)) {
                unsafe {
                    *self.cached_text.get() = format!("Lv {base_level}  |  Job {job_level}");
                };
                self.cached_levels.set(Some((base_level, job_level)));
            }

            let area = resolver.with_height(14.0);
            Self::LayoutInfo { area }
        })
    }

    fn lay_out<'a>(
        &'a self,
        state: &'a State<ClientState>,
        _: ElementStore<'a>,
        layout_info: &'a Self::LayoutInfo,
        layout: &mut WindowLayout<'a, ClientState>,
    ) {
        let base_exp = *state.get(&self.base_exp_path);
        let next_base_exp = *state.get(&self.next_base_exp_path);
        let job_exp = *state.get(&self.job_exp_path);
        let next_job_exp = *state.get(&self.next_job_exp_path);

        let half_width = layout_info.area.width / 2.0;
        let corner = CornerDiameter::uniform(3.0);
        let background_color = Color::rgba_u8(20, 20, 20, 220);
        let base_color = Color::rgba_u8(245, 158, 11, 230);
        let job_color = Color::rgba_u8(34, 197, 211, 230);

        layout.add_rectangle(
            layout_info.area,
            corner,
            background_color,
            Color::TRANSPARENT,
            ShadowPadding::default(),
        );

        let base_fraction = if next_base_exp == 0 {
            0.0
        } else {
            (base_exp as f32 / next_base_exp as f32).clamp(0.0, 1.0)
        };
        if base_fraction > 0.0 {
            let fill_width = half_width * base_fraction;
            let fill_area = Area {
                left: layout_info.area.left + (half_width - fill_width),
                top: layout_info.area.top,
                width: fill_width,
                height: layout_info.area.height,
            };
            layout.add_rectangle(fill_area, corner, base_color, Color::TRANSPARENT, ShadowPadding::default());
        }

        let job_fraction = if next_job_exp == 0 {
            0.0
        } else {
            (job_exp as f32 / next_job_exp as f32).clamp(0.0, 1.0)
        };
        if job_fraction > 0.0 {
            let fill_width = half_width * job_fraction;
            let fill_area = Area {
                left: layout_info.area.left + half_width,
                top: layout_info.area.top,
                width: fill_width,
                height: layout_info.area.height,
            };
            layout.add_rectangle(fill_area, corner, job_color, Color::TRANSPARENT, ShadowPadding::default());
        }

        let text: &'a str = unsafe { (*self.cached_text.get()).as_str() };

        // Center pill around the level text.
        let pill_width = 130.0_f32.min(layout_info.area.width);
        let pill_height = layout_info.area.height + 6.0;
        let pill_area = Area {
            left: layout_info.area.left + (layout_info.area.width - pill_width) / 2.0,
            top: layout_info.area.top - 3.0,
            width: pill_width,
            height: pill_height,
        };
        layout.add_rectangle(
            pill_area,
            CornerDiameter::uniform(pill_height / 2.0),
            Color::rgba_u8(15, 15, 15, 235),
            Color::rgba_u8(255, 255, 255, 80),
            ShadowPadding::diagonal(1.0, 1.0),
        );

        layout.add_text(
            pill_area,
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

pub struct ExperienceBarWindow<P> {
    player_path: P,
}

impl<P> ExperienceBarWindow<P> {
    pub fn new(player_path: P) -> Self {
        Self { player_path }
    }
}

impl<P> CustomWindow<ClientState> for ExperienceBarWindow<P>
where
    P: Path<ClientState, Player> + Copy,
{
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::ExperienceBar)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        let base_level_path = self.player_path.base_level();
        let job_level_path = self.player_path.job_level();
        let base_exp_path = self.player_path.base_experience();
        let next_base_exp_path = self.player_path.next_base_experience();
        let job_exp_path = self.player_path.job_experience();
        let next_job_exp_path = self.player_path.next_job_experience();

        window! {
            title: "",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            title_height: 0.0,
            border: 0.0,
            background_color: Color::TRANSPARENT,
            shadow_padding: ShadowPadding::default(),
            corner_diameter: CornerDiameter::uniform(0.0),
            minimum_width: 600.0,
            elements: (
                DualExperienceBar::new(
                    base_level_path,
                    base_exp_path,
                    next_base_exp_path,
                    job_level_path,
                    job_exp_path,
                    next_job_exp_path,
                ),
            ),
        }
    }
}
