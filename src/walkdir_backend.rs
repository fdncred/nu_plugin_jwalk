use crate::{
    emit::{
        WalkItem, WalkedEntry, finish_walk, spawn_item_stream, walk_root_entry,
        walked_meta_from_std,
    },
    options::{WalkOptions, is_hidden_name},
};
use nu_plugin::EngineInterface;
use nu_protocol::{LabeledError, PipelineData};
use std::{path::Path, sync::mpsc::SyncSender};
use walkdir::{DirEntry, WalkDir};

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
    let mut walker = WalkDir::new(&options.path)
        .min_depth(options.min_depth)
        .max_depth(options.max_depth)
        .follow_links(options.follow_links);
    if options.sort {
        walker = walker.sort_by_file_name();
    }

    let mut iter = walker.into_iter();
    loop {
        let entry = match iter.next() {
            None => break,
            Some(Err(err)) => {
                if tx.send(WalkItem::Error(err.to_string())).is_err() {
                    break;
                }
                continue;
            }
            Some(Ok(entry)) => entry,
        };

        if entry.depth() > 0 && options.skip_hidden && is_hidden_name(entry.file_name()) {
            if entry.file_type().is_dir() {
                iter.skip_current_dir();
            }
            continue;
        }

        let skip_dir =
            entry.file_type().is_dir() && options.should_skip_dir_name(entry.file_name());

        let is_root = entry.depth() == 0 || entry.path() == options.path;
        if is_root {
            if skip_dir {
                iter.skip_current_dir();
            }
            continue;
        }

        if tx
            .send(WalkItem::Entry(to_walked(entry, options.metadata)))
            .is_err()
        {
            break;
        }
        if skip_dir {
            iter.skip_current_dir();
        }
    }
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
        is_dir: file_type.is_dir(),
        is_file: file_type.is_file(),
        is_symlink: file_type.is_symlink(),
        path_is_symlink: entry.path_is_symlink(),
        client_state: false,
        metadata,
    }
}
