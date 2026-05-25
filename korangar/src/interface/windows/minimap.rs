use korangar_interface::element::store::{ElementStore, ElementStoreMut};
use korangar_interface::element::{BaseLayoutInfo, Element};
use korangar_interface::layout::area::Area;
use korangar_interface::layout::{Resolvers, WindowLayout, with_single_resolver};
use korangar_interface::window::{CustomWindow, Window};
use rust_state::{Path, State};

use crate::graphics::{Color, CornerDiameter, ShadowPadding};
use crate::interface::windows::WindowClass;
use crate::renderer::LayoutExt;
use crate::state::ClientState;
use crate::state::minimap::MinimapState;
use crate::state::theme::InterfaceThemeType;
use crate::world::Player;

const MINIMAP_DISPLAY_SIZE: f32 = 200.0;
const PLAYER_DOT_SIZE: f32 = 6.0;

/// Custom element that renders the minimap texture with a red marker for the
/// current player position. Falls back to a flat background while a map is
/// still loading.
struct MinimapElement<M, P> {
    minimap_path: M,
    player_path: P,
}

impl<M, P> Element<ClientState> for MinimapElement<M, P>
where
    M: Path<ClientState, MinimapState>,
    P: Path<ClientState, Player, false>,
{
    type LayoutInfo = BaseLayoutInfo;

    fn create_layout_info(
        &mut self,
        _: &State<ClientState>,
        _: ElementStoreMut,
        resolvers: &mut dyn Resolvers<ClientState>,
    ) -> Self::LayoutInfo {
        with_single_resolver(resolvers, |resolver| {
            let area = resolver.with_height(MINIMAP_DISPLAY_SIZE);
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
        let area = layout_info.area;

        // Use a square viewport centered in the available area so the minimap
        // does not stretch.
        let viewport_size = area.width.min(area.height);
        let viewport = Area {
            left: area.left + (area.width - viewport_size) / 2.0,
            top: area.top + (area.height - viewport_size) / 2.0,
            width: viewport_size,
            height: viewport_size,
        };

        layout.add_rectangle(
            viewport,
            CornerDiameter::uniform(8.0),
            Color::rgba_u8(20, 20, 20, 90),
            Color::rgba_u8(0, 0, 0, 60),
            ShadowPadding::diagonal(2.0, 4.0),
        );

        let minimap = state.get(&self.minimap_path);

        if let Some(texture) = minimap.texture() {
            layout.add_texture(viewport, texture.clone(), Color::rgba_u8(255, 255, 255, 170), false);
        }

        // Player marker: scale tile_position into the viewport. The minimap
        // texture is Y-flipped (build_minimap_image) so 0,0 in tile space is
        // the bottom-left; mirror that here.
        let map_width = minimap.map_width_tiles();
        let map_height = minimap.map_height_tiles();
        if map_width == 0 || map_height == 0 {
            return;
        }

        let Some(player) = state.try_get(&self.player_path) else {
            return;
        };
        let tile = player.common.tile_position;

        let normalized_x = (tile.x as f32 + 0.5) / map_width as f32;
        let normalized_y = 1.0 - (tile.y as f32 + 0.5) / map_height as f32;

        let dot_center_x = viewport.left + normalized_x * viewport.width;
        let dot_center_y = viewport.top + normalized_y * viewport.height;

        let dot_area = Area {
            left: dot_center_x - PLAYER_DOT_SIZE / 2.0,
            top: dot_center_y - PLAYER_DOT_SIZE / 2.0,
            width: PLAYER_DOT_SIZE,
            height: PLAYER_DOT_SIZE,
        };
        // Within a single layer, all rectangles are drawn before any textures,
        // so a rectangle pushed AFTER the minimap texture would still appear
        // below it. Push the marker into a fresh layer so it renders on top.
        layout.with_layer(|layout| {
            layout.add_rectangle(
                dot_area,
                CornerDiameter::uniform(PLAYER_DOT_SIZE / 2.0),
                Color::rgb_u8(255, 60, 60),
                Color::rgba_u8(0, 0, 0, 160),
                ShadowPadding::diagonal(1.0, 2.0),
            );
        });
    }
}

pub struct MinimapWindow<M, P> {
    minimap_path: M,
    player_path: P,
}

impl<M, P> MinimapWindow<M, P> {
    pub fn new(minimap_path: M, player_path: P) -> Self {
        Self {
            minimap_path,
            player_path,
        }
    }
}

impl<M, P> CustomWindow<ClientState> for MinimapWindow<M, P>
where
    M: Path<ClientState, MinimapState> + Copy,
    P: Path<ClientState, Player, false> + Copy,
{
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::Minimap)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        let element = MinimapElement {
            minimap_path: self.minimap_path,
            player_path: self.player_path,
        };

        window! {
            title: "",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            title_height: 0.0,
            border: 0.0,
            background_color: Color::TRANSPARENT,
            shadow_padding: ShadowPadding::default(),
            corner_diameter: CornerDiameter::uniform(0.0),
            // Override the theme defaults so the window doesn't get inflated to
            // the theme's 300 px minimum and extend past the screen edge.
            minimum_width: 200.0,
            maximum_width: 200.0,
            elements: (
                element,
            ),
        }
    }
}
