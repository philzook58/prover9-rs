use std::fmt;
use std::ptr;

use prover9_sys as sys;

use crate::clause::Clause;
use crate::error::Error;
use crate::runtime::with_ladr;
use crate::term::Term;

const ORIENTED: i32 = 1;
const LEX_DEP_LR: i32 = 2;
const LEX_DEP_RL: i32 = 3;
const LEX_DEP_BOTH: i32 = 4;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RewriteRuleKind {
    Oriented,
    LexDependentLeftToRight,
    LexDependentRightToLeft,
    LexDependentBoth,
}

pub struct RewriteSet {
    raw: sys::Mindex,
    rules: Vec<Clause>,
}

impl RewriteSet {
    pub fn new() -> Self {
        with_ladr(|| Self {
            raw: unsafe { sys::prover9_rewrite_index_new() },
            rules: Vec::new(),
        })
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn rules(&self) -> &[Clause] {
        &self.rules
    }

    pub fn insert_rule(&mut self, rule: &Clause) -> Result<RewriteRuleKind, Error> {
        let owned = rule.clone();
        let rendered = owned.to_string();
        with_ladr(|| {
            let raw_kind = unsafe { sys::prover9_rewrite_rule_type(owned.raw) };
            let kind = RewriteRuleKind::from_raw(raw_kind)
                .ok_or_else(|| Error::InvalidRewriteRule(rendered.clone()))?;
            unsafe {
                sys::prover9_rewrite_index_insert(owned.raw, raw_kind, self.raw);
            }
            self.rules.push(owned);
            Ok(kind)
        })
    }

    pub fn rewrite_term(&self, term: &Term) -> Term {
        let owned = term.clone();
        with_ladr(|| Term {
            raw: unsafe { sys::prover9_rewrite_term(owned.into_raw(), self.raw, 0) },
        })
    }

    pub fn rewrite_clause(&self, clause: &Clause) -> Clause {
        with_ladr(|| Clause {
            raw: unsafe { sys::prover9_rewrite_clause(clause.raw, self.raw, 0) },
        })
    }
}

impl RewriteRuleKind {
    fn from_raw(raw: i32) -> Option<Self> {
        match raw {
            ORIENTED => Some(Self::Oriented),
            LEX_DEP_LR => Some(Self::LexDependentLeftToRight),
            LEX_DEP_RL => Some(Self::LexDependentRightToLeft),
            LEX_DEP_BOTH => Some(Self::LexDependentBoth),
            _ => None,
        }
    }
}

impl Default for RewriteSet {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for RewriteSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RewriteSet")
            .field("rule_count", &self.len())
            .finish()
    }
}

impl Drop for RewriteSet {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            with_ladr(|| unsafe {
                sys::prover9_rewrite_index_destroy(self.raw);
            });
            self.raw = ptr::null_mut();
        }
    }
}
