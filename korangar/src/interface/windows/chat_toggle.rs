use korangar_interface::window::{CustomWindow, Window};
use rust_state::State;

use super::WindowClass;
use crate::input::InputEvent;
use crate::state::ClientState;
use crate::state::theme::InterfaceThemeType;

/// Tiny "聊天" button anchored at the bottom-left. Clicking it asks main.rs
/// to open the real chat window. The button stays visible regardless.
pub struct ChatToggleButton;

impl CustomWindow<ClientState> for ChatToggleButton {
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::ChatToggle)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        let open_chat = move |_: &State<ClientState>, queue: &mut EventQueue<ClientState>| {
            queue.queue(InputEvent::OpenChatWindow);
        };

        window! {
            title: "",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            resizable: false,
            border: 1.0,
            gaps: 0.0,
            title_height: 0.0,
            title_gap: 0.0,
            minimum_width: 50.0,
            maximum_width: 80.0,
            minimum_height: 18.0,
            maximum_height: 24.0,
            elements: (
                button! {
                    text: "聊天",
                    event: open_chat,
                },
            ),
        }
    }
}
