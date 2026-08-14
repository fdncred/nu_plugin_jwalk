use crate::{
    emit::{
        WalkItem, WalkedEntry, WalkedMeta, finish_walk, is_root_item, send_maybe_sorted,
        spawn_item_stream, walk_root_entry,
    },
    options::{WalkOptions, WalkOrder, is_hidden_name},
};
use dua_core::{Order, walk};
use nu_plugin::EngineInterface;
use nu_protocol::PipelineData;
use std::time::SystemTime;

pub fn run(
    options: WalkOptions,
    engine: &EngineInterface,
) -> Result<PipelineData, nu_protocol::LabeledError> {
    let start = std::time::Instant::now();
    let signals = engine.signals().clone();
    finish_walk(walk_items(&options), options, start, signals)
}

pub(crate) fn walk_items(options: &WalkOptions) -> impl Iterator<Item = WalkItem> + Send + 'static {
    let options = options.clone();
    let first = (options.min_depth == 0).then(|| WalkItem::Entry(walk_root_entry(&options)));
    spawn_item_stream(first, move |tx| {
        send_maybe_sorted(
            tx,
            dua_iter(&options).filter(|item| !is_root_item(item, &options.path)),
            options.sort,
        );
    })
}

fn dua_iter(options: &WalkOptions) -> impl Iterator<Item = WalkItem> + Send + 'static {
    let threads = dua_threads(options);
    let order = match options.order {
        WalkOrder::Completion => Order::Completion,
        WalkOrder::ParentFirst => Order::ParentFirst,
    };

    let walk_options = options.clone();
    walk(&options.path, threads, order, {
        let walk_options = walk_options.clone();
        move |entry| should_descend(entry, &walk_options)
    })
    .filter_map(move |result| match result {
        Ok(entry) => {
            if !should_yield(&entry, &walk_options) {
                return None;
            }
            Some(WalkItem::Entry(to_walked(entry, walk_options.metadata)))
        }
        Err(err) => Some(WalkItem::Error(err.to_string())),
    })
}

fn dua_threads(options: &WalkOptions) -> usize {
    match options.threads {
        Some(0) | Some(1) => 1,
        Some(n) => n,
        None => std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1),
    }
}

fn should_descend(entry: &dua_core::Entry, options: &WalkOptions) -> bool {
    if entry.depth >= options.max_depth {
        return false;
    }
    // The walk root is always entered, even when its name starts with '.'.
    if entry.depth > 0 && options.skip_hidden && is_hidden_name(&entry.file_name) {
        return false;
    }
    if entry.file_type.is_dir() && options.should_skip_dir_name(&entry.file_name) {
        return false;
    }
    true
}

fn should_yield(entry: &dua_core::Entry, options: &WalkOptions) -> bool {
    if entry.depth < options.min_depth {
        return false;
    }
    if entry.depth > 0 && options.skip_hidden && is_hidden_name(&entry.file_name) {
        return false;
    }
    true
}

fn to_walked(entry: dua_core::Entry, want_metadata: bool) -> WalkedEntry {
    let metadata = if want_metadata {
        entry.metadata.as_ref().ok().map(walked_meta_from_dua)
    } else {
        None
    };

    WalkedEntry {
        depth: entry.depth,
        file_name: entry.file_name.clone(),
        full_path: entry.path(),
        is_dir: entry.file_type.is_dir(),
        is_file: entry.file_type.is_file(),
        is_symlink: entry.file_type.is_symlink(),
        parent_path: entry.parent_path.to_path_buf(),
        path_is_symlink: entry.file_type.is_symlink(),
        client_state: false,
        metadata,
    }
}

/// dua-core 3.0 exposes `std::fs::Metadata` on Linux and a native type on macOS/Windows.
/// Those native types always have `len` and `modified`; access/create/readonly exist only
/// on the std implementation.
fn walked_meta_from_dua(meta: &dua_core::Metadata) -> WalkedMeta {
    WalkedMeta {
        accessed: dua_accessed(meta),
        created: dua_created(meta),
        modified: meta.modified().ok(),
        size: meta.len(),
        readonly: dua_readonly(meta),
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
fn dua_accessed(meta: &dua_core::Metadata) -> Option<SystemTime> {
    meta.accessed().ok()
}

#[cfg(any(windows, target_os = "macos"))]
fn dua_accessed(_meta: &dua_core::Metadata) -> Option<SystemTime> {
    None
}

#[cfg(not(any(windows, target_os = "macos")))]
fn dua_created(meta: &dua_core::Metadata) -> Option<SystemTime> {
    meta.created().ok()
}

#[cfg(any(windows, target_os = "macos"))]
fn dua_created(_meta: &dua_core::Metadata) -> Option<SystemTime> {
    None
}

#[cfg(not(any(windows, target_os = "macos")))]
fn dua_readonly(meta: &dua_core::Metadata) -> bool {
    meta.permissions().readonly()
}

#[cfg(any(windows, target_os = "macos"))]
fn dua_readonly(_meta: &dua_core::Metadata) -> bool {
    false
}
