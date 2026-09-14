use clap::Parser;
use eigenon::circuit::core::Circuit;
use eigenon::cli::{Cli, Commands, DemoCommand, OpenQasmVersion, VerifyCommand};
use eigenon::engine::utils::to_dirac;
use eigenon::export::openqasm::{export_openqasm2, export_openqasm3};
use eigenon::{algorithms, experiments};

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    if !matches!(cli.command, Commands::ExportOpenqasm { .. }) {
        println!("\n🐈  Eigenon 🐈‍⬛\n");
    }
    match cli.command {
        Commands::Run { qubits, gates } => {
            if qubits == 0 {
                return Err("Circuit must contain at least one qubit".to_string());
            }
            for gate in &gates {
                gate.validate(qubits)?;
            }
            let mut circuit = Circuit::new(qubits);
            for gate in &gates {
                gate.apply(&mut circuit);
            }
            let state = circuit.run();
            println!("Qubits: {}, Gates: {:?}", qubits, gates);
            println!("State: {}", to_dirac(&state));
        }
        Commands::ExportOpenqasm {
            qubits,
            gates,
            qasm_version,
        } => {
            if qubits == 0 {
                return Err("Circuit must contain at least one qubit".to_string());
            }

            for gate in &gates {
                gate.validate(qubits)?;
            }

            let mut circuit = Circuit::new(qubits);

            for gate in &gates {
                gate.apply(&mut circuit);
            }

            let qasm = match qasm_version {
                OpenQasmVersion::V2 => export_openqasm2(&circuit),
                OpenQasmVersion::V3 => export_openqasm3(&circuit),
            }
            .map_err(|error| error.to_string())?;

            println!("{qasm}");
        }
        Commands::Demo { command } => match command {
            DemoCommand::All => experiments::run_all_demos(),
            DemoCommand::SingleQubit => experiments::circuit_demos::single_qubit::run(),
            DemoCommand::TensorProduct => experiments::foundations::tensor_product_example::run(),
            DemoCommand::Cnot => experiments::circuit_demos::cnot_verification::run(),
            DemoCommand::Entanglement => experiments::circuit_demos::entanglement::run(),
            DemoCommand::Deutsch => algorithms::deutsch::run_deutsch_demo(),
            DemoCommand::DeutschJozsa => algorithms::deutsch_jozsa::run_deutsch_jozsa_demo(),
            DemoCommand::Grover => algorithms::grover_search::run_grover_demo(),
        },
        Commands::Verify { command } => match command {
            VerifyCommand::All => experiments::run_all_verifications(),
            VerifyCommand::Inplace => experiments::engine_verification::inplace_verification::run(),
            VerifyCommand::CnotInplace => {
                experiments::engine_verification::cnot_inplace_verification::run()
            }
        },
    }
    Ok(())
}
