use crate::{
    emit::{
        WalkItem, WalkedEntry, WalkedMeta, count_pipeline, is_root_item, send_walk_iter,
        spawn_item_stream, stream_items, walk_root_entry,
    },
    options::WalkOptions,
};
use jwalk::{DirEntry, Parallelism, WalkDir, WalkDirGeneric};
use nu_plugin::EngineInterface;
use nu_protocol::PipelineData;
use std::{cmp::Ordering, fs::Metadata, sync::Arc};

pub fn run(
    options: WalkOptions,
    engine: &EngineInterface,
) -> Result<PipelineData, nu_protocol::LabeledError> {
    let start = std::time::Instant::now();
    let signals = engine.signals().clone();
    if options.custom {
        return finish(walk_custom(&options), options, start, signals);
    }
    finish(walk_items(&options), options, start, signals)
}

pub(crate) fn walk_items(options: &WalkOptions) -> impl Iterator<Item = WalkItem> + Send + 'static {
    let options = options.clone();
    let first = (options.min_depth == 0).then(|| WalkItem::Entry(walk_root_entry(&options)));
    spawn_item_stream(first, move |tx| {
        send_walk_iter(
            tx,
            jwalk_iter(&options).filter(|item| !is_root_item(item, &options.path)),
        );
    })
}

fn jwalk_iter(options: &WalkOptions) -> impl Iterator<Item = WalkItem> + Send + 'static {
    let parallelism = jwalk_parallelism(options);
    let skip_dirs = Arc::clone(&options.skip_dirs);
    let want_metadata = options.metadata;
    let span_path = options.path.clone();

    WalkDir::new(&span_path)
        .sort(options.sort)
        .skip_hidden(options.skip_hidden)
        .follow_links(options.follow_links)
        .min_depth(options.min_depth)
        .max_depth(options.max_depth)
        .parallelism(parallelism)
        .process_read_dir(move |_depth, _path, _state, children| {
            if skip_dirs.is_empty() {
                return;
            }
            prune_skip_dirs(children, &skip_dirs);
        })
        .into_iter()
        .map(move |entry| match entry {
            Ok(entry) => WalkItem::Entry(from_jwalk(entry, want_metadata)),
            Err(err) => WalkItem::Error(err.to_string()),
        })
}

fn walk_custom(options: &WalkOptions) -> impl Iterator<Item = WalkItem> + Send + 'static {
    let options = options.clone();
    spawn_item_stream(None, move |tx| {
        send_walk_iter(tx, custom_iter(&options));
    })
}

fn custom_iter(options: &WalkOptions) -> impl Iterator<Item = WalkItem> + Send + 'static {
    let parallelism = jwalk_parallelism(options);
    let want_metadata = options.metadata;
    WalkDirGeneric::<(usize, bool)>::new(&options.path)
        .process_read_dir(|_depth, _path, read_dir_state, children| {
            children.sort_by(|a, b| match (a, b) {
                (Ok(a), Ok(b)) => a.file_name.cmp(&b.file_name),
                (Ok(_), Err(_)) => Ordering::Less,
                (Err(_), Ok(_)) => Ordering::Greater,
                (Err(_), Err(_)) => Ordering::Equal,
            });
            children.retain(|dir_entry_result| {
                dir_entry_result
                    .as_ref()
                    .map(|dir_entry| {
                        dir_entry
                            .file_name
                            .to_str()
                            .map(|s| s.starts_with('.'))
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            });
            children.iter_mut().for_each(|dir_entry_result| {
                if let Ok(dir_entry) = dir_entry_result
                    && dir_entry.depth == 2
                {
                    dir_entry.read_children = None;
                }
            });
            *read_dir_state += 1;
            if let Some(Ok(dir_entry)) = children.first_mut() {
                dir_entry.client_state = true;
            }
        })
        .skip_hidden(options.skip_hidden)
        .follow_links(options.follow_links)
        .min_depth(options.min_depth)
        .max_depth(options.max_depth)
        .parallelism(parallelism)
        .into_iter()
        .map(move |entry| match entry {
            Ok(entry) => WalkItem::Entry(from_jwalk_generic(entry, want_metadata)),
            Err(err) => WalkItem::Error(err.to_string()),
        })
}

fn finish<I>(
    iter: I,
    options: WalkOptions,
    start: std::time::Instant,
    signals: nu_protocol::Signals,
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

    Ok(stream_items(iter, options, start, signals))
}

fn jwalk_parallelism(options: &WalkOptions) -> Parallelism {
    match options.threads {
        Some(0) => Parallelism::Serial,
        Some(n) => Parallelism::RayonNewPool(n),
        None => Parallelism::RayonDefaultPool {
            busy_timeout: std::time::Duration::from_secs(1),
        },
    }
}

fn prune_skip_dirs<C: jwalk::ClientState>(
    children: &mut [jwalk::Result<DirEntry<C>>],
    skip_dirs: &[std::ffi::OsString],
) {
    children.iter_mut().for_each(|dir_entry_result| {
        if let Ok(dir_entry) = dir_entry_result
            && dir_entry.file_type.is_dir()
            && skip_dirs
                .iter()
                .any(|name| name.as_os_str() == dir_entry.file_name.as_os_str())
        {
            dir_entry.read_children = None;
        }
    });
}

fn from_jwalk<C: jwalk::ClientState>(entry: DirEntry<C>, want_metadata: bool) -> WalkedEntry {
    from_jwalk_with_state(entry, want_metadata, false)
}

fn from_jwalk_generic(entry: DirEntry<(usize, bool)>, want_metadata: bool) -> WalkedEntry {
    let client_state = entry.client_state;
    from_jwalk_with_state(entry, want_metadata, client_state)
}

fn from_jwalk_with_state<C: jwalk::ClientState>(
    entry: DirEntry<C>,
    want_metadata: bool,
    client_state: bool,
) -> WalkedEntry {
    let metadata = if want_metadata {
        entry.metadata().ok().map(meta_from_std)
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
        parent_path: entry.parent_path().to_path_buf(),
        path_is_symlink: entry.path_is_symlink(),
        client_state,
        metadata,
    }
}

fn meta_from_std(meta: Metadata) -> WalkedMeta {
    WalkedMeta {
        accessed: meta.accessed().ok(),
        created: meta.created().ok(),
        modified: meta.modified().ok(),
        size: meta.len(),
        readonly: meta.permissions().readonly(),
    }
}
