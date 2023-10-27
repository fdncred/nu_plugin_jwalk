mod jwalk;
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
            .required("path", SyntaxShape::String, "path to jwalk input file")
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
        input: &Value,
    ) -> Result<Value, LabeledError> {
        assert_eq!(name, "jwalk");
        let param: Option<Spanned<String>> = call.opt(0)?;
        let span = input.span();

        let ret_val = match input {
            Value::String { val, .. } => crate::jwalk::jwalk_do_something(param, val, span)?,
            v => {
                return Err(LabeledError {
                    label: "Expected something from pipeline".into(),
                    msg: format!("requires some input, got {}", v.get_type()),
                    span: Some(call.head),
                });
            }
        };

        Ok(ret_val)
    }
}

fn main() {
    serve_plugin(&mut Implementation::new(), MsgPackSerializer);
}
