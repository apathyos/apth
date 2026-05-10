use super::{Workspace, WorkspaceGroup};
use crate::{
    App, stream_emit,
    workspaces::{WorkspaceChanged, WorkspaceCreated, WorkspaceRemoved, WorkspacesChanged},
};
use wayland_client::{Dispatch, Proxy, QueueHandle, WEnum, event_created_child};
use wayland_protocols::ext::workspace::v1::client::{
    ext_workspace_group_handle_v1::{self, ExtWorkspaceGroupHandleV1},
    ext_workspace_handle_v1::{self, ExtWorkspaceHandleV1},
    ext_workspace_manager_v1::{self, ExtWorkspaceManagerV1},
};

impl Dispatch<ext_workspace_manager_v1::ExtWorkspaceManagerV1, ()> for App {
    fn event(
        state: &mut Self,
        _: &ext_workspace_manager_v1::ExtWorkspaceManagerV1,
        event: ext_workspace_manager_v1::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            ext_workspace_manager_v1::Event::Workspace { workspace } => {
                let id = workspace.id();

                state
                    .workspaces
                    .workspaces
                    .insert(id.protocol_id(), Workspace::new());
            }
            ext_workspace_manager_v1::Event::WorkspaceGroup { workspace_group } => {
                let id = workspace_group.id();

                state
                    .workspaces
                    .workspace_groups
                    .insert(id.protocol_id(), WorkspaceGroup::new());
            }
            _ => {}
        }

        stream_emit!(
            state.stream_context,
            true,
            WorkspacesChanged::from(&state.workspaces.workspaces)
        );
    }

    event_created_child!(App, ExtWorkspaceManagerV1, [
        ext_workspace_manager_v1::EVT_WORKSPACE_GROUP_OPCODE => (ExtWorkspaceGroupHandleV1, ()),
        ext_workspace_manager_v1::EVT_WORKSPACE_OPCODE => (ExtWorkspaceHandleV1, ()),
    ]);
}

impl Dispatch<ext_workspace_group_handle_v1::ExtWorkspaceGroupHandleV1, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &ext_workspace_group_handle_v1::ExtWorkspaceGroupHandleV1,
        event: ext_workspace_group_handle_v1::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &QueueHandle<Self>,
    ) {
        let id = proxy.id();
        let group = state.workspaces.workspace_groups.entry(id.protocol_id());

        match event {
            ext_workspace_group_handle_v1::Event::WorkspaceEnter { workspace } => {
                group.and_modify(|group| {
                    group.workspaces.insert(workspace.id().protocol_id());
                });
            }
            ext_workspace_group_handle_v1::Event::WorkspaceLeave { workspace } => {
                group.and_modify(|group| {
                    group.workspaces.remove(&workspace.id().protocol_id());
                });
            }
            ext_workspace_group_handle_v1::Event::OutputEnter { output } => {
                group.and_modify(|group| {
                    group.outputs.insert(output.id().protocol_id());
                });
            }
            ext_workspace_group_handle_v1::Event::OutputLeave { output } => {
                group.and_modify(|group| {
                    group.outputs.remove(&output.id().protocol_id());
                });
            }
            ext_workspace_group_handle_v1::Event::Removed {} => {
                state.workspaces.workspace_groups.remove(&id.protocol_id());
            }
            _ => {}
        }

        stream_emit!(
            state.stream_context,
            true,
            WorkspacesChanged::from(&state.workspaces.workspaces)
        );
    }
}

impl Dispatch<ext_workspace_handle_v1::ExtWorkspaceHandleV1, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &ext_workspace_handle_v1::ExtWorkspaceHandleV1,
        event: ext_workspace_handle_v1::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &QueueHandle<Self>,
    ) {
        let id = proxy.id();
        let ws_entry = state.workspaces.workspaces.entry(id.protocol_id());
        let event_opcode = event.opcode();

        match event {
            ext_workspace_handle_v1::Event::Id { id } => {
                ws_entry.and_modify(|workspace| workspace.id = Some(id));
            }
            ext_workspace_handle_v1::Event::Name { name } => {
                ws_entry.and_modify(|workspace| workspace.name = Some(name));
            }
            ext_workspace_handle_v1::Event::State { state } => {
                ws_entry.and_modify(|workspace| match state {
                    WEnum::Value(flags) => {
                        workspace.is_active =
                            Some(flags.contains(ext_workspace_handle_v1::State::Active));
                        workspace.is_urgent =
                            Some(flags.contains(ext_workspace_handle_v1::State::Urgent));
                        workspace.is_hidden =
                            Some(flags.contains(ext_workspace_handle_v1::State::Hidden));
                    }
                    WEnum::Unknown(raw) => {
                        eprintln!("Unknown workspace state bits: {raw}");
                    }
                });
            }
            ext_workspace_handle_v1::Event::Removed => {
                if let Some(ws) = state.workspaces.workspaces.get(&id.protocol_id()) {
                    stream_emit!(state.stream_context, true, WorkspaceRemoved::from(ws));
                };

                state.workspaces.workspaces.remove(&id.protocol_id());
            }
            _ => {}
        }

        stream_emit!(
            state.stream_context,
            true,
            WorkspacesChanged::from(&state.workspaces.workspaces)
        );

        let Some(ws) = state.workspaces.workspaces.get(&id.protocol_id()) else {
            return;
        };

        if event_opcode == ext_workspace_handle_v1::Event::Removed.opcode() {
            return;
        }

        if !ws.get_is_published() && ws.get_is_ready() {
            stream_emit!(state.stream_context, true, WorkspaceCreated::from(ws));
            let ws_entry = state.workspaces.workspaces.entry(id.protocol_id());
            ws_entry.and_modify(|ws| ws.mark_published());
        } else if ws.get_is_ready() {
            stream_emit!(state.stream_context, true, WorkspaceChanged::from(ws));
        }
    }
}
