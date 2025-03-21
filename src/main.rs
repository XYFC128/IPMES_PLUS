use std::error::Error;

use clap::{arg, Parser};
use log::{info, warn};

use cpu_time::ProcessTime;

use ipmes_rust::pattern::{decompose, Pattern};
use ipmes_rust::process_layers::{
    CompositionLayer, JoinLayer, ParseLayer, UniquenessLayer,
};

/// IPMES implemented in rust
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The path prefix of pattern's files, e.g. data/universal_patterns/SP12.json
    pattern_file: String,

    /// The path to the preprocessed data graph
    data_graph: String,

    /// Window size (sec)
    #[arg(short, long, default_value_t = 1800)]
    window_size: u64,

    /// Enable silent mode will not print individual pattern matches.
    #[arg(short, long, default_value_t = false)]
    print_instances: bool,
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    info!("Command line arguments: {:?}", args);
    let window_size = args.window_size * 1000;

    let mut pattern = Pattern::parse(&args.pattern_file).expect("Failed to parse pattern");
    pattern.optimize();
    info!("Pattern Edges: {:#?}", pattern.events);

    let decomposition = decompose(&pattern);
    info!("Decomposition results: {:#?}", decomposition);

    let csv_reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(args.data_graph)
        .expect("Failed to open input graph");
    let parse_layer = ParseLayer::new(csv_reader);
    let composition_layer =
        CompositionLayer::new(parse_layer, &decomposition, window_size, pattern.use_regex).unwrap();
    let join_layer = JoinLayer::new(composition_layer, &pattern, &decomposition, window_size);
    let uniqueness_layer = UniquenessLayer::new(join_layer, window_size);

    let start_time = ProcessTime::now();

    let mut num_result = 0u32;
    for pattern_match in uniqueness_layer {
        if args.print_instances {
            println!("Pattern Match: {}", pattern_match);
        }
        num_result += 1;
    }
    
    println!("Total number of matches: {num_result}");

    println!(
        "CPU time elapsed: {:?} secs",
        start_time.elapsed().as_secs_f64()
    );

    if let Err(err) = print_peak_memory() {
        warn!(
            "Encounter an error when tring to get peak memory usage: {}",
            err
        )
    }

    info!("Finished");
}

fn print_peak_memory() -> Result<(), Box<dyn Error>> {
    #[cfg(target_family = "windows")]
    {
        use windows::System::Diagnostics::ProcessDiagnosticInfo;
        let info = ProcessDiagnosticInfo::GetForCurrentProcess()?;
        let mem_usage = info.MemoryUsage()?;
        let mem_report = mem_usage.GetReport()?;
        let max_rss = mem_report.PeakWorkingSetSizeInBytes()?;
        println!("Peak memory usage: {} kB", max_rss / 1024u64);
    }

    #[cfg(target_family = "unix")]
    {
        use nix::sys::resource::{getrusage, UsageWho};
        let usage = getrusage(UsageWho::RUSAGE_SELF)?;
        println!("Peak memory usage: {} kB", usage.max_rss());
    }

    Ok(())
}
