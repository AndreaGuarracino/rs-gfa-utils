use fnv::{FnvHashMap, FnvHashSet};
use log::{debug, info, warn};
use std::path::PathBuf;
use structopt::StructOpt;

use gfa::gfa::GFA;

use crate::variants;

/// Output a VCF for the given GFA, using the graph's ultrabubbles to
/// identify areas of variation. (experimental!)
#[derive(StructOpt, Debug)]
pub struct GFA2VCFArgs {
    /// Load ultrabubbles from a file instead of calculating them.
    #[structopt(
        name = "ultrabubbles file",
        long = "ultrabubbles",
        short = "ub"
    )]
    ultrabubbles_file: Option<PathBuf>,
    /// Don't compare two paths if their start and end orientations
    /// don't match each other
    #[structopt(name = "ignore inverted paths", long = "no-inv")]
    ignore_inverted_paths: bool,
}

pub fn gfa2vcf<P: AsRef<std::path::Path>>(
    gfa_path: P,
    gfa: &GFA<usize, ()>,
    args: &GFA2VCFArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let segment_map: FnvHashMap<usize, &[u8]> = gfa
        .segments
        .iter()
        .map(|seg| (seg.name, seg.sequence.as_ref()))
        .collect();

    let all_paths = variants::gfa_paths_with_offsets(&gfa, &segment_map);

    let ultrabubbles = if let Some(path) = &args.ultrabubbles_file {
        let ub = crate::ultrabubbles::load_ultrabubbles(path)?;
        ub
    } else {
        crate::ultrabubbles::gfa_ultrabubbles(&gfa)
    };

    let ultrabubble_nodes = ultrabubbles
        .iter()
        .flat_map(|&(a, b)| {
            use std::iter::once;
            once(a).chain(once(b))
        })
        .collect::<FnvHashSet<_>>();

    let path_indices =
        variants::bubble_path_indices(&all_paths, &ultrabubble_nodes);

    let mut all_vcf_records = Vec::new();

    let var_config = variants::VariantConfig {
        ignore_inverted_paths: args.ignore_inverted_paths,
    };

    for &(from, to) in ultrabubbles.iter() {
        let vars = variants::detect_variants_in_sub_paths(
            &var_config,
            &segment_map,
            &all_paths,
            &path_indices,
            from,
            to,
        );

        let vcf_records = variants::variant_vcf_record(&vars);
        all_vcf_records.extend(vcf_records);

        /*
        let from_indices = path_indices.get(&from).unwrap();
        let to_indices = path_indices.get(&to).unwrap();

        let sub_paths: FnvHashMap<
            &BStr,
            &[(usize, usize, Orientation)],
        > = all_paths
            .iter()
            .filter_map(|(path_name, path)| {
                let from_ix = *from_indices.get(path_name)?;
                let to_ix = *to_indices.get(path_name)?;
                let from = from_ix.min(to_ix);
                let to = from_ix.max(to_ix);
                let sub_path = &path[from..=to];
                Some((path_name.as_bstr(), sub_path))
            })
            .collect();
        */
    }

    all_vcf_records.sort_by(|v0, v1| v0.vcf_cmp(v1));

    let vcf_header = variants::vcf::VCFHeader::new(gfa_path);

    println!("{}", vcf_header);

    for vcf in all_vcf_records {
        println!("{}", vcf);
    }

    Ok(())
}
