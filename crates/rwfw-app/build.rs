//! Flatten the app + module web roots into `$OUT_DIR` by canonical key so they
//! can be embedded (via `rust-embed`) into the binary. Templates are keyed by
//! the MiniJinja lookup name (module pages prefixed with `<module>/`, app pages
//! at the root); vendor + assets are copied verbatim.

use std::path::{Path, PathBuf};

fn main() {
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let app_web = manifest.join("web");

    // App templates -> canonical root (no prefix).
    let tdst = out.join("embed_templates");
    let _ = std::fs::remove_dir_all(&tdst);
    copy_tree(&app_web.join("templates"), &tdst);
    rerun(&app_web.join("templates"));

    // Module templates -> "<module>/" prefix. Keep in sync with workspace
    // members / the `extern crate` list in src/lib.rs.
    let modules_dir = manifest.join("..").join("modules");
    for m in ["home", "auth", "blog", "demo"] {
        let src = modules_dir.join(m).join("web").join("templates");
        if src.is_dir() {
            copy_tree(&src, &tdst.join(m));
            rerun(&src);
        }
    }

    // Vendor (locked JS) + assets, copied verbatim.
    let vdst = out.join("embed_vendor");
    let _ = std::fs::remove_dir_all(&vdst);
    copy_tree(&app_web.join("vendor"), &vdst);
    rerun(&app_web.join("vendor"));

    let adst = out.join("embed_assets");
    let _ = std::fs::remove_dir_all(&adst);
    copy_tree(&app_web.join("assets"), &adst);
    rerun(&app_web.join("assets"));
}

fn rerun(p: &Path) {
    println!("cargo:rerun-if-changed={}", p.display());
}

fn copy_tree(src: &Path, dst: &Path) {
    if !src.is_dir() {
        return;
    }
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src).unwrap().flatten() {
        let from = e.path();
        let to = dst.join(e.file_name());
        if from.is_dir() {
            copy_tree(&from, &to);
        } else {
            std::fs::copy(&from, &to).unwrap();
        }
    }
}
