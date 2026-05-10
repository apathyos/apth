use serde::Serialize;
use wayland_client::{
    QueueHandle,
    globals::GlobalList,
    protocol::{wl_keyboard, wl_registry, wl_seat},
};
use xkbcommon_rs as xkb;

use crate::{App, types::protocol::GlobalInterface};

#[derive(Default, Clone, Serialize)]
pub struct Inputs {
    pub keyboard: Option<Keyboard>,
    #[serde(skip)]
    pub wl_keyboard: Option<wl_keyboard::WlKeyboard>,
}

impl Inputs {
    pub fn bind_input_manager(
        state: &mut App,
        registry: wl_registry::WlRegistry,
        globals: &GlobalList,
        qh: &QueueHandle<App>,
    ) {
        globals.contents().with_list(|globals| {
            for global in globals {
                if global.interface == GlobalInterface::WL_SEAT {
                    let seat = registry.bind::<wl_seat::WlSeat, _, _>(
                        global.name,
                        global.version.min(7),
                        qh,
                        (),
                    );

                    state.seats.wl_seats.insert(global.name, seat);
                }
            }
        });
    }
}

#[derive(Clone, Serialize)]
pub struct Keyboard {
    #[serde(skip)]
    pub ctx: xkb::Context,
    #[serde(skip)]
    pub keymap: Option<xkb::Keymap>,
    #[serde(skip)]
    pub keymap_text: Option<String>,
    #[serde(skip)]
    pub state: Option<xkb::State>,
    pub layout: Option<KeyboardLayout>,
    pub layout_short: Option<KeyboardLayout>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            ctx: xkb::Context::new(0).unwrap(),
            keymap: None,
            keymap_text: None,
            state: None,
            layout: None,
            layout_short: None,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct KeyboardLayout(pub String);
