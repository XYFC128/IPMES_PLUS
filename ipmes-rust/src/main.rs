use clap::{arg, Parser, ValueEnum};
use log::info;

use ipmes_rust::pattern::darpa::DarpaPatternParser;
use ipmes_rust::pattern::parser::{PatternParser, PatternParsingError};
use ipmes_rust::pattern::spade::SpadePatternParser;
use ipmes_rust::pattern::Pattern;
use ipmes_rust::process_layers::{JoinLayer, OrdMatchLayer, ParseLayer};
use ipmes_rust::sub_pattern::decompose;

/// IPMES implemented in rust
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The path prefix of pattern's files, e.g. ./data/patterns/TTP11
    pattern_prefix: String,

    /// The path to the preprocessed data graph
    data_graph: String,

    /// Explicitly use regex matching. Default will automatically depend on the pattern prefix name
    #[arg(short, long)]
    regex: bool,

    /// Pattern signature format.
    #[arg(short = 'F', long, value_enum, default_value_t = PatternFormat::Spade)]
    pattern_format: PatternFormat,

    /// Window size (sec)
    #[arg(short, long, default_value_t = 1800)]
    window_size: u64,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum PatternFormat {
    Spade,
    Darpa,
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    info!("Command line arguments: {:?}", args);
    let window_size = args.window_size * 1000;

    let pattern = parse_pattern(&args).expect("Failed to parse pattern");
    info!("Pattern Edges: {:#?}", pattern.edges);

    let decomposition = decompose(&pattern);
    info!("Decomposition results: {:#?}", decomposition);

    let mut csv = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(args.data_graph).expect("Failed to open input graph");
    let parse_layer = ParseLayer::new(&mut csv);
    let ord_match_layer =
        OrdMatchLayer::new(parse_layer, &decomposition, args.regex, window_size).unwrap();
    let join_layer = JoinLayer::new(ord_match_layer, &pattern, &decomposition, window_size);

    let mut num_result = 0u32;
    for result in join_layer {
        for pattern_match in result {
            info!("Pattern Match: {}", pattern_match);
        }
        num_result += 1;
    }
    info!("Total number of matches: {num_result}");
    info!("Finished");
}

fn parse_pattern(args: &Args) -> Result<Pattern, PatternParsingError> {
    let (node_file, edge_file, orels_file) = get_input_files(&args.pattern_prefix);

    match args.pattern_format {
        PatternFormat::Spade => {
            let parser = SpadePatternParser;
            parser.parse(&node_file, &edge_file, &orels_file)
        }
        PatternFormat::Darpa => {
            let parser = DarpaPatternParser;
            parser.parse(&node_file, &edge_file, &orels_file)
        }
    }
}

fn get_input_files(input_prefix: &str) -> (String, String, String) {
    let node_file = format!("{}_node.json", input_prefix);
    let edge_file = format!("{}_edge.json", input_prefix);
    let orels_file = if let Some(prefix) = input_prefix.strip_suffix("_regex") {
        format!("{}_oRels.json", prefix)
    } else {
        format!("{}_oRels.json", input_prefix)
    };

    (node_file, edge_file, orels_file)
}

#[cfg(test)]
mod tests {
    use crate::get_input_files;

    #[test]
    fn test_input_file_name_parsing() {
        assert_eq!(
            get_input_files("TTP11"),
            (
                "TTP11_node.json".to_string(),
                "TTP11_edge.json".to_string(),
                "TTP11_oRels.json".to_string()
            )
        );

        assert_eq!(
            get_input_files("TTP11_regex"),
            (
                "TTP11_regex_node.json".to_string(),
                "TTP11_regex_edge.json".to_string(),
                "TTP11_oRels.json".to_string()
            )
        );
    }
}
