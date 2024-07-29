use chrono::{DateTime, Local};
use jwalk::{Parallelism, WalkDir, WalkDirGeneric};
use nu_path::expand_path_with;
use nu_plugin::{
    serve_plugin, EngineInterface, EvaluatedCall, MsgPackSerializer, Plugin, PluginCommand,
    SimplePluginCommand,
};
use nu_protocol::{
    record, Category, Example, LabeledError, Signature, Span, Spanned, SyntaxShape, Value,
};
use omnipath::sys_absolute;
use std::{cmp::Ordering, path::Path};

struct JWalkPlugin;

impl Plugin for JWalkPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![Box::new(Implementation)]
    }
}

struct Implementation;

impl SimplePluginCommand for Implementation {
    type Plugin = JWalkPlugin;

    fn name(&self) -> &str {
        "jwalk"
    }

    fn usage(&self) -> &str {
        "View jwalk results of walking the path."
    }
    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self))
            .required("path", SyntaxShape::String, "path to jwalk")
            // .switch("one-column", "run the original jwalk, 1 column", Some('o'))
            .switch(
                "verbose",
                "run in verbose mode with multi-column output",
                Some('v'),
            )
            .switch("sort", "sort by file name", Some('s'))
            .switch(
                "custom",
                "custom hard-coded walker with process_read_dir",
                Some('c'),
            )
            .switch("skip-hidden", "skip hidden files", Some('k'))
            .switch("follow-links", "follow symbolic links", Some('f'))
            .switch(
                "debug",
                "print performance metrics at the end of the table",
                Some('d'),
            )
            .named(
                "min-depth",
                SyntaxShape::Int,
                "minimum depth to search",
                Some('m'),
            )
            .named(
                "max-depth",
                SyntaxShape::Int,
                "maximum depth to search",
                Some('x'),
            )
            .named(
                "threads",
                SyntaxShape::Int,
                "number of rayon threads to use",
                Some('t'),
            )
            .category(Category::Experimental)
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
            description: "Walk the process working directory in debug mode with 2 threads and max depth of 1",
            example: "jwalk --debug --max-depth 1 --threads 2 (pwd)",
            result: None,
        },
            Example {
            description: "Walk the process working directory in debug mode with 2 threads and max depth of 1 using verbose",
            example: "jwalk --debug --verbose --max-depth 1 --threads 2 (pwd)",
            result: None,
        },

        ]
    }

    fn run(
        &self,
        _config: &JWalkPlugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let pattern: Option<Spanned<String>> = call.opt(0)?;
        let sort = call.has_flag("sort")?;
        let custom = call.has_flag("custom")?;
        let skip_hidden = call.has_flag("skip-hidden")?;
        let debug = call.has_flag("debug")?;
        let follow_links = call.has_flag("follow-links")?;
        let min_depth: Option<i64> = call.get_flag("min-depth")?;
        let max_depth: Option<i64> = call.get_flag("max-depth")?;
        let threads: Option<i64> = call.get_flag("threads")?;
        let verbose = call.has_flag("verbose")?;
        let curdir = engine.get_current_dir()?;

        if verbose {
            jwalk_verbose(
                pattern,
                sort,
                custom,
                skip_hidden,
                follow_links,
                min_depth,
                max_depth,
                threads,
                debug,
                curdir,
            )
        } else {
            jwalk_one_column(
                pattern,
                sort,
                custom,
                skip_hidden,
                follow_links,
                min_depth,
                max_depth,
                threads,
                debug,
                curdir,
            )
        }
    }
}

fn main() {
    serve_plugin(&JWalkPlugin, MsgPackSerializer);
}

#[allow(clippy::too_many_arguments)]
pub fn jwalk_verbose(
    param: Option<Spanned<String>>,
    sort: bool,
    custom: bool,
    skip_hidden: bool,
    follow_links: bool,
    min_depth: Option<i64>,
    max_depth: Option<i64>,
    threads: Option<i64>,
    debug: bool,
    curdir: String,
) -> Result<Value, LabeledError> {
    let Some(a_path) = param else {
        return Err(LabeledError::new("Please pass a path parameter to walk")
            .with_label("No pattern provided", Span::unknown()));
    };

    let path_to_walk = expand_path_with(a_path.item, curdir, true);
    let pathbuf = sys_absolute(Path::new(&path_to_walk)).map_err(|err| {
        LabeledError::new(err.to_string()).with_label("Error found using sys_absolute", a_path.span)
    })?;

    let parallelism = match threads {
        Some(thread_count) => Parallelism::RayonNewPool(thread_count as usize),
        None => Parallelism::RayonDefaultPool {
            busy_timeout: std::time::Duration::from_secs(1),
        },
    };
    let minimum_depth = match min_depth {
        Some(m) => m as usize,
        None => 0,
    };
    let maximum_depth = match max_depth {
        Some(m) => m as usize,
        None => usize::MAX,
    };

    let mut entry_list = vec![];
    let start_time = std::time::Instant::now();

    let walk_dir = if custom {
        WalkDirGeneric::<(usize, bool)>::new(pathbuf)
            .process_read_dir(|_depth, _path, read_dir_state, children| {
                // 1. Custom sort
                children.sort_by(|a, b| match (a, b) {
                    (Ok(a), Ok(b)) => a.file_name.cmp(&b.file_name),
                    (Ok(_), Err(_)) => Ordering::Less,
                    (Err(_), Ok(_)) => Ordering::Greater,
                    (Err(_), Err(_)) => Ordering::Equal,
                });
                // 2. Custom filter
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
                // 3. Custom skip
                children.iter_mut().for_each(|dir_entry_result| {
                    if let Ok(dir_entry) = dir_entry_result {
                        if dir_entry.depth == 2 {
                            dir_entry.read_children_path = None;
                        }
                    }
                });
                // 4. Custom state
                *read_dir_state += 1;
                if let Some(Ok(dir_entry)) = children.first_mut() {
                    dir_entry.client_state = true;
                }
                // children.first_mut().map(|dir_entry_result| {
                //     if let Ok(dir_entry) = dir_entry_result {
                //         dir_entry.client_state = true;
                //     }
                // });
            })
            .skip_hidden(skip_hidden)
            .follow_links(follow_links)
            .min_depth(minimum_depth)
            .max_depth(maximum_depth)
            .parallelism(parallelism)
    } else {
        WalkDirGeneric::<(usize, bool)>::new(std::path::Path::new(&path_to_walk))
            .sort(sort)
            .skip_hidden(skip_hidden)
            .follow_links(follow_links)
            .min_depth(minimum_depth)
            .max_depth(maximum_depth)
            .parallelism(parallelism)
    };
    for entry in walk_dir {
        let entry_display = match entry.map_err(|err| {
            LabeledError::new(err.to_string())
                .with_label("Error found with jwalk entry", a_path.span)
        }) {
            Ok(e) => e,
            Err(e) => return Err(e),
        };

        let m = match entry_display.metadata() {
            Ok(e) => {
                let accessed = if let Ok(a) = e.accessed() {
                    Some(a)
                } else {
                    None
                };
                let created = if let Ok(c) = e.created() {
                    Some(c)
                } else {
                    None
                };
                let modified = if let Ok(m) = e.modified() {
                    Some(m)
                } else {
                    None
                };

                Some((accessed, created, modified, e.len(), e.permissions()))
            }
            Err(_e) => None,
        };

        entry_list.push(Value::test_record(
            record! {
                // "path" => Value::test_string(entry_display.path().display().to_string()),
                "depth" => Value::test_int(entry_display.depth as i64),
                "client_state" => Value::test_bool(entry_display.client_state),
                "file_name" => Value::test_string(entry_display.file_name.to_string_lossy().to_string()),
                "full_path" => {
                    let fp = sys_absolute(Path::new(&entry_display.path())).map_err(|err| {
                        LabeledError::new(err.to_string()).with_label("Error found using sys_absolute", a_path.span)
                    })?;

                    Value::test_string(fp.display().to_string())
                },
                "is_dir" => Value::test_bool(entry_display.file_type.is_dir()),
                "is_file" => Value::test_bool(entry_display.file_type.is_file()),
                "is_symlink" => Value::test_bool(entry_display.file_type.is_symlink()),
                // "metadata" => Value::test_string(format!("{:?}", entry_display.metadata())),
                // "read_children_path" => Value::test_string(format!("{:?}", entry_display.read_children_path)),
                "parent_path" => {
                    let fp = sys_absolute(Path::new(&entry_display.parent_path())).map_err(|err| {
                        LabeledError::new(err.to_string()).with_label("Error found using sys_absolute", a_path.span)
                    })?;

                    Value::test_string(fp.display().to_string())
                },
                "path_is_symlink" => Value::test_string(format!("{:?}", entry_display.path_is_symlink())),
                "accessed" => match m {
                    Some((Some(a), _, _, _, _)) => {
                        let dt: DateTime<Local> = a.into();
                        Value::test_date(dt.into())
                    }
                    _ => Value::test_string("".to_string()),
                },
                "created" => match m {
                    Some((_, Some(c), _, _, _)) => {
                        let dt: DateTime<Local> = c.into();
                        Value::test_date(dt.into())
                    }
                    _ => Value::test_string("".to_string()),
                },
                "modified" => match m {
                    Some((_, _, Some(modi), _, _)) => {
                        let dt: DateTime<Local> = modi.into();
                        Value::test_date(dt.into())
                    }
                    _ => Value::test_string("".to_string()),
                },
                "size" => match m {
                    Some((_, _, _, l, _)) => Value::test_int(l as i64),
                    _ => Value::test_int(0),
                },
                "readonly" => match m {
                    Some((_, _, _, _, p)) => Value::test_bool(p.readonly()),
                    _ => Value::test_string("".to_string()),
                },
            }
        ))
    }
    let elapsed = start_time.elapsed();
    if debug {
        entry_list.push(Value::test_record(record! {
            "path" => Value::test_string(format!("sort: {}", sort)),
            "depth" => Value::test_string(format!("skip_hidden: {}", skip_hidden)),
            "client_state" => Value::test_string(format!("follow_links: {}", follow_links)),
            "file_name" => Value::test_string(format!("min_depth: {}", minimum_depth)),
            "is_dir" => Value::test_string(format!("max_depth: {}", maximum_depth)),
            "is_file" => Value::test_string(format!("threads: {}", threads.unwrap_or(0))),
            "is_symlink" => Value::test_string(format!("time: {:?}", elapsed)),
        }))
    }

    Ok(Value::test_list(entry_list))
}

#[allow(clippy::too_many_arguments)]
pub fn jwalk_one_column(
    param: Option<Spanned<String>>,
    sort: bool,
    custom: bool,
    skip_hidden: bool,
    follow_links: bool,
    min_depth: Option<i64>,
    max_depth: Option<i64>,
    threads: Option<i64>,
    debug: bool,
    curdir: String,
) -> Result<Value, LabeledError> {
    if custom {
        return Err(
            LabeledError::new("Please remove the custom flag").with_label(
                "Custom walker only supported with verbose mode",
                Span::unknown(),
            ),
        );
    }

    let Some(a_path) = param else {
        return Err(LabeledError::new("Please pass a path parameter to walk")
            .with_label("No pattern provided", Span::unknown()));
    };

    let path_to_walk = expand_path_with(a_path.item, curdir, true);
    let pathbuf = sys_absolute(Path::new(&path_to_walk)).map_err(|err| {
        LabeledError::new(err.to_string()).with_label("Error found using sys_absolute", a_path.span)
    })?;

    let parallelism = match threads {
        Some(thread_count) => Parallelism::RayonNewPool(thread_count as usize),
        None => Parallelism::RayonDefaultPool {
            busy_timeout: std::time::Duration::from_secs(1),
        },
    };
    let minimum_depth = match min_depth {
        Some(m) => m as usize,
        None => 0,
    };
    let maximum_depth = match max_depth {
        Some(m) => m as usize,
        None => usize::MAX,
    };

    let mut entry_list = vec![];
    let start_time = std::time::Instant::now();

    for entry in WalkDir::new(pathbuf)
        .sort(sort)
        .skip_hidden(skip_hidden)
        .follow_links(follow_links)
        .min_depth(minimum_depth)
        .max_depth(maximum_depth)
        .parallelism(parallelism)
    {
        let entry_display = match entry.map_err(|err| {
            LabeledError::new(err.to_string())
                .with_label("Error found with jwalk entry", a_path.span)
        }) {
            Ok(e) => e,
            Err(e) => return Err(e),
        };
        entry_list.push(Value::test_string(
            sys_absolute(&entry_display.path())
                .map_err(|err| {
                    LabeledError::new(err.to_string())
                        .with_label("Error found using sys_absolute", a_path.span)
                })?
                .to_string_lossy()
                .to_string(),
        ));
    }
    let elapsed = start_time.elapsed();
    if debug {
        // for debugging put the perf metrics in the last rows
        entry_list.push(Value::test_string(format!("Running with these options:\n  sort: {}\n  skip_hidden: {}\n  follow_links: {}\n  min_depth: {}\n  max_depth: {}\n  threads: {:?}\nTime: {:?}", sort, skip_hidden, follow_links, minimum_depth, maximum_depth, threads, elapsed)));
    }

    Ok(Value::test_list(entry_list))
}
