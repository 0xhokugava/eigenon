use clap::Parser;
use eigenon::cli::{Cli, Commands, OpenQasmVersion};

#[test]
fn export_openqasm_defaults_to_v2() {
    let cli = Cli::try_parse_from(["eigenon", "export-openqasm", "--qubits", "2"]).unwrap();

    match cli.command {
        Commands::ExportOpenqasm { qasm_version, .. } => {
            assert!(matches!(qasm_version, OpenQasmVersion::V2));
        }
        _ => panic!("Expected export-openqasm command"),
    }
}

#[test]
fn export_openqasm_accepts_v3() {
    let cli = Cli::try_parse_from([
        "eigenon",
        "export-openqasm",
        "--qubits",
        "2",
        "--qasm-version",
        "3",
    ])
    .unwrap();

    match cli.command {
        Commands::ExportOpenqasm { qasm_version, .. } => {
            assert!(matches!(qasm_version, OpenQasmVersion::V3));
        }
        _ => panic!("Expected export-openqasm command"),
    }
}
