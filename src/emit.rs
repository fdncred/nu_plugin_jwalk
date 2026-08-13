use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    sync::mpsc::{SyncSender, sync_channel},
    thread,
    time::{Duration, Instant, SystemTime},
};

use chrono::{DateTime, Local};
use nu_protocol::{ListStream, PipelineData, ShellError, Signals, Span, Value, record};

use crate::options::WalkOptions;

#[derive(Debug)]
pub struct WalkedMeta {
    pub accessed: Option<SystemTime>,
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
    pub size: u64,
    pub readonly: bool,
}

#[derive(Debug)]
pub struct WalkedEntry {
    pub depth: usize,
    pub file_name: OsString,
    pub full_path: PathBuf,
    pub is_dir: bool,
    pub is_file: bool,
    pub is_symlink: bool,
    pub parent_path: PathBuf,
    pub path_is_symlink: bool,
    pub client_state: bool,
    pub metadata: Option<WalkedMeta>,
}

#[derive(Debug)]
pub enum WalkItem {
    Entry(WalkedEntry),
    Error(String),
}

pub fn path_value(entry: &WalkedEntry, span: Span) -> Value {
    Value::string(entry.full_path.to_string_lossy().into_owned(), span)
}

pub fn record_value(entry: WalkedEntry, options: &WalkOptions, span: Span) -> Value {
    let mut rec = record! {
        "depth" => Value::int(entry.depth as i64, span),
        "client_state" => Value::bool(entry.client_state, span),
        "file_name" => Value::string(entry.file_name.to_string_lossy().into_owned(), span),
        "full_path" => Value::string(entry.full_path.to_string_lossy().into_owned(), span),
        "is_dir" => Value::bool(entry.is_dir, span),
        "is_file" => Value::bool(entry.is_file, span),
        "is_symlink" => Value::bool(entry.is_symlink, span),
        "parent_path" => Value::string(entry.parent_path.to_string_lossy().into_owned(), span),
        "path_is_symlink" => Value::bool(entry.path_is_symlink, span),
    };

    if options.metadata {
        match entry.metadata {
            Some(meta) => {
                rec.push("accessed", system_time_value(meta.accessed, span));
                rec.push("created", system_time_value(meta.created, span));
                rec.push("modified", system_time_value(meta.modified, span));
                rec.push("size", Value::filesize(meta.size as i64, span));
                rec.push("readonly", Value::bool(meta.readonly, span));
            }
            None => {
                rec.push("accessed", Value::string(String::new(), span));
                rec.push("created", Value::string(String::new(), span));
                rec.push("modified", Value::string(String::new(), span));
                rec.push("size", Value::int(0, span));
                rec.push("readonly", Value::string(String::new(), span));
            }
        }
    }

    Value::record(rec, span)
}

pub fn error_value(message: String, span: Span) -> Value {
    Value::error(
        ShellError::LabeledError(Box::new(
            LabeledErrorShim::new(message).with_label("Error found with walk entry", span),
        )),
        span,
    )
}

// nu_protocol::LabeledError is used at the command layer; keep this helper local.
use nu_protocol::LabeledError as LabeledErrorShim;

fn system_time_value(time: Option<SystemTime>, span: Span) -> Value {
    match time {
        Some(time) => {
            let dt: DateTime<Local> = time.into();
            Value::date(dt.into(), span)
        }
        None => Value::string(String::new(), span),
    }
}

pub fn item_to_value(item: WalkItem, options: &WalkOptions, span: Span) -> Value {
    match item {
        WalkItem::Error(message) => error_value(message, span),
        WalkItem::Entry(entry) if options.records() => record_value(entry, options, span),
        WalkItem::Entry(entry) => path_value(&entry, span),
    }
}

pub fn walk_root_entry(options: &WalkOptions) -> WalkedEntry {
    let meta = std::fs::symlink_metadata(&options.path).ok();
    let file_type = meta.as_ref().map(|m| m.file_type());
    let metadata = if options.metadata {
        meta.as_ref().map(|m| WalkedMeta {
            accessed: m.accessed().ok(),
            created: m.created().ok(),
            modified: m.modified().ok(),
            size: m.len(),
            readonly: m.permissions().readonly(),
        })
    } else {
        None
    };
    WalkedEntry {
        depth: 0,
        file_name: options
            .path
            .file_name()
            .unwrap_or_else(|| options.path.as_os_str())
            .to_os_string(),
        full_path: options.path.clone(),
        is_dir: file_type.is_some_and(|ft| ft.is_dir()),
        is_file: file_type.is_some_and(|ft| ft.is_file()),
        is_symlink: file_type.is_some_and(|ft| ft.is_symlink()),
        parent_path: options
            .path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_path_buf(),
        path_is_symlink: file_type.is_some_and(|ft| ft.is_symlink()),
        client_state: false,
        metadata,
    }
}

pub fn is_root_item(item: &WalkItem, root: &Path) -> bool {
    matches!(item, WalkItem::Entry(entry) if entry.depth == 0 || entry.full_path == root)
}

/// Push walker output through a channel so `run` can return a live `ListStream`.
/// `first` is sent on this thread so the stream is not empty while workers start.
pub fn spawn_item_stream<F>(
    first: Option<WalkItem>,
    produce: F,
) -> impl Iterator<Item = WalkItem> + Send + 'static
where
    F: FnOnce(&SyncSender<WalkItem>) + Send + 'static,
{
    let (tx, rx) = sync_channel(1024);
    if let Some(item) = first {
        let _ = tx.send(item);
    }
    let send = tx.clone();
    if let Err(err) = thread::Builder::new()
        .name("jwalk-walk".into())
        .spawn(move || produce(&send))
    {
        let _ = tx.send(WalkItem::Error(err.to_string()));
    }
    drop(tx);
    rx.into_iter()
}

pub fn send_walk_iter<I>(tx: &SyncSender<WalkItem>, iter: I)
where
    I: IntoIterator<Item = WalkItem>,
{
    for item in iter {
        if tx.send(item).is_err() {
            break;
        }
    }
}

pub fn maybe_sort_items(mut items: Vec<WalkItem>, sort: bool) -> Vec<WalkItem> {
    if !sort {
        return items;
    }
    items.sort_by(|left, right| match (left, right) {
        (WalkItem::Entry(a), WalkItem::Entry(b)) => a.file_name.cmp(&b.file_name),
        (WalkItem::Entry(_), WalkItem::Error(_)) => std::cmp::Ordering::Less,
        (WalkItem::Error(_), WalkItem::Entry(_)) => std::cmp::Ordering::Greater,
        (WalkItem::Error(_), WalkItem::Error(_)) => std::cmp::Ordering::Equal,
    });
    items
}

pub fn stream_items<I>(
    items: I,
    options: WalkOptions,
    start: Instant,
    signals: Signals,
) -> PipelineData
where
    I: Iterator<Item = WalkItem> + Send + 'static,
{
    let span = options.span;
    let debug = options.debug;
    let iter = items.map({
        let options = options.clone();
        move |item| item_to_value(item, &options, span)
    });

    if debug {
        let debug_options = options.clone();
        let extra = std::iter::from_fn(move || {
            eprintln!("{}", debug_options.debug_summary(start.elapsed(), None));
            None
        });
        ListStream::new(iter.chain(extra), span, signals).into()
    } else {
        ListStream::new(iter, span, signals).into()
    }
}

pub fn count_pipeline(count: u64, options: &WalkOptions, elapsed: Duration) -> PipelineData {
    if options.debug {
        eprintln!("{}", options.debug_summary(elapsed, Some(count)));
    }
    PipelineData::value(Value::int(count as i64, options.span), None)
}
