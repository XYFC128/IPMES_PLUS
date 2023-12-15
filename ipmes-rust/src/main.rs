use clap::{arg, Parser};
use log::info;

use ipmes_rust::pattern::Pattern;
use ipmes_rust::process_layers::{JoinLayer, ParseLayer, CompositionLayer, UniquenessLayer};
use ipmes_rust::sub_pattern::decompose;

/// IPMES implemented in rust
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The path prefix of pattern's files, e.g. ../data/universal_patterns/SP12.json
    pattern_file: String,

    /// The path to the preprocessed data graph
    data_graph: String,

    /// Window size (sec)
    #[arg(short, long, default_value_t = 1800)]
    window_size: u64,
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    info!("Command line arguments: {:?}", args);
    let window_size = args.window_size * 1000;

    let pattern = Pattern::parse(&args.pattern_file).expect("Failed to parse pattern");
    info!("Pattern Edges: {:#?}", pattern.events);

    let decomposition = decompose(&pattern);
    info!("Decomposition results: {:#?}", decomposition);

    let mut csv = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(args.data_graph).expect("Failed to open input graph");
    let parse_layer = ParseLayer::new(&mut csv);
    let composition_layer =
        CompositionLayer::new(parse_layer, &decomposition, pattern.use_regex, window_size).unwrap();
    let join_layer = JoinLayer::new(composition_layer, &pattern, &decomposition, window_size);
    let uniqueness_layer = UniquenessLayer::new(join_layer, window_size);

    let mut num_result = 0u32;
    for pattern_match in uniqueness_layer {
        info!("Pattern Match: {}", pattern_match);
        num_result += 1;
    }
    println!("Total number of matches: {num_result}");
    info!("Finished");
}