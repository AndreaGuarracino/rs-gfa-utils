use saboten::{
    biedgedgraph::*,
    cactusgraph,
    cactusgraph::{BridgeForest, CactusGraph, CactusTree},
};

use fnv::FnvHashSet;
use gfa::gfa::GFA;
use std::{fs::File, io::BufReader, path::Path};

use bstr::{io::BufReadExt, BStr, BString, ByteSlice, ByteVec};

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

static LINE_ERROR: &str = "Ultrabubble record was missing fields";

pub fn load_ultrabubbles<P: AsRef<Path>>(
    path: P,
) -> Result<FnvHashSet<(u64, u64)>, Box<dyn std::error::Error>> {
    let file = File::open(path.as_ref())?;
    let reader = BufReader::new(file);
    let lines = reader.byte_lines();

    let mut ultrabubbles = FnvHashSet::default();

    for line in lines {
        let line = line?;
        let mut fields = line.split_str("\t");
        let start = fields.next().ok_or(LINE_ERROR)?.to_str()?;
        let start = start.parse::<u64>()?;

        let end = fields.next().ok_or(LINE_ERROR)?.to_str()?;
        let end = end.parse::<u64>()?;

        ultrabubbles.insert((start, end));
    }

    Ok(ultrabubbles)
}
