use serde::Serialize;
use wayland_client::{QueueHandle, globals::GlobalList};
use wayland_protocols::ext::workspace::v1::client::ext_workspace_manager_v1::ExtWorkspaceManagerV1;

use crate::{
    App,
    types::structures::{HashMapVector, HashSetVector},
};

pub type WorkspacesMap = HashMapVector<u32, Workspace>;

#[derive(Default, Serialize)]
pub struct Workspaces {
    #[serde(skip)]
    pub manager: Option<WorkspaceManager>,
    #[serde(skip)]
    pub workspace_groups: HashMapVector<u32, WorkspaceGroup>,
    pub workspaces: WorkspacesMap,
}

impl Workspaces {
    pub fn bind_workspace_manager(state: &mut App, globals: &GlobalList, qh: &QueueHandle<App>) {
        let manager = globals
            .bind::<ExtWorkspaceManagerV1, _, _>(qh, 1..=1, ())
            .expect("ext_workspace_manager_v1 is not advertised by the compositor");

        state.workspaces.manager = Some(WorkspaceManager::new(manager));
    }
}

#[derive(Serialize)]
pub struct WorkspaceManager {
    #[serde(skip)]
    manager: ExtWorkspaceManagerV1,
}

impl WorkspaceManager {
    pub fn new(manager: ExtWorkspaceManagerV1) -> Self {
        Self { manager }
    }
}

#[derive(Default, Clone, Serialize)]
pub struct WorkspaceGroup {
    pub id: Option<String>,
    pub outputs: HashSetVector<u32>,
    pub workspaces: HashSetVector<u32>,
}

impl WorkspaceGroup {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Default, Clone, Serialize)]
pub struct Workspace {
    #[serde(skip)]
    is_published: bool,
    pub id: Option<String>,
    pub name: Option<String>,
    pub is_active: Option<bool>,
    pub is_urgent: Option<bool>,
    pub is_hidden: Option<bool>,
}

impl Workspace {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_is_ready(&self) -> bool {
        self.name.is_some() && self.is_active.is_some()
    }

    pub fn get_is_published(&self) -> bool {
        self.is_published
    }

    pub fn mark_published(&mut self) {
        self.is_published = true;
    }
}
