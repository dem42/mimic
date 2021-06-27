use std::{ffi::OsStr, path::PathBuf};
//////////////////////// Fns ///////////////////////
fn is_build_dir(dir_name: Option<&OsStr>) -> bool {
    match dir_name {
        Some(dir_name) => dir_name == "build",
        None => false,
    }
}

/// This is a hack because we shouldn't be doing things outside of OUT_DIR
/// at least not from within library crates. In binary crates it may be ok to try to get the target dir this way.
/// In library crates, we should probably use something like include_bytes! to add resources to the binary
pub fn get_target_from_out_dir(mut out_dir: PathBuf) -> Option<PathBuf> {
    let mut failed_to_find_target = false;
    while !out_dir.is_dir() || !is_build_dir(out_dir.file_name()) {
        if !out_dir.pop() {
            failed_to_find_target = true;
            break;
        }
    }

    if failed_to_find_target {
        return None;
    }
    // remove /build parent dir
    if !out_dir.pop() {
        return None;
    }
    Some(out_dir)
}
