use korangar_interface::components::text_box::DefaultHandler;
use korangar_interface::element::StateElement;
use korangar_interface::window::{CustomWindow, Window};
use ragnarok_packets::InventoryIndex;
use rust_state::{Path, RustState, State};

use crate::graphics::Color;
use crate::input::InputEvent;
use crate::interface::windows::WindowClass;
use crate::loaders::OverflowBehavior;
use crate::state::ClientState;
use crate::state::theme::InterfaceThemeType;

const MAXIMUM_AMOUNT_TEXT_LENGTH: usize = 5;

#[derive(Default, RustState, StateElement)]
pub struct DropItemPromptState {
    inventory_index: u16,
    max_amount: u16,
    amount_text: String,
}

impl DropItemPromptState {
    pub fn set(&mut self, index: InventoryIndex, max_amount: u16) {
        self.inventory_index = index.0;
        self.max_amount = max_amount;
        self.amount_text = "1".to_string();
    }
}

#[derive(Clone, Copy)]
pub struct DropItemPromptFocus;

pub struct DropItemPromptWindow<S> {
    state_path: S,
}

impl<S> DropItemPromptWindow<S> {
    pub fn new(state_path: S) -> Self {
        Self { state_path }
    }
}

impl<S> CustomWindow<ClientState> for DropItemPromptWindow<S>
where
    S: Path<ClientState, DropItemPromptState> + Copy,
{
    fn window_class() -> Option<WindowClass> {
        Some(WindowClass::DropItemPrompt)
    }

    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        use korangar_interface::prelude::*;

        let amount_text_path = self.state_path.amount_text();
        let confirm_action = move |_: &State<ClientState>, queue: &mut EventQueue<ClientState>| {
            queue.queue(InputEvent::ConfirmDropPrompt);
        };

        window! {
            title: "丟棄道具",
            class: Self::window_class(),
            theme: InterfaceThemeType::InGame,
            closable: true,
            elements: (
                text! {
                    text: "請輸入要丟棄的數量",
                    color: Color::rgb_u8(220, 220, 220),
                },
                text_box! {
                    ghost_text: "數量",
                    state: amount_text_path,
                    input_handler: DefaultHandler::<_, _, MAXIMUM_AMOUNT_TEXT_LENGTH>::new(amount_text_path, confirm_action),
                    focus_id: DropItemPromptFocus,
                    overflow_behavior: OverflowBehavior::Shrink,
                },
                split! {
                    gaps: theme().window().gaps(),
                    children: (
                        button! {
                            text: "取消",
                            event: InputEvent::CancelDropPrompt,
                        },
                        button! {
                            text: "確認丟棄",
                            event: InputEvent::ConfirmDropPrompt,
                        },
                    ),
                },
            ),
        }
    }
}
