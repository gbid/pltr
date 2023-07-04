use std::io;
use std::time::Instant;
use pltr::algorithm;
use pltr::dataset_parsing;

/// Starts a macro benchmark of pltr using the scheduling dataset from
/// `Exact and heuristic algorithms for scheduling jobs
/// with time windows on unrelated parallel machines`
/// (Giorgi Tadumadze, Simon Emde, Heiko Diefenbach)
fn main() -> io::Result<()> {
    let start = Instant::now();
    let instances = dataset_parsing::parse_csv_to_instance("datasets/exact_and_heuristic_scheduling/instances.csv").unwrap();
    let _ = instances.iter().map(algorithm::pltr).collect::<Vec<_>>();
    let duration = start.elapsed();
    println!("Time taken: {:?}", duration);
    Ok(())
}
