use dua_core::{Order, walk};
use nu_plugin::EngineInterface;
use nu_protocol::{PipelineData, Signals};

use crate::{
    emit::{
        WalkItem, WalkedEntry, WalkedMeta, count_pipeline, is_root_item, maybe_sort_items,
        send_walk_iter, spawn_item_stream, stream_items, walk_root_entry,
    },
    options::{WalkOptions, WalkOrder, is_hidden_name},
};

pub fn run(
    options: WalkOptions,
    engine: &EngineInterface,
) -> Result<PipelineData, nu_protocol::LabeledError> {
    let start = std::time::Instant::now();
    let signals = engine.signals().clone();
    let iter = walk_items(&options);
    finish(iter, options, start, signals)
}

pub(crate) fn walk_items(options: &WalkOptions) -> impl Iterator<Item = WalkItem> + Send + 'static {
    let options = options.clone();
    let first = (options.min_depth == 0).then(|| WalkItem::Entry(walk_root_entry(&options)));
    spawn_item_stream(first, move |tx| {
        send_walk_iter(
            tx,
            dua_iter(&options).filter(|item| !is_root_item(item, &options.path)),
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

fn finish<I>(
    iter: I,
    options: WalkOptions,
    start: std::time::Instant,
    signals: Signals,
) -> Result<PipelineData, nu_protocol::LabeledError>
where
    I: Iterator<Item = WalkItem> + Send + 'static,
{
    if options.count {
        let count = iter
            .filter(|item| matches!(item, WalkItem::Entry(_)))
            .count() as u64;
        return Ok(count_pipeline(count, &options, start.elapsed()));
    }

    if options.sort {
        let items = maybe_sort_items(iter.collect(), true);
        return Ok(stream_items(items.into_iter(), options, start, signals));
    }

    Ok(stream_items(iter, options, start, signals))
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
        entry.metadata.as_ref().ok().map(|meta| WalkedMeta {
            accessed: meta.accessed().ok(),
            created: meta.created().ok(),
            modified: meta.modified().ok(),
            size: meta.len(),
            readonly: meta.permissions().readonly(),
        })
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
