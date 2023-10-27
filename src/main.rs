use jwalk::{Parallelism, WalkDir};
use nu_plugin::{serve_plugin, EvaluatedCall, LabeledError, MsgPackSerializer, Plugin};
use nu_protocol::{Category, PluginExample, PluginSignature, Spanned, SyntaxShape, Value};

struct Implementation;

impl Implementation {
    fn new() -> Self {
        Self {}
    }
}

impl Plugin for Implementation {
    fn signature(&self) -> Vec<PluginSignature> {
        vec![PluginSignature::build("jwalk")
            .usage("View jwalk results")
            .required("path", SyntaxShape::String, "path to jwalk")
            .switch("sort", "sort by file name", Some('s'))
            .switch("custom", "custom walker with process_read_dir", Some('c'))
            .switch("skip-hidden", "skip hidden files", Some('k'))
            .switch("follow-links", "follow symbolic links", Some('f'))
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
            .plugin_examples(vec![PluginExample {
                description: "This is the example descripion".into(),
                example: "some pipeline involving jwalk".into(),
                result: None,
            }])]
    }

    fn run(
        &mut self,
        name: &str,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        assert_eq!(name, "jwalk");
        let pattern: Option<Spanned<String>> = call.opt(0)?;
        let sort = call.has_flag("sort");
        let custom = call.has_flag("custom");
        let skip_hidden = call.has_flag("skip-hidden");
        let follow_links = call.has_flag("follow-links");
        let min_depth: Option<i64> = call.get_flag("min-depth")?;
        let max_depth: Option<i64> = call.get_flag("max-depth")?;
        let threads: Option<i64> = call.get_flag("threads")?;

        // if custom {
        //     jwalk_custom(pattern, sort, custom, skip_hidden, follow_links, min_depth, max_depth, threads)
        // } else {
        jwalk_minimal(
            pattern,
            sort,
            custom,
            skip_hidden,
            follow_links,
            min_depth,
            max_depth,
            threads,
        )
        // }
    }
}

fn main() {
    serve_plugin(&mut Implementation::new(), MsgPackSerializer);
}

pub fn jwalk_minimal(
    param: Option<Spanned<String>>,
    sort: bool,
    _custom: bool,
    skip_hidden: bool,
    follow_links: bool,
    min_depth: Option<i64>,
    max_depth: Option<i64>,
    threads: Option<i64>,
) -> Result<Value, LabeledError> {
    let Some(a_val) = param else {
        return Err(LabeledError {
            label: "No pattern provided".into(),
            msg: "Please pass a parameter to walk".into(),
            span: None,
        })
    };

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

    for entry in WalkDir::new(a_val.item.clone())
        .sort(sort)
        .skip_hidden(skip_hidden)
        .follow_links(follow_links)
        .min_depth(minimum_depth)
        .max_depth(maximum_depth)
        .parallelism(parallelism)
    {
        let entry_display = match entry.map_err(|err| {
            return Err(LabeledError {
                label: "Error found with jwalk entry".into(),
                msg: err.to_string(),
                span: Some(a_val.span),
            });
        }) {
            Ok(e) => e,
            Err(e) => return e,
        };
        entry_list.push(Value::test_string(
            entry_display.path().display().to_string(),
        ));
    }
    let elapsed = start_time.elapsed();
    // for debugging put the perf metrics in the last rows
    entry_list.push(Value::test_string(format!("Running with these options:\n  sort: {}\n  skip_hidden: {}\n  follow_links: {}\n  min_depth: {}\n  max_depth: {}\n  threads: {:?}\n", sort, skip_hidden, follow_links, minimum_depth, maximum_depth, threads)));
    entry_list.push(Value::test_string(format!("Time: {:?}", elapsed)));

    Ok(Value::test_list(entry_list))
}
