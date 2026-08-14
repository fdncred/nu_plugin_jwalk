use crate::{
    emit::{
        WalkItem, WalkedEntry, finish_walk, is_root_item, send_walk_iter, spawn_item_stream,
        walk_root_entry, walked_meta_from_std,
    },
    options::WalkOptions,
};
use ignore::{DirEntry, WalkBuilder, WalkState};
use nu_plugin::EngineInterface;
use nu_protocol::{LabeledError, PipelineData};
use std::{path::Path, sync::mpsc::SyncSender};

pub fn run(options: WalkOptions, engine: &EngineInterface) -> Result<PipelineData, LabeledError> {
    let start = std::time::Instant::now();
    let signals = engine.signals().clone();
    finish_walk(walk_items(&options), options, start, signals)
}

pub(crate) fn walk_items(options: &WalkOptions) -> impl Iterator<Item = WalkItem> + Send + 'static {
    let options = options.clone();
    let first = (options.min_depth == 0).then(|| WalkItem::Entry(walk_root_entry(&options)));
    spawn_item_stream(first, move |tx| stream_walk(tx, &options))
}

fn stream_walk(tx: &SyncSender<WalkItem>, options: &WalkOptions) {
    let mut builder = configured_builder(options);
    if ignore_use_parallel(options) {
        builder.threads(ignore_threads(options));
        builder.build_parallel().run(|| {
            let tx = tx.clone();
            let options = options.clone();
            Box::new(move |result| match result {
                Ok(entry) => send_ignore_entry(&tx, entry, &options),
                Err(err) => {
                    if tx.send(WalkItem::Error(err.to_string())).is_err() {
                        WalkState::Quit
                    } else {
                        WalkState::Continue
                    }
                }
            })
        });
        return;
    }

    if options.sort {
        builder.sort_by_file_name(|left, right| left.cmp(right));
    }
    send_walk_iter(tx, serial_items(builder, options));
}

fn serial_items(
    builder: WalkBuilder,
    options: &WalkOptions,
) -> impl Iterator<Item = WalkItem> + '_ {
    builder.build().filter_map(move |result| match result {
        Ok(entry) => match classify_entry(entry, options) {
            Classified::SkipRoot | Classified::Drop => None,
            Classified::Emit { item, .. } => Some(item),
        },
        Err(err) => Some(WalkItem::Error(err.to_string())),
    })
}

fn send_ignore_entry(
    tx: &SyncSender<WalkItem>,
    entry: DirEntry,
    options: &WalkOptions,
) -> WalkState {
    match classify_entry(entry, options) {
        Classified::Drop => WalkState::Continue,
        Classified::SkipRoot => WalkState::Skip,
        Classified::Emit { item, skip } => match tx.send(item) {
            Err(_) => WalkState::Quit,
            Ok(()) if skip => WalkState::Skip,
            Ok(()) => WalkState::Continue,
        },
    }
}

enum Classified {
    Drop,
    SkipRoot,
    Emit { item: WalkItem, skip: bool },
}

fn classify_entry(entry: DirEntry, options: &WalkOptions) -> Classified {
    let walked = to_walked(entry, options.metadata);
    let skip_dir = walked.is_dir && options.should_skip_dir_name(&walked.file_name);
    let item = WalkItem::Entry(walked);
    if is_root_item(&item, &options.path) {
        return if skip_dir {
            Classified::SkipRoot
        } else {
            Classified::Drop
        };
    }
    if let WalkItem::Entry(ref walked) = item
        && walked.depth > 0
        && under_skipped_dir(&walked.full_path, &options.path, options)
    {
        return Classified::Drop;
    }
    Classified::Emit {
        item,
        skip: skip_dir,
    }
}

fn configured_builder(options: &WalkOptions) -> WalkBuilder {
    let mut builder = WalkBuilder::new(&options.path);
    // Comparable to the other engines: do not apply gitignore / .ignore / hidden by default.
    builder
        .standard_filters(false)
        .hidden(options.skip_hidden)
        .follow_links(options.follow_links)
        .min_depth(Some(options.min_depth))
        .max_depth(ignore_max_depth(options));
    builder
}

fn ignore_use_parallel(options: &WalkOptions) -> bool {
    // sort_by_file_name is sequential-only; --threads 0/1 is serial.
    !options.sort && !matches!(options.threads, Some(0 | 1))
}

fn ignore_threads(options: &WalkOptions) -> usize {
    match options.threads {
        None | Some(0) => 0,
        Some(n) => n,
    }
}

fn ignore_max_depth(options: &WalkOptions) -> Option<usize> {
    (options.max_depth != usize::MAX).then_some(options.max_depth)
}

fn under_skipped_dir(path: &Path, root: &Path, options: &WalkOptions) -> bool {
    if options.skip_dirs.is_empty() {
        return false;
    }
    if path != root
        && root
            .file_name()
            .is_some_and(|name| options.should_skip_dir_name(name))
    {
        return true;
    }
    let mut dir = path.parent();
    while let Some(current) = dir {
        if current == root {
            break;
        }
        if current
            .file_name()
            .is_some_and(|name| options.should_skip_dir_name(name))
        {
            return true;
        }
        dir = current.parent();
    }
    false
}

fn to_walked(entry: DirEntry, want_metadata: bool) -> WalkedEntry {
    let file_type = entry.file_type();
    let full_path = entry.path().to_path_buf();
    let metadata = if want_metadata {
        entry.metadata().ok().map(walked_meta_from_std)
    } else {
        None
    };
    WalkedEntry {
        depth: entry.depth(),
        file_name: entry.file_name().to_os_string(),
        parent_path: full_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_path_buf(),
        full_path,
        is_dir: file_type.is_some_and(|ft| ft.is_dir()),
        is_file: file_type.is_some_and(|ft| ft.is_file()),
        is_symlink: file_type.is_some_and(|ft| ft.is_symlink()),
        path_is_symlink: entry.path_is_symlink(),
        client_state: false,
        metadata,
    }
}
