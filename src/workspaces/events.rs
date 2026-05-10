use crate::{event_stream::StreamEventData, workspaces::{Workspace, WorkspacesMap}};

pub struct WorkspacesChanged<'a> {
    payload: &'a WorkspacesMap,
}

impl<'a> WorkspacesChanged<'a> {
    pub fn from(payload: &'a WorkspacesMap) -> Self {
        Self { payload }
    }
}

impl<'a> StreamEventData for WorkspacesChanged<'a> {
    fn get_event_type(&self) -> &'static str {
        "workspaces.changed"
    }

    fn get_payload(&self) -> impl serde::Serialize {
        self.payload
    }
}

pub struct WorkspaceCreated<'a> {
    payload: &'a Workspace,
}

impl<'a> WorkspaceCreated<'a> {
    pub fn from(payload: &'a Workspace) -> Self {
        Self { payload }
    }
}

impl<'a> StreamEventData for WorkspaceCreated<'a> {
    fn get_event_type(&self) -> &'static str {
        "workspace.created"
    }

    fn get_payload(&self) -> impl serde::Serialize {
        self.payload
    }
}

pub struct WorkspaceChanged<'a> {
    payload: &'a Workspace,
}

impl<'a> WorkspaceChanged<'a> {
    pub fn from(payload: &'a Workspace) -> Self {
        Self { payload }
    }
}

impl<'a> StreamEventData for WorkspaceChanged<'a> {
    fn get_event_type(&self) -> &'static str {
        "workspace.changed"
    }

    fn get_payload(&self) -> impl serde::Serialize {
        self.payload
    }
}

pub struct WorkspaceRemoved<'a> {
    payload: &'a Workspace,
}

impl<'a> WorkspaceRemoved<'a> {
    pub fn from(payload: &'a Workspace) -> Self {
        Self { payload }
    }
}

impl<'a> StreamEventData for WorkspaceRemoved<'a> {
    fn get_event_type(&self) -> &'static str {
        "workspace.removed"
    }

    fn get_payload(&self) -> impl serde::Serialize {
        self.payload
    }
}
