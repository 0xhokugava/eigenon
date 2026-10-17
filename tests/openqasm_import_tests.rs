use eigenon::circuit::core::Circuit;
use eigenon::circuit::operation::{GateKind, Operation};
use eigenon::export::openqasm::export_openqasm2;
use eigenon::import::openqasm::{OpenQasmImportError, import_openqasm2};

const HEADER: &str = "OPENQASM 2.0;\ninclude \"qelib1.inc\";\n";

#[test]
fn imports_every_supported_operation() {
    let source = format!(
        "{HEADER}\
         qreg data[3];\n\
         creg result[4];\n\
         id data[0];\n\
         x data[1];\n\
         y data[2];\n\
         z data[0];\n\
         h data[1];\n\
         s data[2];\n\
         t data[0];\n\
         cx data[0], data[1];\n\
         cz data[2], data[0];\n\
         ccx data[2], data[0], data[1];\n\
         measure data[1] -> result[3];\n"
    );

    let circuit = import_openqasm2(&source).unwrap();

    assert_eq!(circuit.n_qubits(), 3);
    assert_eq!(circuit.n_classical_bits(), 4);
    assert_eq!(
        circuit.operations(),
        &[
            Operation::SingleQubit {
                gate: GateKind::I,
                target: 0,
            },
            Operation::SingleQubit {
                gate: GateKind::X,
                target: 1,
            },
            Operation::SingleQubit {
                gate: GateKind::Y,
                target: 2,
            },
            Operation::SingleQubit {
                gate: GateKind::Z,
                target: 0,
            },
            Operation::SingleQubit {
                gate: GateKind::H,
                target: 1,
            },
            Operation::SingleQubit {
                gate: GateKind::S,
                target: 2,
            },
            Operation::SingleQubit {
                gate: GateKind::T,
                target: 0,
            },
            Operation::Cnot {
                control: 0,
                target: 1,
            },
            Operation::Cz {
                control: 2,
                target: 0,
            },
            Operation::Mcx {
                controls: vec![2, 0],
                target: 1,
            },
            Operation::Measure {
                qubit: 1,
                classical_bit: 3,
            },
        ]
    );
}

#[test]
fn imports_comments_and_preserves_source_locations() {
    let source = "// leading comment\n\
                  OPENQASM 2.0; // header\n\
                  include \"qelib1.inc\";\n\
                  qreg q[2];\n\
                  h q[0]; // prepare superposition\n\
                  cx q[0], q[2];\n";

    assert!(matches!(
        import_openqasm2(source),
        Err(OpenQasmImportError::Semantic { line: 6, .. })
    ));
}

#[test]
fn imports_builtin_uppercase_cx_without_qelib() {
    let circuit = import_openqasm2("OPENQASM 2.0; qreg q[2]; CX q[1], q[0];").unwrap();

    assert_eq!(
        circuit.operations(),
        &[Operation::Cnot {
            control: 1,
            target: 0,
        }]
    );
}

#[test]
fn accepts_valid_mixed_case_identifiers_after_lowercase_prefix() {
    let source = format!(
        "{HEADER}qreg quantum_Data_2[1];\ncreg classical_Data_2[1];\n\
         measure quantum_Data_2[0] -> classical_Data_2[0];\n"
    );

    let circuit = import_openqasm2(&source).unwrap();

    assert_eq!(circuit.n_qubits(), 1);
    assert_eq!(circuit.n_classical_bits(), 1);
}

#[test]
fn rejects_identifiers_with_invalid_prefixes_or_reserved_words() {
    for source in [
        "OPENQASM 2.0; qreg Q[1];",
        "OPENQASM 2.0; qreg _q[1];",
        "OPENQASM 2.0; qreg qreg[1];",
        "OPENQASM 2.0; qreg measure[1];",
        "OPENQASM 2.0; qreg q[1]; FOO q[0];",
    ] {
        assert!(matches!(
            import_openqasm2(source),
            Err(OpenQasmImportError::Syntax { .. })
        ));
    }
}

#[test]
fn exporter_output_round_trips_through_importer() {
    let mut original = Circuit::with_classical_bits(3, 4);
    original
        .h(0)
        .x(2)
        .cnot(0, 1)
        .cz(2, 0)
        .mcx(&[2, 0], 1)
        .measure(1, 3);

    let source = export_openqasm2(&original).unwrap();
    let imported = import_openqasm2(&source).unwrap();

    assert_eq!(imported.n_qubits(), original.n_qubits());
    assert_eq!(imported.n_classical_bits(), original.n_classical_bits());
    assert_eq!(imported.operations(), original.operations());
}

#[test]
fn decomposed_mcz_round_trip_preserves_statevector() {
    let mut original = Circuit::new(3);
    original.x(0).x(1).h(2).mcz(&[0, 1], 2);

    let source = export_openqasm2(&original).unwrap();
    let imported = import_openqasm2(&source).unwrap();
    let expected = original.run();
    let actual = imported.run();

    assert_eq!(actual.shape(), expected.shape());
    for (actual, expected) in actual.iter().zip(expected.iter()) {
        assert!((*actual - *expected).norm() < 1e-12);
    }
}

#[test]
fn rejects_unsupported_version_and_include() {
    assert!(matches!(
        import_openqasm2("OPENQASM 3.0;"),
        Err(OpenQasmImportError::UnsupportedOpenQasm3 {
            line: 1,
            column: 10,
        })
    ));

    assert!(matches!(
        import_openqasm2("OPENQASM 2.0; include \"stdgates.inc\"; qreg q[1];"),
        Err(OpenQasmImportError::UnsupportedInclude {
            path,
            line: 1,
            column: 15,
        }) if path == "stdgates.inc"
    ));
}

#[test]
fn identifies_official_headerless_openqasm3_example() {
    let source = include_str!("fixtures/openqasm3/inverseqft1.qasm");

    assert!(matches!(
        import_openqasm2(source),
        Err(OpenQasmImportError::UnsupportedOpenQasm3 { line: 2, column: 1 })
    ));
}

#[test]
fn rejects_parameterized_and_structural_features_explicitly() {
    for (source, expected_feature) in [
        (
            "OPENQASM 2.0; qreg q[1]; U(0,0,0) q[0];",
            "parameterized gate `U`",
        ),
        ("OPENQASM 2.0; gate custom a { x a; } qreg q[1];", "gate"),
        ("OPENQASM 2.0; qreg q[1]; barrier q[0];", "barrier"),
    ] {
        assert!(matches!(
            import_openqasm2(source),
            Err(OpenQasmImportError::UnsupportedFeature { feature, .. })
                if feature == expected_feature
        ));
    }
}

#[test]
fn rejects_unknown_gate_and_missing_qelib() {
    let unknown = format!("{HEADER}qreg q[1];\nfoo q[0];\n");
    assert!(matches!(
        import_openqasm2(&unknown),
        Err(OpenQasmImportError::UnknownGate { gate, .. }) if gate == "foo"
    ));

    assert!(matches!(
        import_openqasm2("OPENQASM 2.0; qreg q[1]; h q[0];"),
        Err(OpenQasmImportError::Semantic { message, .. })
            if message.contains("requires `include")
    ));
}

#[test]
fn rejects_invalid_register_declarations() {
    for source in [
        "OPENQASM 2.0; qreg q[0];",
        "OPENQASM 2.0; qreg q[1]; qreg other[1];",
        "OPENQASM 2.0; qreg bits[1]; creg bits[1];",
        "OPENQASM 2.0; creg c[1];",
    ] {
        assert!(import_openqasm2(source).is_err(), "accepted: {source}");
    }

    assert!(matches!(
        import_openqasm2("OPENQASM 2.0; creg c[1];"),
        Err(OpenQasmImportError::MissingQuantumRegister)
    ));
}

#[test]
fn rejects_invalid_gate_operands_without_panicking() {
    for source in [
        "h q[2];",
        "cx q[0];",
        "cx q[0], q[0];",
        "ccx q[0], q[1], q[1];",
        "x c[0];",
        "x missing[0];",
        "x q;",
    ] {
        let program = format!("{HEADER}qreg q[2];\ncreg c[2];\n{source}\n");
        assert!(import_openqasm2(&program).is_err(), "accepted: {source}");
    }
}

#[test]
fn rejects_invalid_measurements() {
    for source in [
        "measure q[0] -> c[2];",
        "measure c[0] -> c[0];",
        "measure q[0] -> q[1];",
        "measure q -> c;",
    ] {
        let program = format!("{HEADER}qreg q[2];\ncreg c[2];\n{source}\n");
        assert!(import_openqasm2(&program).is_err(), "accepted: {source}");
    }

    let missing_classical = format!("{HEADER}qreg q[1];\nmeasure q[0] -> c[0];\n");
    assert!(matches!(
        import_openqasm2(&missing_classical),
        Err(OpenQasmImportError::Semantic { message, .. })
            if message.contains("requires a classical register")
    ));
}

#[test]
fn malformed_input_returns_errors_instead_of_panicking() {
    for source in [
        "",
        "OPENQASM",
        "OPENQASM 2.0",
        "OPENQASM 2..0;",
        "OPENQASM 2.0; include \"qelib1.inc;",
        "OPENQASM 2.0; qreg q[999999999999999999999999999999];",
        "OPENQASM 2.0; qreg q[1]; x q[0]",
        "OPENQASM 2.0; qreg q[1]; @",
        "OPENQASM 2.0; /* block comments are not in the supported grammar */ qreg q[1];",
    ] {
        assert!(import_openqasm2(source).is_err(), "accepted: {source}");
    }
}

#[test]
fn every_truncated_prefix_is_handled_without_panicking() {
    let source = "OPENQASM 2.0;\n\
                  include \"qelib1.inc\";\n\
                  qreg q[3];\n\
                  creg c[3];\n\
                  h q[0];\n\
                  ccx q[0], q[1], q[2];\n\
                  measure q[2] -> c[1];\n";

    for end in source.char_indices().map(|(index, _)| index) {
        let prefix = &source[..end];
        let outcome = std::panic::catch_unwind(|| import_openqasm2(prefix));
        assert!(outcome.is_ok(), "import panicked for prefix `{prefix}`");
    }
}

#[test]
fn rejects_duplicate_qelib_include() {
    let source = format!("{HEADER}include \"qelib1.inc\";\nqreg q[1];\n");
    assert!(matches!(
        import_openqasm2(&source),
        Err(OpenQasmImportError::Semantic { message, .. })
            if message.contains("duplicate")
    ));
}
