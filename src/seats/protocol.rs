use wayland_client::{Connection, Dispatch, QueueHandle, WEnum, protocol::wl_seat};

use crate::{App, inputs::Keyboard};

impl Dispatch<wl_seat::WlSeat, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &wl_seat::WlSeat,
        event: <wl_seat::WlSeat as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_seat::Event::Capabilities { capabilities } => {
                if let WEnum::Value(caps) = capabilities
                    && caps.contains(wl_seat::Capability::Keyboard)
                    && state.input.keyboard.is_none()
                {
                    let kbd = proxy.get_keyboard(qh, ());
                    state.input.wl_keyboard = Some(kbd);

                    if state.input.keyboard.is_none() {
                        state.input.keyboard = Some(Keyboard::new());
                    }
                }
            }
            _ => {}
        }
    }
}
