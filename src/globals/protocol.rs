use wayland_client::{
    Connection, Dispatch, QueueHandle, globals::GlobalListContents, protocol::wl_registry,
};

use crate::{App, outputs::Outputs};

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for App {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => {
                Outputs::bind_output(state, registry, (name, interface, version), qh);
            }
            wl_registry::Event::GlobalRemove { name } => {
                Outputs::unbind_output(state, name);
            }
            _ => {}
        }
    }
}
