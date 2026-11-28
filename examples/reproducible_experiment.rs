use eigenon::experiments::harness::{ExperimentDefinition, run_canonical_experiment};
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let definition_path = arguments
        .next()
        .map(PathBuf::from)
        .ok_or("usage: reproducible_experiment <definition.json> [result.json]")?;
    let output_path = arguments.next().map(PathBuf::from);

    if arguments.next().is_some() {
        return Err("usage: reproducible_experiment <definition.json> [result.json]".into());
    }

    let definition = ExperimentDefinition::read_json(definition_path)?;
    let record = run_canonical_experiment(&definition)?;

    if let Some(output_path) = output_path {
        record.write_json(output_path)?;
    } else {
        println!("{}", record.to_json_pretty()?);
    }

    if !record.all_comparisons_passed() {
        return Err("one or more reference comparisons failed".into());
    }

    Ok(())
}
