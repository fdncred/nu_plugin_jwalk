use nu_plugin::{serve_plugin, EvaluatedCall, LabeledError, MsgPackSerializer, Plugin};
use nu_protocol::{Category, PluginExample, PluginSignature, Span, Spanned, SyntaxShape, Value};

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
        let param: Option<Spanned<String>> = call.opt(0)?;

        let ret_val = jwalk_do_something(param);

        ret_val
    }
}

fn main() {
    serve_plugin(&mut Implementation::new(), MsgPackSerializer);
}

pub fn jwalk_do_something(param: Option<Spanned<String>>) -> Result<Value, LabeledError> {
    let (a_val, a_span) = match param {
        Some(p) => (format!("Param {}", p.item), p.span),
        None => (format!("No parameter passed"), Span::unknown()),
    };
    Ok(Value::string(a_val, a_span))
}
