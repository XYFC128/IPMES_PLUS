use log::info;
use clap::{Parser, ValueEnum};

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
    window_size: u32,
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
}

/*
for line in input {
    let parsed = parse(line)

    let sorted = xxx.sort(parsed) Iter<[1, 2, 3]>
    let subpatterns = match_sub_pattern(sorted) [(key, [val])]
    let patterns = join(pool3, sub_pattern) [pattern]
}
for result in input.parse().reorder().dispatch().sub_pattern_match().join()
 */