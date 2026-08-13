use crate::{
    dua_backend, jwalk_backend,
    options::{Engine, WalkOptions, WalkOrder},
    zlob_backend,
};
use nu_path::expand_path_with;
use nu_plugin::{EngineInterface, EvaluatedCall, Plugin, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, Span, Spanned, SyntaxShape,
};
use omnipath::sys_absolute;
use std::{ffi::OsString, path::Path, sync::Arc};

pub struct JWalkPlugin;

impl Plugin for JWalkPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![Box::new(Implementation)]
    }
}

struct Implementation;

impl PluginCommand for Implementation {
    type Plugin = JWalkPlugin;

    fn name(&self) -> &str {
        "jwalk"
    }

    fn description(&self) -> &str {
        "Walk a path with jwalk (default), dua-core, or zlob."
    }

    fn extra_description(&self) -> &str {
        "jwalk is the default engine and can list paths without an extra stat. zlob can also skip that stat unless --metadata is set. dua always reads metadata while walking. --custom requires --engine jwalk. --follow-links works with jwalk and zlob."
    }

    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self))
            .required("path", SyntaxShape::String, "path to walk")
            .named(
                "engine",
                SyntaxShape::String,
                "walk engine: jwalk (default), dua, or zlob",
                Some('e'),
            )
            .switch(
                "verbose",
                "multi-column output without extra metadata syscalls",
                Some('v'),
            )
            .switch(
                "metadata",
                "include size, times, and readonly (implies verbose records)",
                None,
            )
            .switch("sort", "sort by file name", Some('s'))
            .switch(
                "custom",
                "custom hard-coded walker with process_read_dir (jwalk only)",
                Some('c'),
            )
            .switch("skip-hidden", "skip hidden files", Some('k'))
            .switch(
                "follow-links",
                "follow symbolic links (jwalk and zlob)",
                Some('f'),
            )
            .named(
                "skip-dir",
                SyntaxShape::List(Box::new(SyntaxShape::String)),
                "directory names to yield but not descend into",
                None,
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
                "worker threads (0 = serial)",
                Some('t'),
            )
            .named(
                "order",
                SyntaxShape::String,
                "dua yield order: completion (default) or parent-first",
                None,
            )
            .switch(
                "count",
                "return only the number of entries (no per-path plugin values)",
                None,
            )
            .switch(
                "debug",
                "show internal settings and performance metrics",
                Some('d'),
            )
            .category(Category::Experimental)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                description: "Fastest path listing: skip hidden names and heavy directories",
                example: "jwalk --skip-hidden --skip-dir [target node_modules .git] ~",
                result: None,
            },
            Example {
                description: "Same walk using the zlob engine",
                example: "jwalk --engine zlob --skip-hidden --skip-dir [target node_modules .git] ~",
                result: None,
            },
            Example {
                description: "Same walk using the dua engine for comparison",
                example: "jwalk --engine dua --skip-hidden --skip-dir [target node_modules .git] ~",
                result: None,
            },
            Example {
                description: "Fastest comparable count (no per-path plugin messages)",
                example: "jwalk --count --skip-hidden --skip-dir [target node_modules .git] ~",
                result: None,
            },
            Example {
                description: "Shallow listing (serial; extra threads do not help at depth < 2)",
                example: "jwalk --skip-hidden --max-depth 1 --threads 0 (pwd)",
                result: None,
            },
            Example {
                description: "Verbose columns without extra stat syscalls on jwalk",
                example: "jwalk --verbose --skip-hidden --skip-dir [target] (pwd)",
                result: None,
            },
            Example {
                description: "Verbose columns including size and times",
                example: "jwalk --verbose --metadata --skip-hidden (pwd)",
                result: None,
            },
        ]
    }

    fn run(
        &self,
        _plugin: &JWalkPlugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let options = parse_options(call, engine)?;
        options.validate()?;
        match options.engine {
            Engine::Dua => dua_backend::run(options, engine),
            Engine::Jwalk => jwalk_backend::run(options, engine),
            Engine::Zlob => zlob_backend::run(options, engine),
        }
    }
}

fn parse_options(
    call: &EvaluatedCall,
    engine: &EngineInterface,
) -> Result<WalkOptions, LabeledError> {
    let pattern: Option<Spanned<String>> = call.opt(0)?;
    let Some(a_path) = pattern else {
        return Err(LabeledError::new("Please pass a path parameter to walk")
            .with_label("No pattern provided", Span::unknown()));
    };
    let span = a_path.span;
    let curdir = engine.get_current_dir()?;
    let path_to_walk = expand_path_with(a_path.item, curdir, true);
    let path = sys_absolute(Path::new(&path_to_walk)).map_err(|err| {
        LabeledError::new(err.to_string()).with_label("Error found using sys_absolute", span)
    })?;

    let engine_flag: Option<Spanned<String>> = call.get_flag("engine")?;
    let walk_engine = Engine::parse(engine_flag.as_ref().map(|s| s.item.as_str()), span)?;
    let order_flag: Option<Spanned<String>> = call.get_flag("order")?;
    let order = WalkOrder::parse(order_flag.as_ref().map(|s| s.item.as_str()), span)?;

    let skip_dir_flag: Option<Vec<String>> = call.get_flag("skip-dir")?;
    let skip_dirs = skip_dir_flag
        .unwrap_or_default()
        .into_iter()
        .map(OsString::from)
        .collect::<Vec<_>>();

    Ok(WalkOptions {
        engine: walk_engine,
        path,
        span,
        sort: call.has_flag("sort")?,
        custom: call.has_flag("custom")?,
        skip_hidden: call.has_flag("skip-hidden")?,
        follow_links: call.has_flag("follow-links")?,
        min_depth: optional_usize(call.get_flag("min-depth")?, 0, span)?,
        max_depth: optional_usize(call.get_flag("max-depth")?, usize::MAX, span)?,
        threads: optional_threads(call.get_flag("threads")?, span)?,
        skip_dirs: Arc::from(skip_dirs),
        verbose: call.has_flag("verbose")?,
        metadata: call.has_flag("metadata")?,
        count: call.has_flag("count")?,
        debug: call.has_flag("debug")?,
        order,
    })
}

fn optional_usize(value: Option<i64>, default: usize, span: Span) -> Result<usize, LabeledError> {
    match value {
        None => Ok(default),
        Some(n) if n >= 0 => Ok(n as usize),
        Some(n) => Err(LabeledError::new("invalid depth")
            .with_label(format!("expected a non-negative integer, got {n}"), span)),
    }
}

fn optional_threads(value: Option<i64>, span: Span) -> Result<Option<usize>, LabeledError> {
    match value {
        None => Ok(None),
        Some(n) if n >= 0 => Ok(Some(n as usize)),
        Some(n) => Err(LabeledError::new("invalid thread count")
            .with_label(format!("expected a non-negative integer, got {n}"), span)),
    }
}
