//! OpenQASM 2.0 circuit import.
//!
//! The importer accepts one quantum register, an optional classical register,
//! `qelib1.inc`, Eigenon's built-in single-qubit gates, `cx`, `cz`, `ccx`, and
//! indexed measurements. Unsupported OpenQASM features are rejected explicitly
//! instead of being silently discarded.

use crate::circuit::core::Circuit;
use crate::circuit::operation::GateKind;
use std::fmt;

/// Error returned when OpenQASM 2.0 source cannot be imported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenQasmImportError {
    /// The source is not syntactically valid for the supported grammar.
    Syntax {
        /// One-based source line.
        line: usize,
        /// One-based source column.
        column: usize,
        /// Description of the syntax problem.
        message: String,
    },
    /// The source declares an OpenQASM version other than 2.0.
    UnsupportedVersion {
        /// Version text found in the source.
        version: String,
        /// One-based source line.
        line: usize,
        /// One-based source column.
        column: usize,
    },
    /// The source uses OpenQASM 3 syntax, which this importer does not support.
    UnsupportedOpenQasm3 {
        /// One-based source line where OpenQASM 3 was identified.
        line: usize,
        /// One-based source column where OpenQASM 3 was identified.
        column: usize,
    },
    /// The source requests an include other than `qelib1.inc`.
    UnsupportedInclude {
        /// Included path found in the source.
        path: String,
        /// One-based source line.
        line: usize,
        /// One-based source column.
        column: usize,
    },
    /// The source uses a valid OpenQASM concept not represented by Eigenon.
    UnsupportedFeature {
        /// Human-readable feature name.
        feature: String,
        /// One-based source line.
        line: usize,
        /// One-based source column.
        column: usize,
    },
    /// The source calls a gate unknown to the importer.
    UnknownGate {
        /// Gate name found in the source.
        gate: String,
        /// One-based source line.
        line: usize,
        /// One-based source column.
        column: usize,
    },
    /// The program is syntactically valid but semantically invalid.
    Semantic {
        /// One-based source line.
        line: usize,
        /// One-based source column.
        column: usize,
        /// Description of the semantic problem.
        message: String,
    },
    /// The program does not declare a quantum register.
    MissingQuantumRegister,
}

impl fmt::Display for OpenQasmImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax {
                line,
                column,
                message,
            } => write!(f, "OpenQASM syntax error at {line}:{column}: {message}"),
            Self::UnsupportedVersion {
                version,
                line,
                column,
            } => write!(
                f,
                "unsupported OpenQASM version `{version}` at {line}:{column}; expected 2.0"
            ),
            Self::UnsupportedOpenQasm3 { line, column } => write!(
                f,
                "OpenQASM 3 import is not supported yet at {line}:{column}; current importer supports only the OpenQASM 2.0 subset"
            ),
            Self::UnsupportedInclude { path, line, column } => write!(
                f,
                "unsupported OpenQASM include `{path}` at {line}:{column}; only qelib1.inc is supported"
            ),
            Self::UnsupportedFeature {
                feature,
                line,
                column,
            } => write!(
                f,
                "unsupported OpenQASM feature `{feature}` at {line}:{column}"
            ),
            Self::UnknownGate { gate, line, column } => {
                write!(f, "unknown OpenQASM gate `{gate}` at {line}:{column}")
            }
            Self::Semantic {
                line,
                column,
                message,
            } => write!(f, "OpenQASM semantic error at {line}:{column}: {message}"),
            Self::MissingQuantumRegister => {
                write!(f, "OpenQASM program does not declare a quantum register")
            }
        }
    }
}

impl std::error::Error for OpenQasmImportError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Location {
    line: usize,
    column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TokenKind {
    Identifier(String),
    Number(String),
    StringLiteral(String),
    Semicolon,
    LeftBracket,
    RightBracket,
    Comma,
    LeftParen,
    RightParen,
    Arrow,
    Symbol(char),
    End,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    location: Location,
}

struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            chars: source.char_indices().peekable(),
            line: 1,
            column: 1,
        }
    }

    fn tokenize(mut self) -> Result<Vec<Token>, OpenQasmImportError> {
        let mut tokens = Vec::new();

        while let Some((_, character)) = self.chars.peek().copied() {
            if character.is_whitespace() {
                self.advance();
                continue;
            }

            let location = self.location();

            if character == '/' {
                self.advance();
                if self.chars.peek().is_some_and(|(_, next)| *next == '/') {
                    self.advance();
                    while let Some((_, next)) = self.chars.peek().copied() {
                        self.advance();
                        if next == '\n' {
                            break;
                        }
                    }
                    continue;
                }

                tokens.push(Token {
                    kind: TokenKind::Symbol('/'),
                    location,
                });
                continue;
            }

            let kind = match character {
                ';' => {
                    self.advance();
                    TokenKind::Semicolon
                }
                '[' => {
                    self.advance();
                    TokenKind::LeftBracket
                }
                ']' => {
                    self.advance();
                    TokenKind::RightBracket
                }
                ',' => {
                    self.advance();
                    TokenKind::Comma
                }
                '(' => {
                    self.advance();
                    TokenKind::LeftParen
                }
                ')' => {
                    self.advance();
                    TokenKind::RightParen
                }
                '-' => {
                    self.advance();
                    if self.chars.peek().is_some_and(|(_, next)| *next == '>') {
                        self.advance();
                        TokenKind::Arrow
                    } else {
                        TokenKind::Symbol('-')
                    }
                }
                '"' => self.string_literal(location)?,
                character if character.is_ascii_digit() => self.number(),
                character if is_identifier_start(character) => self.identifier(),
                character => {
                    self.advance();
                    TokenKind::Symbol(character)
                }
            };

            tokens.push(Token { kind, location });
        }

        tokens.push(Token {
            kind: TokenKind::End,
            location: self.location(),
        });

        Ok(tokens)
    }

    fn advance(&mut self) -> Option<char> {
        let (_, character) = self.chars.next()?;
        if character == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(character)
    }

    fn location(&self) -> Location {
        Location {
            line: self.line,
            column: self.column,
        }
    }

    fn identifier(&mut self) -> TokenKind {
        let mut value = String::new();
        while let Some((_, character)) = self.chars.peek().copied() {
            if !is_identifier_continue(character) {
                break;
            }
            value.push(character);
            self.advance();
        }
        TokenKind::Identifier(value)
    }

    fn number(&mut self) -> TokenKind {
        let mut value = String::new();
        while let Some((_, character)) = self.chars.peek().copied() {
            if !character.is_ascii_digit() && character != '.' {
                break;
            }
            value.push(character);
            self.advance();
        }
        TokenKind::Number(value)
    }

    fn string_literal(&mut self, location: Location) -> Result<TokenKind, OpenQasmImportError> {
        self.advance();
        let mut value = String::new();

        while let Some((_, character)) = self.chars.peek().copied() {
            match character {
                '"' => {
                    self.advance();
                    return Ok(TokenKind::StringLiteral(value));
                }
                '\n' | '\r' => {
                    return Err(syntax(location, "unterminated string literal"));
                }
                '\\' => {
                    self.advance();
                    let Some(escaped) = self.advance() else {
                        return Err(syntax(location, "unterminated string literal"));
                    };
                    match escaped {
                        '"' | '\\' => value.push(escaped),
                        other => {
                            return Err(syntax(
                                location,
                                format!("unsupported escape sequence `\\{other}`"),
                            ));
                        }
                    }
                }
                _ => {
                    value.push(character);
                    self.advance();
                }
            }
        }

        Err(syntax(location, "unterminated string literal"))
    }
}

fn is_identifier_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

fn is_identifier_continue(character: char) -> bool {
    is_identifier_start(character) || character.is_ascii_digit()
}

fn is_openqasm_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_lowercase())
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn is_reserved_identifier(value: &str) -> bool {
    matches!(
        value,
        "OPENQASM"
            | "include"
            | "opaque"
            | "qreg"
            | "creg"
            | "barrier"
            | "if"
            | "measure"
            | "reset"
            | "gate"
            | "U"
            | "CX"
            | "pi"
            | "sin"
            | "cos"
            | "tan"
            | "exp"
            | "ln"
            | "sqrt"
    )
}

#[derive(Debug)]
struct Program {
    statements: Vec<Statement>,
}

#[derive(Debug)]
enum Statement {
    Include {
        path: String,
        location: Location,
    },
    QuantumRegister {
        name: String,
        size: usize,
        location: Location,
    },
    ClassicalRegister {
        name: String,
        size: usize,
        location: Location,
    },
    Gate {
        name: String,
        operands: Vec<Operand>,
        location: Location,
    },
    Measure {
        qubit: Operand,
        classical_bit: Operand,
        location: Location,
    },
}

#[derive(Debug)]
struct Operand {
    register: String,
    index: Option<usize>,
    location: Location,
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn parse(mut self) -> Result<Program, OpenQasmImportError> {
        self.parse_header()?;
        let mut statements = Vec::new();

        while !matches!(self.peek().kind, TokenKind::End) {
            statements.push(self.parse_statement()?);
        }

        Ok(Program { statements })
    }

    fn parse_header(&mut self) -> Result<(), OpenQasmImportError> {
        let header = self.expect_identifier("`OPENQASM` header")?;
        if header.0 != "OPENQASM" {
            return Err(syntax(header.1, "expected `OPENQASM` header"));
        }

        let version = self.expect_number("OpenQASM version")?;
        if is_openqasm3_version(&version.0) {
            return Err(OpenQasmImportError::UnsupportedOpenQasm3 {
                line: version.1.line,
                column: version.1.column,
            });
        }
        if version.0 != "2.0" {
            return Err(OpenQasmImportError::UnsupportedVersion {
                version: version.0,
                line: version.1.line,
                column: version.1.column,
            });
        }

        self.expect_simple(TokenKind::Semicolon, "`;` after OpenQASM version")
    }

    fn parse_statement(&mut self) -> Result<Statement, OpenQasmImportError> {
        let token = self.peek().clone();
        let TokenKind::Identifier(keyword) = &token.kind else {
            return Err(syntax(
                token.location,
                format!("expected statement, found {}", describe_token(&token.kind)),
            ));
        };

        match keyword.as_str() {
            "include" => self.parse_include(),
            "qreg" => self.parse_register(true),
            "creg" => self.parse_register(false),
            "measure" => self.parse_measurement(),
            "gate" | "opaque" | "if" | "reset" | "barrier" => {
                Err(OpenQasmImportError::UnsupportedFeature {
                    feature: keyword.clone(),
                    line: token.location.line,
                    column: token.location.column,
                })
            }
            "OPENQASM" => Err(syntax(token.location, "duplicate `OPENQASM` header")),
            _ => self.parse_gate(),
        }
    }

    fn parse_include(&mut self) -> Result<Statement, OpenQasmImportError> {
        let location = self.advance().location;
        let path = self.expect_string("include path")?;
        self.expect_simple(TokenKind::Semicolon, "`;` after include")?;
        Ok(Statement::Include {
            path: path.0,
            location,
        })
    }

    fn parse_register(&mut self, quantum: bool) -> Result<Statement, OpenQasmImportError> {
        let location = self.advance().location;
        let name = self.expect_user_identifier("register name")?.0;
        self.expect_simple(TokenKind::LeftBracket, "`[` after register name")?;
        let size = self.expect_integer("register size")?.0;
        self.expect_simple(TokenKind::RightBracket, "`]` after register size")?;
        self.expect_simple(TokenKind::Semicolon, "`;` after register declaration")?;

        if quantum {
            Ok(Statement::QuantumRegister {
                name,
                size,
                location,
            })
        } else {
            Ok(Statement::ClassicalRegister {
                name,
                size,
                location,
            })
        }
    }

    fn parse_measurement(&mut self) -> Result<Statement, OpenQasmImportError> {
        let location = self.advance().location;
        let qubit = self.parse_operand()?;
        self.expect_simple(TokenKind::Arrow, "`->` in measurement")?;
        let classical_bit = self.parse_operand()?;
        self.expect_simple(TokenKind::Semicolon, "`;` after measurement")?;
        Ok(Statement::Measure {
            qubit,
            classical_bit,
            location,
        })
    }

    fn parse_gate(&mut self) -> Result<Statement, OpenQasmImportError> {
        let (name, location) = self.expect_identifier("gate name")?;

        if name != "U" && name != "CX" {
            validate_user_identifier(&name, location, "gate name")?;
        }

        if matches!(self.peek().kind, TokenKind::LeftParen) {
            return Err(OpenQasmImportError::UnsupportedFeature {
                feature: format!("parameterized gate `{name}`"),
                line: location.line,
                column: location.column,
            });
        }

        let mut operands = vec![self.parse_operand()?];
        while matches!(self.peek().kind, TokenKind::Comma) {
            self.advance();
            operands.push(self.parse_operand()?);
        }
        self.expect_simple(TokenKind::Semicolon, "`;` after gate call")?;

        Ok(Statement::Gate {
            name,
            operands,
            location,
        })
    }

    fn parse_operand(&mut self) -> Result<Operand, OpenQasmImportError> {
        let (register, location) = self.expect_user_identifier("register operand")?;
        let index = if matches!(self.peek().kind, TokenKind::LeftBracket) {
            self.advance();
            let index = self.expect_integer("register index")?.0;
            self.expect_simple(TokenKind::RightBracket, "`]` after register index")?;
            Some(index)
        } else {
            None
        };

        Ok(Operand {
            register,
            index,
            location,
        })
    }

    fn expect_identifier(
        &mut self,
        expected: &str,
    ) -> Result<(String, Location), OpenQasmImportError> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Identifier(value) => Ok((value, token.location)),
            other => Err(syntax(
                token.location,
                format!("expected {expected}, found {}", describe_token(&other)),
            )),
        }
    }

    fn expect_user_identifier(
        &mut self,
        expected: &str,
    ) -> Result<(String, Location), OpenQasmImportError> {
        let (value, location) = self.expect_identifier(expected)?;
        validate_user_identifier(&value, location, expected)?;
        Ok((value, location))
    }

    fn expect_number(&mut self, expected: &str) -> Result<(String, Location), OpenQasmImportError> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Number(value) => Ok((value, token.location)),
            other => Err(syntax(
                token.location,
                format!("expected {expected}, found {}", describe_token(&other)),
            )),
        }
    }

    fn expect_integer(&mut self, expected: &str) -> Result<(usize, Location), OpenQasmImportError> {
        let (number, location) = self.expect_number(expected)?;
        if number.contains('.') {
            return Err(syntax(location, format!("{expected} must be an integer")));
        }
        let value = number
            .parse::<usize>()
            .map_err(|_| syntax(location, format!("{expected} is too large")))?;
        Ok((value, location))
    }

    fn expect_string(&mut self, expected: &str) -> Result<(String, Location), OpenQasmImportError> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::StringLiteral(value) => Ok((value, token.location)),
            other => Err(syntax(
                token.location,
                format!("expected {expected}, found {}", describe_token(&other)),
            )),
        }
    }

    fn expect_simple(
        &mut self,
        expected_kind: TokenKind,
        expected: &str,
    ) -> Result<(), OpenQasmImportError> {
        let token = self.advance().clone();
        if token.kind == expected_kind {
            Ok(())
        } else {
            Err(syntax(
                token.location,
                format!("expected {expected}, found {}", describe_token(&token.kind)),
            ))
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.position];
        if !matches!(token.kind, TokenKind::End) {
            self.position += 1;
        }
        token
    }
}

fn validate_user_identifier(
    value: &str,
    location: Location,
    expected: &str,
) -> Result<(), OpenQasmImportError> {
    if !is_openqasm_identifier(value) {
        return Err(syntax(
            location,
            format!("invalid {expected} `{value}`; expected `[a-z][A-Za-z0-9_]*`"),
        ));
    }

    if is_reserved_identifier(value) {
        return Err(syntax(
            location,
            format!("reserved word `{value}` cannot be used as {expected}"),
        ));
    }

    Ok(())
}

fn describe_token(kind: &TokenKind) -> String {
    match kind {
        TokenKind::Identifier(value) | TokenKind::Number(value) => format!("`{value}`"),
        TokenKind::StringLiteral(value) => format!("\"{value}\""),
        TokenKind::Semicolon => "`;`".to_string(),
        TokenKind::LeftBracket => "`[`".to_string(),
        TokenKind::RightBracket => "`]`".to_string(),
        TokenKind::Comma => "`,`".to_string(),
        TokenKind::LeftParen => "`(`".to_string(),
        TokenKind::RightParen => "`)`".to_string(),
        TokenKind::Arrow => "`->`".to_string(),
        TokenKind::Symbol(character) => format!("`{character}`"),
        TokenKind::End => "end of input".to_string(),
    }
}

#[derive(Debug)]
struct Register {
    name: String,
    size: usize,
}

#[derive(Debug)]
enum LoweredOperation {
    SingleQubit { gate: GateKind, target: usize },
    Cnot { control: usize, target: usize },
    Cz { control: usize, target: usize },
    Mcx { controls: [usize; 2], target: usize },
    Measure { qubit: usize, classical_bit: usize },
}

/// Imports an OpenQASM 2.0 program into an Eigenon circuit.
///
/// The supported subset contains one quantum register, an optional classical
/// register, `qelib1.inc`, `id`, `x`, `y`, `z`, `h`, `s`, `t`, `cx`, `cz`,
/// `ccx`, and indexed measurements. Register names are not preserved because
/// [`Circuit`] stores flat quantum and classical bit spaces.
///
/// Qubit indices are preserved exactly. Qubit `0` remains Eigenon's least
/// significant bit, and controlled gates retain OpenQASM operand order.
///
/// # Errors
///
/// Returns [`OpenQasmImportError`] for malformed input, unsupported language
/// features, invalid register references, or operations that cannot be
/// represented by [`Circuit`].
///
/// # Example
///
/// ```
/// use eigenon::import::openqasm::import_openqasm2;
///
/// let source = r#"OPENQASM 2.0;
/// include "qelib1.inc";
/// qreg q[2];
/// h q[0];
/// cx q[0], q[1];
/// "#;
///
/// let circuit = import_openqasm2(source)?;
/// assert_eq!(circuit.n_qubits(), 2);
/// assert_eq!(circuit.operations().len(), 2);
///
/// # Ok::<(), eigenon::import::openqasm::OpenQasmImportError>(())
/// ```
pub fn import_openqasm2(source: &str) -> Result<Circuit, OpenQasmImportError> {
    let tokens = Lexer::new(source).tokenize()?;
    if let Some(location) = inferred_openqasm3_location(&tokens) {
        return Err(OpenQasmImportError::UnsupportedOpenQasm3 {
            line: location.line,
            column: location.column,
        });
    }
    let program = Parser::new(tokens).parse()?;
    lower_program(program)
}

fn is_openqasm3_version(version: &str) -> bool {
    version.split('.').next() == Some("3")
}

fn inferred_openqasm3_location(tokens: &[Token]) -> Option<Location> {
    if matches!(
        tokens.first().map(|token| &token.kind),
        Some(TokenKind::Identifier(identifier)) if identifier == "OPENQASM"
    ) {
        return None;
    }

    for (index, token) in tokens.iter().enumerate() {
        let TokenKind::Identifier(identifier) = &token.kind else {
            continue;
        };

        if matches!(
            identifier.as_str(),
            "qubit" | "bit" | "uint" | "int" | "float" | "angle" | "bool" | "const" | "let"
        ) {
            return Some(token.location);
        }

        if identifier == "include"
            && matches!(
                tokens.get(index + 1).map(|next| &next.kind),
                Some(TokenKind::StringLiteral(path)) if path == "stdgates.inc"
            )
        {
            return Some(token.location);
        }
    }

    None
}

fn lower_program(program: Program) -> Result<Circuit, OpenQasmImportError> {
    let mut quantum_register: Option<Register> = None;
    let mut classical_register: Option<Register> = None;
    let mut qelib1_included = false;
    let mut operations = Vec::new();

    for statement in program.statements {
        match statement {
            Statement::Include { path, location } => {
                if path != "qelib1.inc" {
                    return Err(OpenQasmImportError::UnsupportedInclude {
                        path,
                        line: location.line,
                        column: location.column,
                    });
                }
                if qelib1_included {
                    return Err(semantic(location, "duplicate `qelib1.inc` include"));
                }
                qelib1_included = true;
            }
            Statement::QuantumRegister {
                name,
                size,
                location,
            } => {
                validate_register_declaration(
                    &name,
                    size,
                    location,
                    quantum_register.as_ref(),
                    classical_register.as_ref(),
                    "quantum",
                )?;
                quantum_register = Some(Register { name, size });
            }
            Statement::ClassicalRegister {
                name,
                size,
                location,
            } => {
                validate_register_declaration(
                    &name,
                    size,
                    location,
                    classical_register.as_ref(),
                    quantum_register.as_ref(),
                    "classical",
                )?;
                classical_register = Some(Register { name, size });
            }
            Statement::Gate {
                name,
                operands,
                location,
            } => {
                let Some(register) = quantum_register.as_ref() else {
                    return Err(semantic(
                        location,
                        "gate call appears before the quantum register declaration",
                    ));
                };
                operations.push(lower_gate(
                    name,
                    operands,
                    location,
                    register,
                    classical_register.as_ref(),
                    qelib1_included,
                )?);
            }
            Statement::Measure {
                qubit,
                classical_bit,
                location,
            } => {
                let Some(quantum) = quantum_register.as_ref() else {
                    return Err(semantic(
                        location,
                        "measurement appears before the quantum register declaration",
                    ));
                };
                let Some(classical) = classical_register.as_ref() else {
                    return Err(semantic(
                        location,
                        "measurement requires a classical register",
                    ));
                };
                let qubit = resolve_operand(&qubit, quantum, classical_register.as_ref(), true)?;
                let classical_bit =
                    resolve_operand(&classical_bit, classical, Some(quantum), false)?;
                operations.push(LoweredOperation::Measure {
                    qubit,
                    classical_bit,
                });
            }
        }
    }

    let Some(quantum) = quantum_register else {
        return Err(OpenQasmImportError::MissingQuantumRegister);
    };

    let mut circuit = if let Some(classical) = classical_register {
        Circuit::with_classical_bits(quantum.size, classical.size)
    } else {
        Circuit::new(quantum.size)
    };

    for operation in operations {
        match operation {
            LoweredOperation::SingleQubit { gate, target } => match gate {
                GateKind::I => {
                    circuit.i(target);
                }
                GateKind::X => {
                    circuit.x(target);
                }
                GateKind::Y => {
                    circuit.y(target);
                }
                GateKind::Z => {
                    circuit.z(target);
                }
                GateKind::H => {
                    circuit.h(target);
                }
                GateKind::S => {
                    circuit.s(target);
                }
                GateKind::T => {
                    circuit.t(target);
                }
            },
            LoweredOperation::Cnot { control, target } => {
                circuit.cnot(control, target);
            }
            LoweredOperation::Cz { control, target } => {
                circuit.cz(control, target);
            }
            LoweredOperation::Mcx { controls, target } => {
                circuit.mcx(&controls, target);
            }
            LoweredOperation::Measure {
                qubit,
                classical_bit,
            } => {
                circuit.measure(qubit, classical_bit);
            }
        }
    }

    Ok(circuit)
}

fn validate_register_declaration(
    name: &str,
    size: usize,
    location: Location,
    same_kind: Option<&Register>,
    other_kind: Option<&Register>,
    register_kind: &str,
) -> Result<(), OpenQasmImportError> {
    if size == 0 {
        return Err(semantic(
            location,
            format!("{register_kind} register `{name}` must not be empty"),
        ));
    }
    if let Some(existing) = same_kind {
        return Err(OpenQasmImportError::UnsupportedFeature {
            feature: format!(
                "multiple {register_kind} registers (`{}` and `{name}`)",
                existing.name
            ),
            line: location.line,
            column: location.column,
        });
    }
    if other_kind.is_some_and(|existing| existing.name == name) {
        return Err(semantic(
            location,
            format!("register name `{name}` is already declared"),
        ));
    }
    Ok(())
}

fn lower_gate(
    name: String,
    operands: Vec<Operand>,
    location: Location,
    quantum: &Register,
    classical: Option<&Register>,
    qelib1_included: bool,
) -> Result<LoweredOperation, OpenQasmImportError> {
    let expected_operands = match name.as_str() {
        "id" | "x" | "y" | "z" | "h" | "s" | "t" => 1,
        "CX" | "cx" | "cz" => 2,
        "ccx" => 3,
        _ => {
            return Err(OpenQasmImportError::UnknownGate {
                gate: name,
                line: location.line,
                column: location.column,
            });
        }
    };

    if name != "CX" && !qelib1_included {
        return Err(semantic(
            location,
            format!("gate `{name}` requires `include \"qelib1.inc\";`"),
        ));
    }

    if operands.len() != expected_operands {
        return Err(semantic(
            location,
            format!(
                "gate `{name}` expects {expected_operands} operand(s), found {}",
                operands.len()
            ),
        ));
    }

    let indices = operands
        .iter()
        .map(|operand| resolve_operand(operand, quantum, classical, true))
        .collect::<Result<Vec<_>, _>>()?;

    for (position, index) in indices.iter().enumerate() {
        if indices[..position].contains(index) {
            return Err(semantic(
                operands[position].location,
                format!("gate `{name}` uses qubit {index} more than once"),
            ));
        }
    }

    match name.as_str() {
        "id" => Ok(LoweredOperation::SingleQubit {
            gate: GateKind::I,
            target: indices[0],
        }),
        "x" => Ok(LoweredOperation::SingleQubit {
            gate: GateKind::X,
            target: indices[0],
        }),
        "y" => Ok(LoweredOperation::SingleQubit {
            gate: GateKind::Y,
            target: indices[0],
        }),
        "z" => Ok(LoweredOperation::SingleQubit {
            gate: GateKind::Z,
            target: indices[0],
        }),
        "h" => Ok(LoweredOperation::SingleQubit {
            gate: GateKind::H,
            target: indices[0],
        }),
        "s" => Ok(LoweredOperation::SingleQubit {
            gate: GateKind::S,
            target: indices[0],
        }),
        "t" => Ok(LoweredOperation::SingleQubit {
            gate: GateKind::T,
            target: indices[0],
        }),
        "CX" | "cx" => Ok(LoweredOperation::Cnot {
            control: indices[0],
            target: indices[1],
        }),
        "cz" => Ok(LoweredOperation::Cz {
            control: indices[0],
            target: indices[1],
        }),
        "ccx" => Ok(LoweredOperation::Mcx {
            controls: [indices[0], indices[1]],
            target: indices[2],
        }),
        _ => unreachable!("gate name was validated above"),
    }
}

fn resolve_operand(
    operand: &Operand,
    expected: &Register,
    other_kind: Option<&Register>,
    quantum: bool,
) -> Result<usize, OpenQasmImportError> {
    if operand.register != expected.name {
        let message = if other_kind.is_some_and(|register| register.name == operand.register) {
            let actual = if quantum { "classical" } else { "quantum" };
            let expected_kind = if quantum { "quantum" } else { "classical" };
            format!(
                "expected a {expected_kind} operand, but `{}` is a {actual} register",
                operand.register
            )
        } else {
            format!("unknown register `{}`", operand.register)
        };
        return Err(semantic(operand.location, message));
    }

    let Some(index) = operand.index else {
        return Err(OpenQasmImportError::UnsupportedFeature {
            feature: format!("whole-register operand `{}`", operand.register),
            line: operand.location.line,
            column: operand.location.column,
        });
    };

    if index >= expected.size {
        return Err(semantic(
            operand.location,
            format!(
                "index {index} is out of range for register `{}` of size {}",
                expected.name, expected.size
            ),
        ));
    }

    Ok(index)
}

fn syntax(location: Location, message: impl Into<String>) -> OpenQasmImportError {
    OpenQasmImportError::Syntax {
        line: location.line,
        column: location.column,
        message: message.into(),
    }
}

fn semantic(location: Location, message: impl Into<String>) -> OpenQasmImportError {
    OpenQasmImportError::Semantic {
        line: location.line,
        column: location.column,
        message: message.into(),
    }
}
