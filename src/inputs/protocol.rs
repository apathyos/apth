use wayland_client::{Dispatch, WEnum, protocol::wl_keyboard};
use xkbcommon_rs as xkb;

use crate::{
    App,
    inputs::{
        KeyboardLayoutChanged, module::KeyboardLayout, utils::{parse_group_layout_ids, read_keymap_from_fd}
    },
    stream_emit,
};

impl Dispatch<wl_keyboard::WlKeyboard, ()> for App {
    fn event(
        state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: <wl_keyboard::WlKeyboard as wayland_client::Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        let kbd = state.input.keyboard.as_mut().unwrap();

        match event {
            wl_keyboard::Event::Keymap { format, fd, size } => {
                let is_xkb_v1 = matches!(format, WEnum::Value(wl_keyboard::KeymapFormat::XkbV1));

                if !is_xkb_v1 {
                    eprintln!("Unsupported keymap format: {:?}", format);
                    return;
                }

                let keymap = &match read_keymap_from_fd(fd, size) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Failed to read keymap: {e:#}");
                        return;
                    }
                };

                let ctx = kbd.ctx.clone();

                let km =
                    match xkb::Keymap::new_from_string(ctx, keymap, xkb::KeymapFormat::TextV1, 0) {
                        Ok(km) => km,
                        Err(e) => {
                            eprintln!("Keymap::new_from_string failed: {e:?}");
                            return;
                        }
                    };

                let st = xkb::State::new(km.clone());

                kbd.keymap = Some(km.clone());
                kbd.keymap_text = Some(keymap.clone());
                kbd.state = Some(st);
            }
            wl_keyboard::Event::Modifiers {
                serial: _,
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
            } => {
                if let Some(kbd_state) = kbd.state.as_mut() {
                    let mut layout_has_changed = false;

                    kbd_state.update_mask(
                        mods_depressed,
                        mods_latched,
                        mods_locked,
                        0,
                        0,
                        group as usize,
                    );

                    let prev_layout = kbd.layout.as_ref();
                    let next_layout = KeyboardLayout(
                        kbd.keymap
                            .as_ref()
                            .unwrap()
                            .layout_get_name(group as usize)
                            .unwrap()
                            .to_string(),
                    );

                    if let Some(prev_layout) = prev_layout
                        && next_layout.0 != prev_layout.0
                    {
                        layout_has_changed = true;
                    }

                    kbd.layout = Some(next_layout);

                    let keymap_text = kbd.keymap_text.clone().unwrap();
                    let map = parse_group_layout_ids(&keymap_text);
                    let short = map
                        .get(group as usize)
                        .and_then(|x| x.clone())
                        .unwrap_or_else(|| String::new());

                    kbd.layout_short = Some(KeyboardLayout(short));

                    if layout_has_changed {
                        stream_emit!(state.stream_context, true, KeyboardLayoutChanged::from(kbd));
                    }
                }
            }
            _ => {}
        }
    }
}
