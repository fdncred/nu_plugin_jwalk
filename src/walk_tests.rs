#[cfg(feature = "zlob")]
use crate::zlob_backend;
use crate::{
    dua_backend,
    emit::WalkItem,
    ignore_backend, jwalk_backend,
    options::{Engine, WalkOptions, WalkOrder},
    walkdir_backend,
};
use nu_protocol::Span;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

const ENGINES: &[Engine] = &[
    Engine::Dua,
    Engine::Ignore,
    Engine::Jwalk,
    Engine::Walkdir,
    #[cfg(feature = "zlob")]
    Engine::Zlob,
];

fn fixture_tree() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path();
    fs::write(root.join("visible.txt"), b"ok").unwrap();
    fs::write(root.join(".hidden.txt"), b"hid").unwrap();
    fs::create_dir(root.join(".hiddendir")).unwrap();
    fs::write(root.join(".hiddendir").join("secret.txt"), b"s").unwrap();
    fs::create_dir(root.join("target")).unwrap();
    fs::write(root.join("target").join("junk.txt"), b"j").unwrap();
    fs::create_dir(root.join("keep")).unwrap();
    fs::write(root.join("keep").join("file.txt"), b"f").unwrap();
    dir
}

fn options(root: &Path, engine: Engine) -> WalkOptions {
    WalkOptions {
        engine,
        path: root.to_path_buf(),
        span: Span::test_data(),
        sort: false,
        custom: false,
        skip_hidden: false,
        follow_links: false,
        min_depth: 0,
        max_depth: usize::MAX,
        threads: Some(2),
        skip_dirs: Arc::from([]),
        verbose: false,
        metadata: false,
        count: false,
        debug: false,
        order: WalkOrder::Completion,
    }
}

fn relative_names(root: &Path, items: impl Iterator<Item = WalkItem>) -> BTreeSet<String> {
    items
        .map(|item| match item {
            WalkItem::Entry(entry) => entry.full_path,
            WalkItem::Error(err) => panic!("walk error: {err}"),
        })
        .map(|path| {
            if path == root {
                return String::new();
            }
            path.strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect()
}

fn walk_names(options: &WalkOptions) -> BTreeSet<String> {
    relative_names(&options.path, walk_items(options).into_iter())
}

fn walk_items(options: &WalkOptions) -> Vec<WalkItem> {
    match options.engine {
        Engine::Dua => dua_backend::walk_items(options).collect(),
        Engine::Ignore => ignore_backend::walk_items(options).collect(),
        Engine::Jwalk => jwalk_backend::walk_items(options).collect(),
        Engine::Walkdir => walkdir_backend::walk_items(options).collect(),
        #[cfg(feature = "zlob")]
        Engine::Zlob => zlob_backend::walk_items(options).collect(),
    }
}

#[test]
fn default_lists_hidden_and_target_for_both_engines() {
    let dir = fixture_tree();
    for &engine in ENGINES {
        let names = walk_names(&options(dir.path(), engine));
        assert!(
            names.contains(".hidden.txt"),
            "{engine:?} should list hidden files by default: {names:?}"
        );
        assert!(
            names.contains("target/junk.txt"),
            "{engine:?} should descend into target by default: {names:?}"
        );
        assert!(
            names.contains("keep/file.txt"),
            "{engine:?} should list nested files: {names:?}"
        );
    }
}

#[test]
fn skip_hidden_drops_dotfiles_for_both_engines() {
    let dir = fixture_tree();
    for &engine in ENGINES {
        let mut opts = options(dir.path(), engine);
        opts.skip_hidden = true;
        let names = walk_names(&opts);
        assert!(
            !names
                .iter()
                .any(|name| name.starts_with('.') || name.contains("/.")),
            "{engine:?} --skip-hidden should drop hidden paths: {names:?}"
        );
        assert!(names.contains("visible.txt"), "{engine:?}: {names:?}");
        assert!(names.contains("keep/file.txt"), "{engine:?}: {names:?}");
    }
}

#[test]
fn skip_dir_yields_directory_but_not_children() {
    let dir = fixture_tree();
    for &engine in ENGINES {
        let mut opts = options(dir.path(), engine);
        opts.skip_dirs = Arc::from([std::ffi::OsString::from("target")]);
        let names = walk_names(&opts);
        assert!(
            names.contains("target"),
            "{engine:?} should yield the skipped directory: {names:?}"
        );
        assert!(
            !names.contains("target/junk.txt"),
            "{engine:?} should not descend into skipped directories: {names:?}"
        );
    }
}

#[test]
fn count_matches_streamed_length() {
    let dir = fixture_tree();
    for &engine in ENGINES {
        let items = walk_items(&options(dir.path(), engine));
        let streamed = items
            .iter()
            .filter(|item| matches!(item, WalkItem::Entry(_)))
            .count();
        assert!(streamed > 0, "{engine:?} should walk some entries");
        assert_eq!(
            streamed,
            items.len(),
            "{engine:?} should not emit walk errors"
        );
    }
}

#[test]
fn max_depth_one_does_not_list_nested_file() {
    let dir = fixture_tree();
    for &engine in ENGINES {
        let mut opts = options(dir.path(), engine);
        opts.max_depth = 1;
        let names = walk_names(&opts);
        assert!(
            !names.contains("keep/file.txt"),
            "{engine:?} --max-depth 1 should not list nested files: {names:?}"
        );
        assert!(names.contains("keep"), "{engine:?}: {names:?}");
        assert!(names.contains("visible.txt"), "{engine:?}: {names:?}");
    }
}

#[test]
fn first_item_is_the_root_for_all_engines() {
    let dir = fixture_tree();
    for &engine in ENGINES {
        let first = walk_items(&options(dir.path(), engine)).into_iter().next();
        match first {
            Some(WalkItem::Entry(entry)) => {
                assert_eq!(entry.depth, 0, "{engine:?}");
                assert_eq!(entry.full_path, dir.path(), "{engine:?}");
            }
            other => panic!("{engine:?}: expected root entry, got {other:?}"),
        }
    }
}

#[test]
fn dua_follow_links_errors() {
    let mut opts = options(PathBuf::from(".").as_path(), Engine::Dua);
    opts.follow_links = true;
    let err = opts.validate().expect_err("dua cannot follow links");
    assert!(
        err.to_string().contains("dua"),
        "error should mention the dua engine: {err}"
    );
}

#[test]
fn parse_accepts_ignore_and_walkdir() {
    assert_eq!(
        Engine::parse(Some("ignore"), Span::test_data()).unwrap(),
        Engine::Ignore
    );
    assert_eq!(
        Engine::parse(Some("walkdir"), Span::test_data()).unwrap(),
        Engine::Walkdir
    );
}

#[cfg(not(feature = "zlob"))]
#[test]
fn zlob_requires_feature() {
    let err = Engine::parse(Some("zlob"), Span::test_data()).expect_err("zlob needs a feature");
    assert!(
        err.to_string().contains("zlob feature"),
        "error should mention the feature: {err}"
    );
}

#[cfg(feature = "zlob")]
#[test]
fn parse_accepts_zlob_when_enabled() {
    assert_eq!(
        Engine::parse(Some("zlob"), Span::test_data()).unwrap(),
        Engine::Zlob
    );
}

#[test]
fn ignore_serial_and_sort_still_honor_skip_dir() {
    let dir = fixture_tree();
    for (engine, threads, sort) in [
        (Engine::Ignore, Some(0), false),
        (Engine::Ignore, Some(2), true),
        (Engine::Walkdir, Some(0), true),
    ] {
        let mut opts = options(dir.path(), engine);
        opts.threads = threads;
        opts.sort = sort;
        opts.skip_dirs = Arc::from([std::ffi::OsString::from("target")]);
        let names = walk_names(&opts);
        assert!(
            names.contains("target"),
            "{engine:?} threads={threads:?} sort={sort} should yield the skipped directory: {names:?}"
        );
        assert!(
            !names.contains("target/junk.txt"),
            "{engine:?} threads={threads:?} sort={sort} should not descend: {names:?}"
        );
        let first = walk_items(&opts).into_iter().next();
        match first {
            Some(WalkItem::Entry(entry)) => {
                assert_eq!(entry.full_path, dir.path(), "{engine:?}");
            }
            other => panic!("{engine:?}: expected streamed root first, got {other:?}"),
        }
    }
}

#[test]
fn metadata_is_optional_on_walked_entries() {
    let dir = fixture_tree();
    for &engine in ENGINES {
        let without = walk_items(&options(dir.path(), engine));
        let without_meta = without.iter().find_map(|item| match item {
            WalkItem::Entry(entry) => Some(entry),
            WalkItem::Error(_) => None,
        });
        assert!(
            without_meta.is_some_and(|entry| entry.metadata.is_none()),
            "{engine:?} should omit metadata unless requested"
        );

        let mut opts = options(dir.path(), engine);
        opts.metadata = true;
        let with = walk_items(&opts);
        let with_meta = with.iter().find_map(|item| match item {
            WalkItem::Entry(entry) => Some(entry),
            WalkItem::Error(_) => None,
        });
        assert!(
            with_meta.is_some_and(|entry| entry.metadata.is_some()),
            "{engine:?} should populate metadata when requested"
        );
    }
}

#[test]
fn dua_metadata_includes_size_and_mtime() {
    let dir = fixture_tree();
    let mut opts = options(dir.path(), Engine::Dua);
    opts.metadata = true;
    let with = walk_items(&opts);
    let file = with.iter().find_map(|item| match item {
        WalkItem::Entry(entry) if entry.file_name == "visible.txt" => Some(entry),
        _ => None,
    });
    let meta = file
        .and_then(|entry| entry.metadata.as_ref())
        .expect("dua should emit metadata for visible.txt");
    assert_eq!(meta.size, 2, "visible.txt is two bytes");
    assert!(
        meta.modified.is_some(),
        "dua-core 3.0 always reports mtime: {meta:?}"
    );
}
