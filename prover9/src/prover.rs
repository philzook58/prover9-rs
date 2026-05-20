use std::collections::HashSet;

use crate::clause::Clause;
use crate::raw::max_vars;

pub(crate) fn prove_unsat(initial: Vec<Clause>) -> Option<Clause> {
    let mut processed: Vec<Clause> = Vec::new();
    let mut unprocessed = initial;
    let mut seen = HashSet::new();

    while !unprocessed.is_empty() {
        let given_index = select_given_clause(&unprocessed);
        let given = unprocessed.swap_remove(given_index);
        let given = match given.simplify_basic() {
            Some(clause) => clause,
            None => continue,
        };
        let given = canonicalize_clause(given);
        if processed.iter().any(|clause| clause.subsumes(&given)) {
            continue;
        }

        let given_key = given.to_string();
        if !seen.insert(given_key) {
            continue;
        }
        if given.is_empty() {
            return Some(given);
        }

        let mut generated: Vec<Clause> = Vec::new();
        for i in 0..given.len() {
            for j in (i + 1)..given.len() {
                if let Some(factor) = given.factor_with(i, j) {
                    generated.push(factor);
                }
            }
        }
        for lit_index in 0..given.len() {
            if let Some(resolvent) = given.resolve_with_xx(lit_index) {
                generated.push(resolvent);
            }
            for other in &processed {
                for other_lit_index in 0..other.len() {
                    if let Some(resolvent) = given.resolve_with(lit_index, other, other_lit_index) {
                        generated.push(resolvent);
                    }
                    if let Some(other_lit) = other.literals().get(other_lit_index) {
                        if other_lit.is_positive_equality() || other_lit.is_negative_equality() {
                            if let Some(resolvent) =
                                given.resolve_with_flipped_other(lit_index, other, other_lit_index)
                            {
                                generated.push(resolvent);
                            }
                        }
                    }
                }
            }
        }

        processed.push(given);

        for clause in generated {
            let clause = match clause.simplify_basic() {
                Some(clause) => clause,
                None => continue,
            };
            let clause = canonicalize_clause(clause);
            if clause.is_empty() {
                return Some(clause);
            }
            let clause_key = clause.to_string();
            if seen.contains(&clause_key) {
                continue;
            }
            if processed.iter().any(|existing| existing.subsumes(&clause)) {
                continue;
            }
            if unprocessed
                .iter()
                .any(|existing| existing.subsumes(&clause))
            {
                continue;
            }
            unprocessed.retain(|existing| !clause.subsumes(existing));
            unprocessed.push(clause);
        }
    }

    None
}

fn select_given_clause(clauses: &[Clause]) -> usize {
    clauses
        .iter()
        .enumerate()
        .min_by_key(|(_, clause)| {
            (
                clause.len(),
                clause.symbol_count(),
                clause.to_string().len(),
            )
        })
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn canonicalize_clause(mut clause: Clause) -> Clause {
    clause.renumber_variables(max_vars());
    clause
}
