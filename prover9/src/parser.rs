use std::collections::BTreeMap;

use crate::clause::Clause;
use crate::error::Error;
use crate::literal::{Literal, LiteralList};
use crate::prover::prove_unsat;
use crate::term::Term;

#[derive(Clone)]
pub struct CnfProblem {
    clauses: Vec<Clause>,
}

impl CnfProblem {
    pub fn parse_tptp_cnf(input: &str) -> Result<Self, Error> {
        let mut clauses = Vec::new();
        for body in extract_tptp_cnf_bodies(input)? {
            clauses.push(parse_tptp_clause(&body)?);
        }
        Ok(Self { clauses })
    }

    pub fn clauses(&self) -> &[Clause] {
        &self.clauses
    }

    pub fn prove_unsat(&self) -> Option<Clause> {
        prove_unsat(self.clauses.clone())
    }
}

fn extract_tptp_cnf_bodies(input: &str) -> Result<Vec<String>, Error> {
    let stripped = strip_tptp_comments(input);
    let mut clauses = Vec::new();
    let mut offset = 0;

    while let Some(found) = stripped[offset..].find("cnf(") {
        let start = offset + found + 4;
        let mut depth = 1usize;
        let mut end = start;
        let bytes = stripped.as_bytes();
        while end < stripped.len() && depth > 0 {
            match bytes[end] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            end += 1;
        }
        if depth != 0 {
            return Err(Error::Parse("unterminated cnf(...)".into()));
        }

        let inner = &stripped[start..end - 1];
        let fields = split_top_level(inner, ',');
        if fields.len() < 3 {
            return Err(Error::Parse(format!("bad cnf fields: {inner}")));
        }
        clauses.push(strip_outer_parens(fields[2].trim()).trim().to_owned());
        offset = end;
    }

    Ok(clauses)
}

fn strip_tptp_comments(input: &str) -> String {
    let mut out = String::new();
    for line in input.lines() {
        let line = line.split('%').next().unwrap_or("");
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn split_top_level(input: &str, delimiter: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (index, ch) in input.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ if ch == delimiter && depth == 0 => {
                parts.push(input[start..index].trim());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(input[start..].trim());
    parts
}

fn strip_outer_parens(input: &str) -> &str {
    let mut text = input.trim();
    while text.starts_with('(') && text.ends_with(')') && encloses_whole_text(text) {
        text = text[1..text.len() - 1].trim();
    }
    text
}

fn encloses_whole_text(input: &str) -> bool {
    let mut depth = 0i32;
    for (index, ch) in input.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 && index + 1 != input.len() {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

fn parse_tptp_clause(input: &str) -> Result<Clause, Error> {
    let mut parser = TptpClauseParser::new(input);
    let literals = parser.parse_clause()?;
    if !parser.is_done() {
        return Err(Error::Parse(format!(
            "trailing text in clause body: {}",
            &input[parser.pos..]
        )));
    }
    let term = LiteralList::new(literals)
        .to_term()
        .ok_or_else(|| Error::Parse("empty literal list".into()))?;
    Ok(Clause::from_term(term))
}

struct TptpClauseParser<'a> {
    input: &'a str,
    pos: usize,
    variables: BTreeMap<String, usize>,
}

impl<'a> TptpClauseParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            variables: BTreeMap::new(),
        }
    }

    fn is_done(&mut self) -> bool {
        self.skip_ws();
        self.pos == self.input.len()
    }

    fn parse_clause(&mut self) -> Result<Vec<Literal>, Error> {
        let mut literals = Vec::new();
        loop {
            literals.push(self.parse_literal()?);
            self.skip_ws();
            if self.consume_if('|') {
                continue;
            }
            break;
        }
        Ok(literals)
    }

    fn parse_literal(&mut self) -> Result<Literal, Error> {
        self.skip_ws();
        let negated = self.consume_if('~');
        let atom = self.parse_atom()?;
        Ok(if negated {
            Literal::negative(atom)
        } else {
            Literal::positive(atom)
        })
    }

    fn parse_atom(&mut self) -> Result<Term, Error> {
        let lhs = self.parse_term()?;
        self.skip_ws();
        if self.consume_if('=') {
            let rhs = self.parse_term()?;
            Term::app_vec("=", vec![lhs, rhs])
        } else {
            Ok(lhs)
        }
    }

    fn parse_term(&mut self) -> Result<Term, Error> {
        self.skip_ws();
        let ident = self.parse_ident()?;
        self.skip_ws();
        if self.consume_if('(') {
            let mut args = Vec::new();
            self.skip_ws();
            if !self.consume_if(')') {
                loop {
                    args.push(self.parse_term()?);
                    self.skip_ws();
                    if self.consume_if(',') {
                        continue;
                    }
                    self.expect(')')?;
                    break;
                }
            }
            Term::app_vec(&ident, args)
        } else if ident
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
        {
            let next = self.variables.len();
            let index = *self.variables.entry(ident).or_insert(next);
            Term::var(index)
        } else {
            Term::atom(&ident)
        }
    }

    fn parse_ident(&mut self) -> Result<String, Error> {
        self.skip_ws();
        let start = self.pos;
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }
        if self.pos == start {
            Err(Error::Parse(format!(
                "expected identifier at: {}",
                &self.input[start..]
            )))
        } else {
            Ok(self.input[start..self.pos].to_owned())
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), Error> {
        self.skip_ws();
        if self.consume_if(expected) {
            Ok(())
        } else {
            Err(Error::Parse(format!(
                "expected '{expected}' at: {}",
                &self.input[self.pos..]
            )))
        }
    }

    fn consume_if(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.pos += expected.len_utf8();
            true
        } else {
            false
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn skip_ws(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }
    }
}
