use saboten::{
    biedgedgraph::*,
    cactusgraph,
    cactusgraph::{BridgeForest, CactusGraph, CactusTree},
};

use fnv::FnvHashSet;
use gfa::gfa::GFA;

pub fn gfa_ultrabubbles(gfa: &GFA<usize, ()>) -> FnvHashSet<(u64, u64)> {
    let be_graph = BiedgedGraph::from_gfa(gfa);
    let orig_graph = be_graph.clone();

    let cactus_graph = CactusGraph::from_biedged_graph(&orig_graph);

    let cactus_tree = CactusTree::from_cactus_graph(&cactus_graph);

    let bridge_forest = BridgeForest::from_cactus_graph(&cactus_graph);

    let ultrabubbles =
        cactusgraph::find_ultrabubbles_par(&cactus_tree, &bridge_forest);

    let ultrabubbles = cactusgraph::inverse_map_ultrabubbles(ultrabubbles);

    ultrabubbles.into_iter().map(|(x_y, _cont)| x_y).collect()
}
