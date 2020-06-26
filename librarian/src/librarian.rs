use crossbeam_utils::sync::ShardedLock;
use petgraph::graphmap::DiGraphMap as Graph;
use uuid::Uuid;

#[derive(Default)]
pub(crate) struct Librarian {
    pub(crate) graph: ShardedLock<Graph<Uuid, ()>>,
}
