use sm213_parser::parser;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Error,
    Warning,
}

impl From<sm213_parser::Severity> for Severity {
    fn from(value: sm213_parser::Severity) -> Self {
        match value {
            sm213_parser::Severity::Error => Severity::Error,
            sm213_parser::Severity::Warning => Severity::Warning,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ParseError {
    inner: parser::ParseError,
}

#[wasm_bindgen]
impl ParseError {
    pub fn message(&self) -> String {
        self.inner.message.to_string()
    }
    pub fn span(&self) -> Span {
        match self.inner.span_or_pos {
            parser::SpanOrPos::Span(parser::prelude::Span { start, end }) => Span { start, end },
            parser::SpanOrPos::Pos(p) => Span {
                start: p,
                end: p + 1,
            },
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct SemanticError {
    inner: sm213_parser::Diagnostic,
}
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Related {
    message: String,
    pub span: Span,
}

#[wasm_bindgen]
impl Related {
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

#[wasm_bindgen]
impl SemanticError {
    pub fn message(&self) -> String {
        self.inner.message.clone()
    }
    pub fn span(&self) -> Span {
        let parser::prelude::Span { start, end } = self.inner.span;
        Span { start, end }
    }
    pub fn severity(&self) -> Severity {
        let severity = self.inner.severity;
        severity.into()
    }
    pub fn related(&self) -> Option<Related> {
        self.inner.related.as_ref().map(|(message, span)| Related {
            message: message.clone(),
            span: {
                let parser::prelude::Span { start, end } = span;
                Span {
                    start: *start,
                    end: *end,
                }
            },
        })
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ErrDiagnostics {
    parsing_error: Option<ParseError>,
    semantic_errors: Vec<SemanticError>,
}

#[wasm_bindgen]
impl ErrDiagnostics {
    pub fn parsing_error(&self) -> Option<ParseError> {
        self.parsing_error.clone()
    }
    pub fn semantic_errors(&self) -> Vec<SemanticError> {
        self.semantic_errors.clone()
    }
}

#[wasm_bindgen]
pub fn parse_sm213(s: &str) -> ErrDiagnostics {
    let program = match sm213_parser::parse(s) {
        Ok(p) => p,
        Err(e) => {
            return ErrDiagnostics {
                parsing_error: Some(ParseError { inner: e }),
                semantic_errors: Vec::new(),
            }
        }
    };

    let (diagnostics, _second_pass) = sm213_parser::second_pass(&program);

    let diagnostics = diagnostics
        .into_iter()
        .map(|diagnostic| SemanticError { inner: diagnostic })
        .collect::<Vec<_>>();

    ErrDiagnostics {
        parsing_error: None,
        semantic_errors: diagnostics,
    }
}
#[wasm_bindgen]
pub fn reformat(s: &str) -> Option<String> {
    let program = match sm213_parser::parse(s) {
        Ok(p) => p,
        Err(_) => return None,
    };
    Some(program.to_string())
}
