
use syntax::codemap::{Span, DUMMY_SP};


/// Represents a parsed store definition in template contents
#[derive(Clone, Debug)]
pub struct Store {
    name: String,
    span: Span,
    nodes: Vec<StoreNode>
}

impl Store {
    pub fn with_nodes(span: Span, name: &str, nodes: Vec<StoreNode>) -> Store {
        Store {
            name: name.to_owned(),
            span: span,
            nodes: nodes
        }
    }

    pub fn empty(span: Span, name: &str) -> Store {
        Store {
            name: name.to_owned(),
            span: span,
            nodes: vec![]
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub enum StoreNode {
    // TODO: Define nodes
}

mod parse {
}

pub mod output {
    use super::Store;
    use syntax::ext::base::ExtCtxt;
    use output_actions::{OutputAction, IntoOutputActions, WriteOutputActions, OutputActionWrite};
    use js_write::{WriteJsFunctions, JsWriteFunctions, WriteJs};

    impl IntoOutputActions for Store {
        fn into_output_actions(&self) -> Vec<OutputAction> {
            let name = &self.name;
            let nodes = &self.nodes;

            let output_actions = Vec::new();
            /*
            let output_actions: Vec<OutputAction> = nodes.iter()
                .flat_map(|node| node.into_output_actions(ecx))
                .collect();
            */

            output_actions
        }
    }

    impl WriteOutputActions for Store {
        fn write_output_actions(&self, w: &mut OutputActionWrite) {
            // TODO: Implement
        }
    }

    impl WriteJsFunctions for Store {
        fn write_js_functions(&self, funcs: &mut JsWriteFunctions) {
            let store_name = self.name();
            let func_name = format!("rusttemplate_store_template_{}_{}", "main", &store_name);

            funcs.function(&func_name, &|js| {
                // TODO: Implement store nodes
            });
        }
    }
}
