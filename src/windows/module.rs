use serde::Serialize;
use wayland_client::{QueueHandle, globals::GlobalList};
use wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1;

use crate::{App, types::structures::HashMapVector};

pub type WindowsMap = HashMapVector<u32, Window>;

#[derive(Default, Serialize)]
pub struct Windows {
    #[serde(skip)]
    manager: Option<WindowsManager>,
    pub windows: WindowsMap,
}

impl Windows {
    pub fn bind_toplevel_manager(state: &mut App, globals: &GlobalList, qh: &QueueHandle<App>) {
        let manager = globals
            .bind::<zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, _, _>(
                qh,
                1..=3,
                (),
            )
            .expect("zwlr_foreign_toplevel_manager_v1 is not advertised by the compositor");

        state.windows.manager = Some(WindowsManager::new(manager));
    }
}

#[derive(Serialize)]
pub struct WindowsManager {
    #[serde(skip)]
    manager: zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1,
}

impl WindowsManager {
    pub fn new(manager: zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1) -> Self {
        Self { manager }
    }
}

#[derive(Default, Serialize)]
pub struct Window {
    pub id: Option<String>,
    pub app_id: Option<String>,
    pub title: Option<String>,
    pub is_activated: Option<bool>,
    pub is_fullscreen: Option<bool>,
    pub is_maximized: Option<bool>,
    pub is_minimized: Option<bool>,
    pub parent_id: Option<u32>,
    pub outputs: HashMapVector<u32, String>,
    pub workspaces: HashMapVector<u32, String>,
}

impl Window {
    pub fn new() -> Self {
        Default::default()
    }
}
