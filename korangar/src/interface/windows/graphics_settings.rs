use korangar_interface::window::{CustomWindow, Window};
use rust_state::Path;

use crate::interface::windows::WindowClass;
use crate::loaders::OverflowBehavior;
use crate::settings::{GraphicsSettingsCapabilitiesPathExt, GraphicsSettingsPathExt};
use crate::state::localization::LocalizationPathExt;
use crate::state::theme::InterfaceThemeType;
use crate::state::{ClientState, ClientStatePathExt, client_state};
use crate::{GraphicsSettings, GraphicsSettingsCapabilities};

pub struct GraphicsSettingsWindow<A, B> {
    settings_path: A,
    capabilities_path: B,
}

impl<A, B> GraphicsSettingsWindow<A, B> {
    pub fn new(settings_path: A, capabilities_path: B) -> Self {
        Self {
            settings_path,
            capabilities_path,
        }
    }
}

impl<A, B> CustomWindow<ClientState> for GraphicsSettingsWindow<A, B>
where
    A: Path<ClientState, GraphicsSettings>,
    B: Path<ClientState, GraphicsSettingsCapabilities>,
{
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::GraphicsSettings)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        let elements = (
            split! {
                children: (
                    text! {
                        text: client_state().localization().lighting_mode_text(),
                        overflow_behavior: OverflowBehavior::Shrink,
                    },
                    drop_down! {
                        selected: self.settings_path.lighting_mode(),
                        options: self.capabilities_path.lighting_modes(),
                    }
                )
            },
            state_button! {
                text: client_state().localization().triple_buffering_text(),
                state: self.settings_path.triple_buffering(),
                event: Toggle(self.settings_path.triple_buffering()),
            },
            state_button! {
                text: client_state().localization().vsync_text(),
                state: self.settings_path.vsync(),
                event: Toggle(self.settings_path.vsync()),
                disabled: self.capabilities_path.vsync_setting_disabled(),
                disabled_tooltip: client_state().localization().vsync_not_supported_tooltip(),
            },
            split! {
                children: (
                    text! {
                        text: client_state().localization().limit_framerate_text(),
                        overflow_behavior: OverflowBehavior::Shrink,
                    },
                    drop_down! {
                        selected: self.settings_path.limit_framerate(),
                        options: self.capabilities_path.limit_framerate_options(),
                    }
                )
            },
            split! {
                children: (
                    text! {
                        text: client_state().localization().texture_filtering_text(),
                        overflow_behavior: OverflowBehavior::Shrink,
                    },
                    drop_down! {
                        selected: self.settings_path.texture_filtering(),
                        options: self.capabilities_path.texture_filtering_options(),
                    }
                )
            },
            split! {
                children: (
                    text! {
                        text: client_state().localization().multisampling_text(),
                        overflow_behavior: OverflowBehavior::Shrink,
                    },
                    drop_down! {
                        selected: self.settings_path.msaa(),
                        options: self.capabilities_path.supported_msaa(),
                    }
                )
            },
            split! {
                children: (
                    text! {
                        text: client_state().localization().supersampling_text(),
                        overflow_behavior: OverflowBehavior::Shrink,
                    },
                    drop_down! {
                        selected: self.settings_path.ssaa(),
                        options: self.capabilities_path.ssaa_options(),
                    }
                )
            },
            split! {
                children: (
                    text! {
                        text: client_state().localization().screen_space_aa_text(),
                        overflow_behavior: OverflowBehavior::Shrink,
                    },
                    drop_down! {
                        selected: self.settings_path.screen_space_anti_aliasing(),
                        options: self.capabilities_path.screen_space_anti_aliasing_options(),
                    }
                )
            },
            split! {
                children: (
                    text! {
                        text: client_state().localization().shadow_method_text(),
                        overflow_behavior: OverflowBehavior::Shrink,
                    },
                    drop_down! {
                        selected: self.settings_path.shadow_method(),
                        options: self.capabilities_path.shadow_method_options(),
                    }
                )
            },
            split! {
                children: (
                    text! {
                        text: client_state().localization().shadow_detail_text(),
                        overflow_behavior: OverflowBehavior::Shrink,
                    },
                    drop_down! {
                        selected: self.settings_path.shadow_detail(),
                        options: self.capabilities_path.shadow_detail_options(),
                    }
                )
            },
            split! {
                children: (
                    text! {
                        text: client_state().localization().shadow_resolution_text(),
                        overflow_behavior: OverflowBehavior::Shrink,
                    },
                    drop_down! {
                        selected: self.settings_path.shadow_resolution(),
                        options: self.capabilities_path.shadow_resolution_options(),
                    }
                )
            },
            state_button! {
                text: client_state().localization().sdsm_text(),
                state: self.settings_path.sdsm(),
                event: Toggle(self.settings_path.sdsm()),
            },
            state_button! {
                text: client_state().localization().high_quality_interface_text(),
                state: self.settings_path.high_quality_interface(),
                event: Toggle(self.settings_path.high_quality_interface()),
            },
        );

        window! {
            title: client_state().localization().graphics_settings_window_title(),
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            closable: true,
            elements,
        }
    }
}
