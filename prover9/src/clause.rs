use std::fmt;
use std::ptr;

use prover9_sys as sys;

use crate::literal::LiteralList;
use crate::runtime::with_ladr;
use crate::term::Term;
use crate::types::{DemodDirection, EqualitySide};

pub struct Clause {
    pub(crate) raw: sys::Topform,
}

impl Clause {
    pub fn from_term(term: Term) -> Self {
        let raw = with_ladr(|| unsafe { sys::term_to_clause(term.raw) });
        Self { raw }
    }

    pub fn to_term(&self) -> Term {
        with_ladr(|| Term {
            raw: unsafe { sys::topform_to_term_without_attributes(self.raw) },
        })
    }

    pub fn literals(&self) -> LiteralList {
        with_ladr(|| LiteralList::from_raw_copy(unsafe { sys::prover9_topform_literals(self.raw) }))
    }

    fn literals_raw(&self) -> sys::Literals {
        unsafe { sys::prover9_topform_literals(self.raw) }
    }

    pub fn is_empty(&self) -> bool {
        with_ladr(|| self.literals_raw().is_null())
    }

    pub fn len(&self) -> usize {
        with_ladr(|| unsafe { sys::number_of_literals(self.literals_raw()) as usize })
    }

    pub fn positive_count(&self) -> usize {
        with_ladr(|| unsafe { sys::positive_literals(self.literals_raw()) as usize })
    }

    pub fn negative_count(&self) -> usize {
        with_ladr(|| unsafe { sys::negative_literals(self.literals_raw()) as usize })
    }

    pub fn is_unit(&self) -> bool {
        with_ladr(|| unsafe { sys::unit_clause(self.literals_raw()) != 0 })
    }

    pub fn is_horn(&self) -> bool {
        with_ladr(|| unsafe { sys::horn_clause(self.literals_raw()) != 0 })
    }

    pub fn is_positive(&self) -> bool {
        with_ladr(|| unsafe { sys::positive_clause(self.literals_raw()) != 0 })
    }

    pub fn is_negative(&self) -> bool {
        with_ladr(|| unsafe { sys::negative_clause(self.literals_raw()) != 0 })
    }

    pub fn is_mixed(&self) -> bool {
        with_ladr(|| unsafe { sys::mixed_clause(self.literals_raw()) != 0 })
    }

    pub fn is_ground(&self) -> bool {
        with_ladr(|| unsafe { sys::ground_clause(self.literals_raw()) != 0 })
    }

    pub fn depth(&self) -> usize {
        with_ladr(|| unsafe { sys::clause_depth(self.literals_raw()) as usize })
    }

    pub fn symbol_count(&self) -> usize {
        with_ladr(|| unsafe { sys::clause_symbol_count(self.literals_raw()) as usize })
    }

    pub fn is_positive_equality_unit(&self) -> bool {
        with_ladr(|| unsafe { sys::pos_eq_unit(self.literals_raw()) != 0 })
    }

    pub fn is_negative_equality_unit(&self) -> bool {
        with_ladr(|| unsafe { sys::neg_eq_unit(self.literals_raw()) != 0 })
    }

    pub fn contains_positive_equality(&self) -> bool {
        with_ladr(|| unsafe { sys::contains_pos_eq(self.literals_raw()) != 0 })
    }

    pub fn contains_equality(&self) -> bool {
        with_ladr(|| unsafe { sys::contains_eq(self.literals_raw()) != 0 })
    }

    pub fn only_equality_literals(&self) -> bool {
        with_ladr(|| unsafe { sys::only_eq(self.literals_raw()) != 0 })
    }

    pub fn is_tautology(&self) -> bool {
        with_ladr(|| unsafe { sys::tautology(self.literals_raw()) != 0 })
    }

    pub fn maximal_literal_indices(&self) -> Vec<usize> {
        with_ladr(|| {
            let lits = self.literals_raw();
            let len = unsafe { sys::number_of_literals(lits) as usize };
            (0..len)
                .filter(|&index| unsafe {
                    let lit = sys::ith_literal(lits, index as i32 + 1);
                    !lit.is_null() && sys::maximal_literal(lits, lit, 1) != 0
                })
                .collect()
        })
    }

    pub fn maximal_signed_literal_indices(&self) -> Vec<usize> {
        with_ladr(|| {
            let lits = self.literals_raw();
            let len = unsafe { sys::number_of_literals(lits) as usize };
            (0..len)
                .filter(|&index| unsafe {
                    let lit = sys::ith_literal(lits, index as i32 + 1);
                    !lit.is_null() && sys::maximal_signed_literal(lits, lit, 1) != 0
                })
                .collect()
        })
    }

    pub fn number_of_maximal_literals(&self) -> usize {
        with_ladr(|| unsafe { sys::number_of_maximal_literals(self.literals_raw(), 1) as usize })
    }

    pub fn paramodulate_into(
        &self,
        from_literal_index: usize,
        from_side: EqualitySide,
        into_clause: &Clause,
        into_literal_index: usize,
        into_path: &[usize],
        allow_into_vars: bool,
    ) -> Option<Clause> {
        if from_literal_index >= self.len() || into_literal_index >= into_clause.len() {
            return None;
        }
        if into_path.contains(&0) {
            return None;
        }

        let path: Vec<i32> = into_path.iter().map(|&index| index as i32).collect();
        with_ladr(|| {
            let raw = unsafe {
                sys::prover9_para_pos(
                    self.raw,
                    from_literal_index as i32 + 1,
                    from_side.into_raw(),
                    into_clause.raw,
                    into_literal_index as i32 + 1,
                    path.as_ptr(),
                    path.len() as i32,
                    if allow_into_vars { 1 } else { 0 },
                )
            };
            if raw.is_null() {
                None
            } else {
                Some(Clause { raw })
            }
        })
    }

    pub fn resolve_with(
        &self,
        self_literal_index: usize,
        other: &Clause,
        other_literal_index: usize,
    ) -> Option<Clause> {
        self.resolve_with_inner(self_literal_index, other, other_literal_index, false)
    }

    pub fn resolve_with_flipped_other(
        &self,
        self_literal_index: usize,
        other: &Clause,
        other_literal_index: usize,
    ) -> Option<Clause> {
        self.resolve_with_inner(self_literal_index, other, other_literal_index, true)
    }

    pub fn resolve_with_xx(&self, literal_index: usize) -> Option<Clause> {
        if literal_index >= self.len() {
            return None;
        }

        with_ladr(|| {
            let raw = unsafe { sys::xx_resolve2(self.raw, literal_index as i32 + 1, 1) };
            if raw.is_null() {
                None
            } else {
                Some(Clause { raw })
            }
        })
    }

    pub fn factor_with(
        &self,
        first_literal_index: usize,
        second_literal_index: usize,
    ) -> Option<Clause> {
        if first_literal_index >= self.len() || second_literal_index >= self.len() {
            return None;
        }

        with_ladr(|| {
            let raw = unsafe {
                sys::prover9_factor2(
                    self.raw,
                    first_literal_index as i32 + 1,
                    second_literal_index as i32 + 1,
                    1,
                )
            };
            if raw.is_null() {
                None
            } else {
                Some(Clause { raw })
            }
        })
    }

    pub fn merge_duplicate_literals(&self) -> Clause {
        with_ladr(|| {
            let raw = unsafe { sys::copy_clause(self.raw) };
            unsafe {
                sys::merge_literals(raw);
            }
            Clause { raw }
        })
    }

    pub fn simplify_basic(&self) -> Option<Clause> {
        let clause = self.merge_duplicate_literals();
        if clause.is_tautology() {
            None
        } else {
            Some(clause)
        }
    }

    pub fn renumber_variables(&mut self, max_vars: usize) {
        with_ladr(|| unsafe {
            sys::renumber_variables(self.raw, max_vars as i32);
        });
    }

    pub fn demodulate_once_with(
        &self,
        demodulator: &Clause,
        direction: DemodDirection,
    ) -> Option<Clause> {
        with_ladr(|| unsafe {
            if sys::prover9_can_demodulate(self.raw, demodulator.raw, direction.into_raw(), 0) == 0
            {
                return None;
            }

            let result = Clause {
                raw: sys::copy_clause(self.raw),
            };
            let mut from_pos = ptr::null_mut();
            let mut into_pos = ptr::null_mut();
            sys::demod1(
                result.raw,
                demodulator.raw,
                direction.into_raw(),
                &mut from_pos,
                &mut into_pos,
                0,
            );
            sys::zap_ilist(from_pos);
            sys::zap_ilist(into_pos);
            Some(result)
        })
    }

    pub fn subsumes(&self, other: &Clause) -> bool {
        with_ladr(|| unsafe { sys::subsumes(self.raw, other.raw) != 0 })
    }

    pub fn subsumes_bt(&self, other: &Clause) -> bool {
        with_ladr(|| unsafe { sys::subsumes_bt(self.raw, other.raw) != 0 })
    }

    fn resolve_with_inner(
        &self,
        self_literal_index: usize,
        other: &Clause,
        other_literal_index: usize,
        flip_other: bool,
    ) -> Option<Clause> {
        if self_literal_index >= self.len() || other_literal_index >= other.len() {
            return None;
        }

        with_ladr(|| {
            let n1 = self_literal_index as i32 + 1;
            let n2 = other_literal_index as i32 + 1;
            let n2 = if flip_other { -n2 } else { n2 };
            let raw = unsafe { sys::resolve2(self.raw, n1, other.raw, n2, 1) };
            if raw.is_null() {
                None
            } else {
                Some(Clause { raw })
            }
        })
    }
}

impl Clone for Clause {
    fn clone(&self) -> Self {
        with_ladr(|| Self {
            raw: unsafe { sys::copy_clause(self.raw) },
        })
    }
}

impl fmt::Debug for Clause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Clause")
            .field(&self.to_term().to_string())
            .finish()
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_term().to_string())
    }
}

impl Drop for Clause {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            with_ladr(|| unsafe {
                sys::zap_topform(self.raw);
            });
        }
    }
}
