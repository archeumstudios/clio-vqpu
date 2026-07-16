//! Bounded Clio Assembly line parser with retained source spans.

#![forbid(unsafe_code)]

use clio_core::{Diagnostic, SourceSpan};

/// Maximum accepted source size for the initial front end.
pub const MAX_SOURCE_BYTES: usize = 1024 * 1024;

/// A parsed directive before semantic interpretation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Directive {
    /// Lowercase directive name without the dot.
    pub name: String,
    /// Remaining trimmed directive text.
    pub value: String,
    /// Full directive source span.
    pub span: SourceSpan,
}

/// A parsed instruction before typed operand validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedInstruction {
    /// Uppercase mnemonic.
    pub mnemonic: String,
    /// Comma-delimited trimmed operand strings.
    pub operands: Vec<String>,
    /// Mnemonic source span.
    pub span: SourceSpan,
}

/// A source label resolved by the assembler.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedLabel {
    /// Case-sensitive symbol name.
    pub name: String,
    /// Instruction index designated by the label.
    pub instruction_index: usize,
    /// Label source span.
    pub span: SourceSpan,
}

/// A syntactically parsed source program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedProgram {
    /// Directives in source order.
    pub directives: Vec<Directive>,
    /// Labels in source order.
    pub labels: Vec<ParsedLabel>,
    /// Instructions in source order.
    pub instructions: Vec<ParsedInstruction>,
}

/// Parses the bounded line-oriented Bell-path grammar.
pub fn parse(source: &str) -> Result<ParsedProgram, Vec<Diagnostic>> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err(vec![Diagnostic::error(
            "E001",
            format!("source exceeds the {MAX_SOURCE_BYTES}-byte limit"),
            SourceSpan::new(0, source.len(), 1, 1),
        )]);
    }

    let mut directives = Vec::new();
    let mut labels = Vec::new();
    let mut instructions = Vec::new();
    let mut diagnostics = Vec::new();
    let mut offset = 0;

    for (line_index, raw_line) in source.split_inclusive('\n').enumerate() {
        let line_without_newline = raw_line.trim_end_matches(['\r', '\n']);
        let code = line_without_newline.split(';').next().unwrap_or_default();
        let mut trimmed = code.trim();
        if trimmed.is_empty() {
            offset += raw_line.len();
            continue;
        }

        let leading = code.len() - code.trim_start().len();
        let span = SourceSpan::new(
            offset + leading,
            offset + leading + trimmed.len(),
            line_index + 1,
            leading + 1,
        );

        if let Some((candidate, remainder)) = trimmed.split_once(':') {
            let name = candidate.trim();
            if !valid_identifier(name) {
                diagnostics.push(Diagnostic::error("E010", "invalid label name", span));
                offset += raw_line.len();
                continue;
            }
            let label_column = code.find(name).unwrap_or(leading) + 1;
            labels.push(ParsedLabel {
                name: name.to_owned(),
                instruction_index: instructions.len(),
                span: SourceSpan::new(
                    offset + label_column - 1,
                    offset + label_column - 1 + name.len(),
                    line_index + 1,
                    label_column,
                ),
            });
            trimmed = remainder.trim();
            if trimmed.is_empty() {
                offset += raw_line.len();
                continue;
            }
        }

        let statement_leading = code.find(trimmed).unwrap_or(leading);
        let span = SourceSpan::new(
            offset + statement_leading,
            offset + statement_leading + trimmed.len(),
            line_index + 1,
            statement_leading + 1,
        );

        if let Some(directive) = trimmed.strip_prefix('.') {
            let mut parts = directive.splitn(2, char::is_whitespace);
            let name = parts.next().unwrap_or_default();
            let value = parts.next().unwrap_or_default().trim();
            if name.is_empty() || value.is_empty() {
                diagnostics.push(Diagnostic::error(
                    "E011",
                    "directive requires a name and value",
                    span,
                ));
            } else {
                directives.push(Directive {
                    name: name.to_ascii_lowercase(),
                    value: value.to_owned(),
                    span,
                });
            }
        } else {
            let mut parts = trimmed.splitn(2, char::is_whitespace);
            let mnemonic = parts.next().unwrap_or_default().to_ascii_uppercase();
            let operand_text = parts.next().unwrap_or_default().trim();
            let operands = if operand_text.is_empty() {
                Vec::new()
            } else {
                operand_text
                    .split(',')
                    .map(str::trim)
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>()
            };
            if operands.iter().any(String::is_empty) {
                diagnostics.push(Diagnostic::error(
                    "E012",
                    "empty operand between separators",
                    span,
                ));
            } else {
                let mnemonic_span = SourceSpan::new(
                    span.start,
                    span.start + mnemonic.len(),
                    span.line,
                    span.column,
                );
                instructions.push(ParsedInstruction {
                    mnemonic,
                    operands,
                    span: mnemonic_span,
                });
            }
        }
        offset += raw_line.len();
    }

    if source.is_empty() {
        diagnostics.push(Diagnostic::error(
            "E002",
            "source is empty",
            SourceSpan::new(0, 0, 1, 1),
        ));
    }

    if diagnostics.is_empty() {
        Ok(ParsedProgram {
            directives,
            labels,
            instructions,
        })
    } else {
        Err(diagnostics)
    }
}

fn valid_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    matches!(characters.next(), Some('A'..='Z' | 'a'..='z' | '_'))
        && characters
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_comments_directives_and_typed_text() {
        let parsed =
            parse(".program bell\n; comment\nQALLOC q0\nQCX q0, q1\nHALT\n").expect("valid syntax");
        assert_eq!(parsed.directives[0].name, "program");
        assert_eq!(parsed.instructions[1].operands, ["q0", "q1"]);
        assert_eq!(parsed.instructions[2].span.line, 5);
    }

    #[test]
    fn labels_point_to_the_next_instruction() {
        let parsed = parse("start:\nQALLOC q0\ndone: HALT\n").expect("valid labels");
        assert_eq!(parsed.labels[0].instruction_index, 0);
        assert_eq!(parsed.labels[1].instruction_index, 1);
        assert_eq!(parsed.instructions[1].mnemonic, "HALT");
    }

    #[test]
    fn rejects_empty_operands() {
        let diagnostics = parse("QCX q0,\n").expect_err("invalid operand");
        assert_eq!(diagnostics[0].code, "E012");
    }
}
