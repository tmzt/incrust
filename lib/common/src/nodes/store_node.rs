
use syntax::codemap::{Span, DUMMY_SP};

use super::content_node::ContentNode;


/// Represents a parsed store definition in template contents
#[derive(Clone, Debug)]
pub struct Store {
    name: String,
    span: Span,
    nodes: Vec<ContentNode>
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
}
