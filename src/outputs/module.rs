use std::collections::HashMap;

use serde::Serialize;
use wayland_client::{
    Proxy, QueueHandle,
    globals::GlobalList,
    protocol::{wl_output, wl_registry::WlRegistry},
};

use wayland_protocols_wlr::output_management::v1::client::zwlr_output_manager_v1;

use crate::{
    App,
    types::{protocol::GlobalInterface, structures::HashMapVector},
};

pub type OutputsMap = HashMapVector<u32, Output>;

#[derive(Default, Serialize)]
pub struct Outputs {
    #[serde(skip)]
    pub wl_outputs_by_global_name: HashMap<u32, wl_output::WlOutput>,
    #[serde(skip)]
    pub wl_outputs_by_name: HashMap<String, u32>,
    #[serde(skip)]
    pub head_for_wl_output: HashMap<u32, u32>,
    #[serde(skip)]
    pub outputs_manager: Option<zwlr_output_manager_v1::ZwlrOutputManagerV1>,
    pub outputs: OutputsMap,
    #[serde(skip)]
    pub modes: HashMap<u32, OutputModes>,
    #[serde(skip)]
    pub output_for_mode: HashMap<u32, u32>,
}

impl Outputs {
    pub fn bind_output(
        state: &mut App,
        registry: &WlRegistry,
        global: (u32, String, u32),
        qh: &QueueHandle<App>,
    ) {
        let (name, interface, version) = global;

        if interface == GlobalInterface::WL_OUTPUT
            && !state.output.wl_outputs_by_global_name.contains_key(&name)
        {
            let v = version.min(4);
            let output: wl_output::WlOutput = registry.bind(name, v, qh, ());
            state.output.wl_outputs_by_global_name.insert(name, output);
        }
    }

    pub fn unbind_output(state: &mut App, global_name: u32) {
        if let Some(wl_output) = state.output.wl_outputs_by_global_name.get(&global_name) {
            let id = wl_output.id().protocol_id();

            if let Some(head) = state
                .output
                .head_for_wl_output
                .get(&id)
                .and_then(|id| state.output.outputs.get(id))
            {
                state
                    .output
                    .wl_outputs_by_name
                    .remove(head.name.as_ref().unwrap_or(&String::new()));
            };

            state.output.head_for_wl_output.remove(&id);
        };

        state.output.wl_outputs_by_global_name.remove(&global_name);
    }

    pub fn bind_outputs_manager(state: &mut App, globals: &GlobalList, qh: &QueueHandle<App>) {
        let registry: WlRegistry = globals.registry().clone();

        globals.contents().with_list(|globals| {
            for global in globals {
                Self::bind_output(
                    state,
                    &registry,
                    (global.name, global.interface.clone(), global.version),
                    qh,
                );
            }
        });

        let outputs_manager =
            globals.bind::<zwlr_output_manager_v1::ZwlrOutputManagerV1, _, _>(qh, 1..=1, ());

        if let Ok(outputs_manager) = outputs_manager {
            state.output.outputs_manager = Some(outputs_manager);
        }
    }
}

#[derive(Default, Serialize)]
pub struct Output {
    #[serde(skip)]
    is_published: bool,
    #[serde(skip)]
    pub wl_output_id: Option<u32>,
    pub name: Option<String>,
    pub model: Option<String>,
    pub make: Option<String>,
    pub description: Option<String>,
    pub mode: Option<OutputMode>,
    pub scale: Option<f64>,
    pub power: Option<OutputPower>,
    pub windows: HashMapVector<u32, String>,
}

impl Output {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_is_ready(&self) -> bool {
        self.name.is_some()
            && self.description.is_some()
            && self.mode.is_some()
            && self.scale.is_some()
            && self.power.is_some()
    }

    pub fn get_is_published(&self) -> bool {
        self.is_published
    }

    pub fn mark_published(&mut self) {
        self.is_published = true;
    }
}

#[derive(Default, Clone, Serialize)]
pub struct OutputMode {
    pub width: i32,
    pub height: i32,
    pub refresh: i32,
    pub state: OutputModeState,
}

impl OutputMode {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Default, Clone, Serialize)]
pub struct OutputPower {
    pub mode: OutputPowerMode,
}

impl OutputPower {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Default, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputModeState {
    #[default]
    Current,
    Preffered,
}

#[derive(Default, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputPowerMode {
    #[default]
    On,
    Off,
}

#[derive(Default, Clone)]
pub struct OutputModes {
    pub available_modes: HashMap<u32, OutputMode>,
    pub current_mode: u32,
}

impl OutputModes {
    pub fn new() -> Self {
        Default::default()
    }
}
