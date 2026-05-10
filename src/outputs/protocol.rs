use crate::{
    App,
    outputs::{
        OutputChanged, OutputCreated, OutputRemoved, OutputsChanged,
        module::{Output, OutputMode, OutputModeState, OutputModes, OutputPower, OutputPowerMode},
    },
    stream_emit,
};
use wayland_client::{Dispatch, Proxy, event_created_child, protocol::wl_output};

use wayland_protocols_wlr::output_management::v1::client::{
    zwlr_output_head_v1, zwlr_output_manager_v1, zwlr_output_mode_v1,
};

impl Dispatch<wl_output::WlOutput, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &wl_output::WlOutput,
        event: <wl_output::WlOutput as Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        let id = proxy.id().protocol_id();

        match event {
            wl_output::Event::Name { name } => {
                state.output.wl_outputs_by_name.insert(name, id);
            }
            _ => {}
        }
    }
}

impl Dispatch<zwlr_output_manager_v1::ZwlrOutputManagerV1, ()> for App {
    event_created_child!(App, zwlr_output_manager_v1::ZwlrOutputManagerV1, [
        zwlr_output_manager_v1::EVT_HEAD_OPCODE => (zwlr_output_head_v1::ZwlrOutputHeadV1, ()),
    ]);

    fn event(
        state: &mut Self,
        _: &zwlr_output_manager_v1::ZwlrOutputManagerV1,
        event: <zwlr_output_manager_v1::ZwlrOutputManagerV1 as Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            zwlr_output_manager_v1::Event::Head { head } => {
                state
                    .output
                    .outputs
                    .insert(head.id().protocol_id(), Output::new());
            }
            zwlr_output_manager_v1::Event::Finished => {
                state.output.outputs_manager = None;
            }
            _ => {}
        }

        stream_emit!(
            state.stream_context,
            true,
            OutputsChanged::from(&state.output.outputs)
        );
    }
}

impl Dispatch<zwlr_output_head_v1::ZwlrOutputHeadV1, ()> for App {
    event_created_child!(App, zwlr_output_head_v1::ZwlrOutputHeadV1, [
        zwlr_output_head_v1::EVT_MODE_OPCODE => (zwlr_output_mode_v1::ZwlrOutputModeV1, ()),
    ]);

    fn event(
        state: &mut Self,
        proxy: &zwlr_output_head_v1::ZwlrOutputHeadV1,
        event: <zwlr_output_head_v1::ZwlrOutputHeadV1 as Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        let event_opcode = event.opcode();
        let output_id = proxy.id().protocol_id();
        let output = state.output.outputs.entry(proxy.id().protocol_id());

        match event {
            zwlr_output_head_v1::Event::Enabled { enabled } => {
                output.and_modify(|output| {
                    let power = output.power.get_or_insert(OutputPower::new());

                    power.mode = if enabled != 0 {
                        OutputPowerMode::On
                    } else {
                        OutputPowerMode::Off
                    }
                });
            }
            zwlr_output_head_v1::Event::Name { name } => {
                output.and_modify(|output| {
                    if let Some(wl_output_id) = state.output.wl_outputs_by_name.get(&name) {
                        output.wl_output_id = Some(*wl_output_id);
                        state
                            .output
                            .head_for_wl_output
                            .insert(*wl_output_id, output_id);
                    };

                    output.name = Some(name);
                });
            }
            zwlr_output_head_v1::Event::Model { model } => {
                output.and_modify(|output| output.model = Some(model));
            }
            zwlr_output_head_v1::Event::Make { make } => {
                output.and_modify(|output| output.make = Some(make));
            }
            zwlr_output_head_v1::Event::Description { description } => {
                output.and_modify(|output| output.description = Some(description));
            }
            zwlr_output_head_v1::Event::Mode { mode } => {
                let mode_id = mode.id().protocol_id();

                state.output.output_for_mode.insert(mode_id, output_id);

                state
                    .output
                    .modes
                    .entry(output_id)
                    .or_insert(OutputModes::new())
                    .available_modes
                    .entry(mode_id)
                    .or_insert(OutputMode::new());
            }
            zwlr_output_head_v1::Event::CurrentMode { mode } => {
                let mode_id = mode.id().protocol_id();

                state.output.modes.entry(output_id).and_modify(|modes| {
                    modes.current_mode = mode_id;
                    modes
                        .available_modes
                        .entry(mode_id)
                        .and_modify(|mode| mode.state = OutputModeState::Current);
                });

                if let Some(output_mode) = state
                    .output
                    .modes
                    .get(&output_id)
                    .and_then(|modes| modes.available_modes.get(&mode_id))
                {
                    output.and_modify(|output| output.mode = Some(output_mode.clone()));
                };
            }
            zwlr_output_head_v1::Event::Scale { scale } => {
                output.and_modify(|output| output.scale = Some(scale));
            }
            zwlr_output_head_v1::Event::Finished => {
                if let Some(output) = state.output.outputs.get(&output_id) {
                    stream_emit!(state.stream_context, true, OutputRemoved::from(output));
                };

                state.output.outputs.remove(&output_id);
                state.output.modes.remove(&output_id);
                state
                    .output
                    .output_for_mode
                    .retain(|_, id| *id != output_id);
            }
            _ => {}
        }

        stream_emit!(
            state.stream_context,
            true,
            OutputsChanged::from(&state.output.outputs)
        );

        let Some(output) = state.output.outputs.get(&output_id) else {
            return;
        };

        if event_opcode == zwlr_output_head_v1::Event::Finished.opcode() {
            return;
        }

        if !output.get_is_published() && output.get_is_ready() {
            stream_emit!(state.stream_context, true, OutputCreated::from(output));
            let output_entry = state.output.outputs.entry(output_id);
            output_entry.and_modify(|output| output.mark_published());
        } else if output.get_is_ready() {
            stream_emit!(state.stream_context, true, OutputChanged::from(output));
        }
    }
}

impl Dispatch<zwlr_output_mode_v1::ZwlrOutputModeV1, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &zwlr_output_mode_v1::ZwlrOutputModeV1,
        event: <zwlr_output_mode_v1::ZwlrOutputModeV1 as Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        let mode_id = proxy.id().protocol_id();
        let output_id = state.output.output_for_mode.get(&mode_id);

        let Some(output_id) = output_id else { return };

        match event {
            zwlr_output_mode_v1::Event::Preferred => {
                state.output.modes.entry(*output_id).and_modify(|modes| {
                    modes
                        .available_modes
                        .entry(mode_id)
                        .and_modify(|mode| mode.state = OutputModeState::Preffered);
                });
            }
            zwlr_output_mode_v1::Event::Refresh { refresh } => {
                state.output.modes.entry(*output_id).and_modify(|modes| {
                    modes
                        .available_modes
                        .entry(mode_id)
                        .and_modify(|mode| mode.refresh = refresh);
                });
            }
            zwlr_output_mode_v1::Event::Size { width, height } => {
                state.output.modes.entry(*output_id).and_modify(|modes| {
                    modes.available_modes.entry(mode_id).and_modify(|mode| {
                        mode.width = width;
                        mode.height = height;
                    });
                });
            }
            zwlr_output_mode_v1::Event::Finished => {
                state.output.modes.entry(*output_id).and_modify(|modes| {
                    modes.available_modes.remove(&mode_id);
                });
                state.output.output_for_mode.remove(&mode_id);
            }
            _ => {}
        }

        stream_emit!(
            state.stream_context,
            true,
            OutputsChanged::from(&state.output.outputs)
        );
    }
}
