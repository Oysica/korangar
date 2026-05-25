use std::collections::HashMap;

#[cfg(feature = "debug")]
use korangar_debug::logging::{Colorize, print_debug};
use korangar_interface::window::{Anchor, AnchorPoint};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

use super::WindowClass;
use crate::graphics::{ScreenPosition, ScreenSize};
use crate::state::ClientState;

/// Width of the bottom-anchored experience bar in pixels.
const EXPERIENCE_BAR_WIDTH: f32 = 600.0;
/// Height of the bottom-anchored experience bar (matches `DualExperienceBar`).
const EXPERIENCE_BAR_HEIGHT: f32 = 14.0;
/// Margin between the experience bar and the screen bottom.
const EXPERIENCE_BAR_BOTTOM_MARGIN: f32 = 8.0;

fn experience_bar_anchor() -> Anchor<ClientState> {
    Anchor::pinned(
        AnchorPoint::BottomCenter,
        ScreenPosition {
            left: -EXPERIENCE_BAR_WIDTH / 2.0,
            top: -(EXPERIENCE_BAR_HEIGHT + EXPERIENCE_BAR_BOTTOM_MARGIN),
        },
    )
}

fn experience_bar_size() -> ScreenSize {
    ScreenSize {
        width: EXPERIENCE_BAR_WIDTH,
        height: EXPERIENCE_BAR_HEIGHT,
    }
}

/// Width of the bottom-left chat window in pixels.
const CHAT_WIDTH: f32 = 450.0;
/// Height of the bottom-left chat window in pixels.
const CHAT_HEIGHT: f32 = 220.0;
/// Width of the chat toggle button.
const CHAT_TOGGLE_WIDTH: f32 = 60.0;
/// Height of the chat toggle button.
const CHAT_TOGGLE_HEIGHT: f32 = 22.0;
/// Margin between the chat window and the screen left edge.
const CHAT_EDGE_MARGIN: f32 = 8.0;
/// Bottom margin = leave room for the experience bar + a small gap.
const CHAT_BOTTOM_MARGIN: f32 =
    EXPERIENCE_BAR_HEIGHT + EXPERIENCE_BAR_BOTTOM_MARGIN + 16.0;

fn chat_anchor() -> Anchor<ClientState> {
    Anchor::pinned(
        AnchorPoint::BottomLeft,
        ScreenPosition {
            left: CHAT_EDGE_MARGIN,
            top: -(CHAT_HEIGHT + CHAT_BOTTOM_MARGIN + CHAT_TOGGLE_HEIGHT + 4.0),
        },
    )
}

fn chat_size() -> ScreenSize {
    ScreenSize {
        width: CHAT_WIDTH,
        height: CHAT_HEIGHT,
    }
}

fn chat_toggle_anchor() -> Anchor<ClientState> {
    Anchor::pinned(
        AnchorPoint::BottomLeft,
        ScreenPosition {
            left: CHAT_EDGE_MARGIN,
            top: -(CHAT_TOGGLE_HEIGHT + CHAT_BOTTOM_MARGIN),
        },
    )
}

fn chat_toggle_size() -> ScreenSize {
    ScreenSize {
        width: CHAT_TOGGLE_WIDTH,
        height: CHAT_TOGGLE_HEIGHT,
    }
}

/// Width of the top-left character overview window in pixels.
const CHARACTER_OVERVIEW_WIDTH: f32 = 320.0;
/// Height of the top-left character overview window in pixels.
const CHARACTER_OVERVIEW_HEIGHT: f32 = 200.0;
/// Margin between the character overview window and the screen edges.
const CHARACTER_OVERVIEW_EDGE_MARGIN: f32 = 8.0;

fn character_overview_anchor() -> Anchor<ClientState> {
    Anchor::pinned(
        AnchorPoint::TopLeft,
        ScreenPosition {
            left: CHARACTER_OVERVIEW_EDGE_MARGIN,
            top: CHARACTER_OVERVIEW_EDGE_MARGIN,
        },
    )
}

fn character_overview_size() -> ScreenSize {
    ScreenSize {
        width: CHARACTER_OVERVIEW_WIDTH,
        height: CHARACTER_OVERVIEW_HEIGHT,
    }
}

/// Size (width = height) of the top-right minimap in pixels. Matches the
/// `MINIMAP_DISPLAY_SIZE` constant in `minimap.rs`.
const MINIMAP_SIZE: f32 = 200.0;
/// Margin between the minimap and the screen edges.
const MINIMAP_EDGE_MARGIN: f32 = 8.0;

fn minimap_anchor() -> Anchor<ClientState> {
    Anchor::pinned(
        AnchorPoint::TopRight,
        ScreenPosition {
            left: -(MINIMAP_SIZE + MINIMAP_EDGE_MARGIN),
            top: MINIMAP_EDGE_MARGIN,
        },
    )
}

fn minimap_size() -> ScreenSize {
    ScreenSize {
        width: MINIMAP_SIZE,
        height: MINIMAP_SIZE,
    }
}

#[derive(Serialize, Deserialize)]
pub struct WindowState {
    pub anchor: Anchor<ClientState>,
    pub size: ScreenSize,
}

impl WindowState {
    pub fn new(anchor: Anchor<ClientState>, size: ScreenSize) -> Self {
        Self { anchor, size }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct WindowCache {
    entries: HashMap<WindowClass, WindowState>,
}

impl WindowCache {
    // Since `WindowClass` has some variants with debug features enabled, we use a
    // differen file to store the window cache. This avoids failing to load and
    // thereby wiping the previous window cache when switching between debug and
    // non-debug builds.
    #[cfg(not(feature = "debug"))]
    const FILE_NAME: &'static str = "client/window_cache.ron";
    #[cfg(feature = "debug")]
    const FILE_NAME: &'static str = "client/window_cache_debug.ron";

    fn load() -> Option<Self> {
        #[cfg(feature = "debug")]
        print_debug!("loading window cache from {}", Self::FILE_NAME.magenta());

        std::fs::read_to_string(Self::FILE_NAME)
            .ok()
            .and_then(|data| ron::from_str(&data).ok())
            .map(|entries| Self { entries })
    }

    fn save(&self) {
        #[cfg(feature = "debug")]
        print_debug!("saving window cache to {}", Self::FILE_NAME.magenta());

        let data = ron::ser::to_string_pretty(&self.entries, PrettyConfig::new()).unwrap();
        std::fs::write(Self::FILE_NAME, data).expect("unable to write file");
    }
}

impl korangar_interface::application::WindowCache<ClientState> for WindowCache {
    fn create() -> Self {
        Self::load().unwrap_or_else(|| {
            #[cfg(feature = "debug")]
            print_debug!(
                "failed to load window cache from {}. creating empty cache",
                Self::FILE_NAME.magenta()
            );

            Default::default()
        })
    }

    fn get_window_state(&self, class: WindowClass) -> Option<(Anchor<ClientState>, ScreenSize)> {
        if matches!(class, WindowClass::ExperienceBar) {
            return Some((experience_bar_anchor(), experience_bar_size()));
        }
        if matches!(class, WindowClass::Chat) {
            return Some((chat_anchor(), chat_size()));
        }
        if matches!(class, WindowClass::ChatToggle) {
            return Some((chat_toggle_anchor(), chat_toggle_size()));
        }
        if matches!(class, WindowClass::CharacterOverview) {
            return Some((character_overview_anchor(), character_overview_size()));
        }
        if matches!(class, WindowClass::Minimap) {
            return Some((minimap_anchor(), minimap_size()));
        }
        self.entries.get(&class).map(|entry| (entry.anchor, entry.size))
    }

    fn register_window(&mut self, class: WindowClass, anchor: Anchor<ClientState>, size: ScreenSize) {
        if matches!(
            class,
            WindowClass::ExperienceBar
                | WindowClass::Chat
                | WindowClass::ChatToggle
                | WindowClass::CharacterOverview
                | WindowClass::Minimap
        ) {
            return;
        }
        if let Some(entry) = self.entries.get_mut(&class) {
            entry.anchor = anchor;
            entry.size = size;
        } else {
            let entry = WindowState::new(anchor, size);
            self.entries.insert(class, entry);
        }
    }

    fn update_anchor(&mut self, class: WindowClass, anchor: Anchor<ClientState>) {
        if matches!(
            class,
            WindowClass::ExperienceBar
                | WindowClass::Chat
                | WindowClass::ChatToggle
                | WindowClass::CharacterOverview
                | WindowClass::Minimap
        ) {
            return;
        }
        if let Some(entry) = self.entries.get_mut(&class) {
            entry.anchor = anchor;
        }
    }

    fn update_size(&mut self, class: WindowClass, size: ScreenSize) {
        if matches!(
            class,
            WindowClass::ExperienceBar
                | WindowClass::Chat
                | WindowClass::ChatToggle
                | WindowClass::CharacterOverview
                | WindowClass::Minimap
        ) {
            return;
        }
        if let Some(entry) = self.entries.get_mut(&class) {
            entry.size = size;
        }
    }
}

impl Drop for WindowCache {
    fn drop(&mut self) {
        self.save();
    }
}
