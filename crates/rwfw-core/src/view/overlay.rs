use std::path::PathBuf;

/// Resolve the disk-overlay root next to the running executable.
///
/// Auto-enabled iff a `web/` directory exists alongside the binary
/// (`<exe_dir>/web`). This lets a packaged app ship a single binary (assets
/// embedded) yet override templates and `/assets` per installation by dropping
/// files in `web/` next to it. `None` when there's no such folder or the exe
/// path can't be resolved.
pub fn exe_overlay_root() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let web = exe.parent()?.join("web");
    web.is_dir().then_some(web)
}
