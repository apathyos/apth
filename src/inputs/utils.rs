use std::fs::File;

use memmap2;

pub fn read_keymap_from_fd(fd: std::os::fd::OwnedFd, size: u32) -> Result<String, String> {
    let file = File::from(fd);

    let mmap = unsafe {
        memmap2::MmapOptions::new()
            .len(size as usize)
            .map_copy_read_only(&file)
            .expect("mmap keymap fd failed")
    };

    let bytes = &mmap[..];
    let nul = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let txt = std::str::from_utf8(&bytes[..nul]).expect("keymap is not utf-8");

    Ok(txt.to_string())
}

pub fn parse_group_layout_ids(keymap: &str) -> Vec<Option<String>> {
    // Attempt 1: xkb_symbols { include "pc+us+ru:2+..." };
    if let Some(inc) = extract_symbols_include_string(keymap) {
        let v = build_group_map_from_include(&inc);
        if !v.is_empty() {
            return v;
        }
    }

    // Attempt 2: xkb_symbols "pc_us_ru_2_inet(evdev)_..." { ... }
    if let Some(sym_name) = extract_symbols_section_name(keymap) {
        let v = build_group_map_from_symbols_section_name(&sym_name);
        if !v.is_empty() {
            return v;
        }
    }

    Vec::new()
}

fn extract_symbols_include_string(keymap: &str) -> Option<String> {
    // find xkb_symbvols block
    let sym_pos = keymap.find("xkb_symbols")?;
    let after = &keymap[sym_pos..];
    let open = sym_pos + after.find('{')?;
    let close = open + 1 + keymap[open + 1..].find('}')?;
    let block = &keymap[open + 1..close];

    // include "...."
    let inc = block.find("include")?;
    let block2 = &block[inc..];
    let q1 = block2.find('"')? + inc;
    let q2 = q1 + 1 + block[q1 + 1..].find('"')?;
    Some(block[q1 + 1..q2].trim().to_string())
}

fn extract_symbols_section_name(keymap: &str) -> Option<String> {
    // find xkb_symbols "...." {
    let sym_pos = keymap.find("xkb_symbols")?;
    let after = &keymap[sym_pos..];
    let q1_rel = after.find('"')?;
    let q1 = sym_pos + q1_rel + 1;
    let q2 = q1 + keymap[q1..].find('"')?;
    Some(keymap[q1..q2].trim().to_string())
}

fn build_group_map_from_include(include: &str) -> Vec<Option<String>> {
    // include-chain: pc+us+ru:2+inet(evdev)...
    let mut out: Vec<Option<String>> = Vec::new();

    for token in include
        .split('+')
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
    {
        let base = token.split('(').next().unwrap_or(token);
        let (name, group_1based) = if let Some((l, r)) = base.rsplit_once(':') {
            (l, r.parse::<u32>().unwrap_or(1))
        } else {
            (base, 1)
        };

        if !looks_like_layout_id(name) {
            continue;
        }

        let idx = group_1based.saturating_sub(1) as usize;
        if out.len() <= idx {
            out.resize(idx + 1, None);
        }
        if out[idx].is_none() {
            out[idx] = Some(name.to_string());
        }
    }

    out
}

fn build_group_map_from_symbols_section_name(name: &str) -> Vec<Option<String>> {
    let mut out: Vec<Option<String>> = Vec::new();

    let mut s = name.trim();

    // strip prefix
    if let Some(rest) = s.strip_prefix("pc_") {
        s = rest;
    }

    // strip unnecessary
    for cut in ["_inet", "_evdev", "_group", "_compose"] {
        if let Some(i) = s.find(cut) {
            s = &s[..i];
            break;
        }
    }

    let parts: Vec<&str> = s.split('_').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return out;
    }

    // collect layout groups
    for p in parts {
        if p.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        if !looks_like_layout_id(p) {
            continue;
        }
        out.push(Some(p.to_string()));
    }

    out
}

fn looks_like_layout_id(s: &str) -> bool {
    let s = s.trim();
    if s.len() < 2 || s.len() > 12 {
        return false;
    }
    if matches!(s, "pc" | "inet" | "evdev" | "complete" | "compose") {
        return false;
    }
    s.bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
