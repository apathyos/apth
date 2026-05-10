use anyhow::Context;

pub fn get_runtime_dir() -> anyhow::Result<String> {
    std::env::var("XDG_RUNTIME_DIR").context("XDG_RUNTIME_DIR is not set")
}
