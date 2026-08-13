use std::{
    path::Path,
    sync::mpsc::SyncSender,
    time::{Duration, SystemTime},
};

use nu_plugin::EngineInterface;
use nu_protocol::{LabeledError, PipelineData, Signals};
use zlob::walk::{WalkBuilder, WalkEntry, WalkFlags, WalkMetadata, WalkState};

use crate::{
    emit::{
        WalkItem, WalkedEntry, WalkedMeta, count_pipeline, maybe_sort_items, spawn_item_stream,
        stream_items, walk_root_entry,
    },
    options::WalkOptions,
};

pub fn run(options: WalkOptions, engine: &EngineInterface) -> Result<PipelineData, LabeledError> {
    let start = std::time::Instant::now();
    let signals = engine.signals().clone();
    let iter = walk_items(&options);
    finish(iter, options, start, signals)
}

pub(crate) fn walk_items(options: &WalkOptions) -> impl Iterator<Item = WalkItem> + Send + 'static {
    let options = options.clone();
    let first = (options.min_depth == 0).then(|| WalkItem::Entry(walk_root_entry(&options)));
    spawn_item_stream(first, move |tx| {
        if let Err(err) = stream_walk(tx, &options) {
            let _ = tx.send(WalkItem::Error(err));
        }
    })
}

fn finish<I>(
    iter: I,
    options: WalkOptions,
    start: std::time::Instant,
    signals: Signals,
) -> Result<PipelineData, LabeledError>
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

fn stream_walk(tx: &SyncSender<WalkItem>, options: &WalkOptions) -> Result<(), String> {
    let mut builder = WalkBuilder::new(&options.path).map_err(|err| err.to_string())?;
    builder.options(walk_flags(options));
    builder.threads(zlob_threads(options));
    builder.max_depth(zlob_max_depth(options));
    if options.metadata {
        builder.metadata(
            WalkMetadata::SIZE
                | WalkMetadata::MTIME
                | WalkMetadata::ATIME
                | WalkMetadata::BTIME
                | WalkMetadata::MODE,
        );
    }

    builder
        .run(|entry| {
            if entry.path() == options.path {
                return WalkState::Continue;
            }
            if !should_yield(&entry, options) {
                return WalkState::Continue;
            }
            let skip = entry.is_dir() && is_skip_dir(&entry, options);
            match tx.send(WalkItem::Entry(to_walked(entry, options))) {
                Err(_) => WalkState::Quit,
                Ok(()) if skip => WalkState::SkipDir,
                Ok(()) => WalkState::Continue,
            }
        })
        .map_err(|err| err.to_string())?;
    Ok(())
}

fn walk_flags(options: &WalkOptions) -> WalkFlags {
    let mut flags = WalkFlags::empty();
    if options.skip_hidden {
        flags |= WalkFlags::SKIP_HIDDEN;
    }
    if options.follow_links {
        flags |= WalkFlags::FOLLOW_SYMLINKS;
    }
    flags
}

fn zlob_threads(options: &WalkOptions) -> usize {
    match options.threads {
        // zlob: 0 = one worker per CPU, 1 = calling thread
        None => 0,
        Some(0) => 1,
        Some(n) => n,
    }
}

fn zlob_max_depth(options: &WalkOptions) -> Option<usize> {
    if options.max_depth == usize::MAX {
        None
    } else {
        Some(options.max_depth)
    }
}

fn should_yield(entry: &WalkEntry<'_>, options: &WalkOptions) -> bool {
    entry.depth() >= options.min_depth
}

fn is_skip_dir(entry: &WalkEntry<'_>, options: &WalkOptions) -> bool {
    entry
        .path()
        .file_name()
        .is_some_and(|name| options.should_skip_dir_name(name))
}

fn to_walked(entry: WalkEntry<'_>, options: &WalkOptions) -> WalkedEntry {
    let full_path = entry.path().to_path_buf();
    let file_name = full_path
        .file_name()
        .unwrap_or_else(|| full_path.as_os_str())
        .to_os_string();
    let parent_path = full_path
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .to_path_buf();

    let metadata = if options.metadata {
        Some(WalkedMeta {
            accessed: ns_to_system_time(entry.accessed_ns()),
            created: ns_to_system_time(entry.created_ns()),
            modified: ns_to_system_time(entry.modified_ns()),
            size: entry.size().unwrap_or(0),
            readonly: entry.mode().is_some_and(|mode| mode & 0o222 == 0),
        })
    } else {
        None
    };

    WalkedEntry {
        depth: entry.depth(),
        file_name,
        full_path,
        is_dir: entry.is_dir(),
        is_file: entry.is_file(),
        is_symlink: entry.is_symlink(),
        parent_path,
        path_is_symlink: entry.is_symlink(),
        client_state: false,
        metadata,
    }
}

fn ns_to_system_time(ns: Option<i64>) -> Option<SystemTime> {
    let ns = ns?;
    if ns >= 0 {
        SystemTime::UNIX_EPOCH.checked_add(Duration::from_nanos(ns as u64))
    } else {
        SystemTime::UNIX_EPOCH.checked_sub(Duration::from_nanos(ns.unsigned_abs()))
    }
}
