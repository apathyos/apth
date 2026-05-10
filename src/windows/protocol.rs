use wayland_client::{Dispatch, Proxy, event_created_child};
use wayland_protocols::ext::foreign_toplevel_list::v1::client::{
    ext_foreign_toplevel_handle_v1, ext_foreign_toplevel_list_v1,
};
use wayland_protocols_wlr::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_handle_v1, zwlr_foreign_toplevel_manager_v1,
};

use crate::{
    App, stream_emit,
    windows::{events::WindowsChanged, module::Window},
};

impl Dispatch<ext_foreign_toplevel_list_v1::ExtForeignToplevelListV1, ()> for App {
    fn event(
        _: &mut Self,
        _: &ext_foreign_toplevel_list_v1::ExtForeignToplevelListV1,
        _: <ext_foreign_toplevel_list_v1::ExtForeignToplevelListV1 as Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1,
        event: <ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1 as Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        let id = proxy.id().protocol_id();
        let toplevel = state.windows.windows.entry(id);

        match event {
            ext_foreign_toplevel_handle_v1::Event::AppId { app_id } => {
                toplevel.and_modify(|toplevel| toplevel.id = Some(app_id));
            }
            _ => {}
        }
    }
}

impl Dispatch<zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1,
        event: <zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1 as wayland_client::Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            zwlr_foreign_toplevel_manager_v1::Event::Toplevel { toplevel } => {
                let id = toplevel.id().protocol_id();
                state.windows.windows.insert(id, Window::new());
            }
            zwlr_foreign_toplevel_manager_v1::Event::Finished => {
                let id = proxy.id().protocol_id();
                state.windows.windows.remove(&id);
            }
            _ => {}
        }

        stream_emit!(
            state.stream_context,
            true,
            WindowsChanged::from(&state.windows.windows)
        )
    }

    event_created_child!(App, zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, [
        zwlr_foreign_toplevel_manager_v1::EVT_TOPLEVEL_OPCODE => (zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1, ())
    ]);
}

impl Dispatch<zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1,
        event: <zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1 as wayland_client::Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        let id = proxy.id().protocol_id();
        let toplevel = state.windows.windows.entry(id);

        match event {
            zwlr_foreign_toplevel_handle_v1::Event::AppId { app_id } => {
                toplevel.and_modify(|toplevel| toplevel.app_id = Some(app_id));
            }
            zwlr_foreign_toplevel_handle_v1::Event::Title { title } => {
                toplevel.and_modify(|toplevel| toplevel.title = Some(title));
            }
            zwlr_foreign_toplevel_handle_v1::Event::State { state } => {
                toplevel.and_modify(|toplevel| {
                    toplevel.is_activated = Some(
                        state.contains(&(zwlr_foreign_toplevel_handle_v1::State::Activated as u8)),
                    );
                    toplevel.is_fullscreen = Some(
                        state.contains(&(zwlr_foreign_toplevel_handle_v1::State::Fullscreen as u8)),
                    );
                    toplevel.is_maximized = Some(
                        state.contains(&(zwlr_foreign_toplevel_handle_v1::State::Maximized as u8)),
                    );
                    toplevel.is_minimized = Some(
                        state.contains(&(zwlr_foreign_toplevel_handle_v1::State::Minimized as u8)),
                    );
                });
            }
            zwlr_foreign_toplevel_handle_v1::Event::Parent { parent } => {
                if let Some(parent) = parent {
                    toplevel.and_modify(|toplevel| {
                        toplevel.parent_id = Some(parent.id().protocol_id())
                    });
                }
            }
            zwlr_foreign_toplevel_handle_v1::Event::OutputEnter { output } => {
                let output_id = output.id().protocol_id();
                let Some(output_entry) = state
                    .output
                    .head_for_wl_output
                    .get(&output_id)
                    .and_then(|id| state.output.outputs.get(id))
                else {
                    return eprintln!("Can't find output for the window");
                };

                toplevel.and_modify(|toplevel| {
                    toplevel.outputs.insert(
                        output_id,
                        output_entry.name.clone().expect("Output has no name!"),
                    );
                });
            }
            zwlr_foreign_toplevel_handle_v1::Event::OutputLeave { output } => {
                let output_id = output.id().protocol_id();

                toplevel.and_modify(|toplevel| {
                    toplevel.outputs.remove(&output_id);
                });
            }
            zwlr_foreign_toplevel_handle_v1::Event::Closed => {
                state.windows.windows.remove(&id);
                stream_emit!(
                    state.stream_context,
                    true,
                    WindowsChanged::from(&state.windows.windows)
                )
            }
            zwlr_foreign_toplevel_handle_v1::Event::Done => {
                stream_emit!(
                    state.stream_context,
                    true,
                    WindowsChanged::from(&state.windows.windows)
                )
            }
            _ => {}
        }
    }
}
