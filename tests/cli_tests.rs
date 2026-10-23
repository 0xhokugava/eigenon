use clap::Parser;
use eigenon::cli::{Cli, Commands, OpenQasmVersion};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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

#[test]
fn export_openqasm_accepts_parameterized_gates() {
    let output = Command::new(env!("CARGO_BIN_EXE_eigenon"))
        .args([
            "export-openqasm",
            "--qubits",
            "1",
            "--gate",
            "rx:0.5,0",
            "--gate",
            "u:0.1,0.2,0.3,0",
            "--qasm-version",
            "3",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("rx(0.5) q[0];"));
    assert!(stdout.contains("U(0.1, 0.2, 0.3) q[0];"));
}

#[test]
fn import_openqasm_accepts_file_path() {
    let cli =
        Cli::try_parse_from(["eigenon", "import-openqasm", "--file", "circuit.qasm"]).unwrap();

    match cli.command {
        Commands::ImportOpenqasm { file } => {
            assert_eq!(file, PathBuf::from("circuit.qasm"));
        }
        _ => panic!("Expected import-openqasm command"),
    }
}

#[test]
fn import_openqasm_executes_imported_circuit() {
    let path = temporary_qasm_path("bell");
    fs::write(
        &path,
        "OPENQASM 2.0;\n\
         include \"qelib1.inc\";\n\
         qreg q[2];\n\
         h q[0];\n\
         cx q[0], q[1];\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_eigenon"))
        .args(["import-openqasm", "--file"])
        .arg(&path)
        .output()
        .unwrap();
    let _ = fs::remove_file(&path);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Imported OpenQASM 2.0:"));
    assert!(stdout.contains("Qubits: 2, Classical bits: 0, Operations: 2"));
    assert!(stdout.contains("|00>"));
    assert!(stdout.contains("|11>"));
}

#[test]
fn import_openqasm_reports_source_errors() {
    let path = temporary_qasm_path("invalid");
    fs::write(&path, "OPENQASM 3.0;\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_eigenon"))
        .args(["import-openqasm", "--file"])
        .arg(&path)
        .output()
        .unwrap();
    let _ = fs::remove_file(&path);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("OpenQASM 3 import is not supported yet at 1:10"));
    assert!(stderr.contains("current importer supports only the OpenQASM 2.0 subset"));
}

#[test]
fn import_openqasm_identifies_headerless_openqasm3() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/openqasm3/inverseqft1.qasm");

    let output = Command::new(env!("CARGO_BIN_EXE_eigenon"))
        .args(["import-openqasm", "--file"])
        .arg(path)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("OpenQASM 3 import is not supported yet at 2:1"));
    assert!(stderr.contains("current importer supports only the OpenQASM 2.0 subset"));
}

#[test]
fn import_openqasm_reports_file_read_errors() {
    let path = temporary_qasm_path("missing");

    let output = Command::new(env!("CARGO_BIN_EXE_eigenon"))
        .args(["import-openqasm", "--file"])
        .arg(&path)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Failed to read OpenQASM file"));
    assert!(stderr.contains(path.to_string_lossy().as_ref()));
}

fn temporary_qasm_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("eigenon_cli_{name}_{}.qasm", std::process::id()))
}
