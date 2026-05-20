use std::fmt;
use std::marker::PhantomData;
use std::ptr;

use prover9_sys as sys;

use crate::runtime::with_ladr;
use crate::term::{Term, TermRef};

pub struct Literal {
    pub(crate) raw: sys::Literals,
}

pub struct LiteralList {
    pub(crate) raw: sys::Literals,
}

impl Literal {
    pub fn positive(atom: Term) -> Self {
        with_ladr(|| Self {
            raw: unsafe { sys::new_literal(1, atom.into_raw()) },
        })
    }

    pub fn negative(atom: Term) -> Self {
        with_ladr(|| Self {
            raw: unsafe { sys::new_literal(0, atom.into_raw()) },
        })
    }

    pub fn is_positive(&self) -> bool {
        with_ladr(|| unsafe { sys::prover9_literal_sign(self.raw) != 0 })
    }

    pub fn atom(&self) -> TermRef<'_> {
        with_ladr(|| TermRef {
            raw: unsafe { sys::prover9_literal_atom(self.raw) },
            _marker: PhantomData,
        })
    }

    pub fn is_positive_equality(&self) -> bool {
        with_ladr(|| unsafe { sys::pos_eq(self.raw) != 0 })
    }

    pub fn is_negative_equality(&self) -> bool {
        with_ladr(|| unsafe { sys::neg_eq(self.raw) != 0 })
    }

    pub fn equality_sides(&self) -> Option<(Term, Term)> {
        if !self.is_positive_equality() && !self.is_negative_equality() {
            return None;
        }

        let atom = self.atom();
        let children = atom.children();
        if children.len() != 2 {
            return None;
        }
        Some((children[0].to_owned(), children[1].to_owned()))
    }

    pub fn to_term(&self) -> Term {
        with_ladr(|| Self::to_term_unlocked(self))
    }

    pub(crate) fn to_term_unlocked(&self) -> Term {
        Term {
            raw: unsafe { sys::literal_to_term(self.raw) },
        }
    }

    pub(crate) fn into_raw(mut self) -> sys::Literals {
        let raw = self.raw;
        self.raw = ptr::null_mut();
        raw
    }
}

impl LiteralList {
    pub fn new(literals: Vec<Literal>) -> Self {
        with_ladr(|| {
            let mut raw: sys::Literals = ptr::null_mut();
            for literal in literals {
                raw = unsafe { sys::append_literal(raw, literal.into_raw()) };
            }
            Self { raw }
        })
    }

    pub fn len(&self) -> usize {
        with_ladr(|| unsafe { sys::number_of_literals(self.raw) as usize })
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_null()
    }

    pub fn positive_count(&self) -> usize {
        with_ladr(|| unsafe { sys::positive_literals(self.raw) as usize })
    }

    pub fn negative_count(&self) -> usize {
        with_ladr(|| unsafe { sys::negative_literals(self.raw) as usize })
    }

    pub fn get(&self, index: usize) -> Option<Literal> {
        with_ladr(|| {
            let raw = unsafe { sys::ith_literal(self.raw, index as i32 + 1) };
            if raw.is_null() {
                None
            } else {
                Some(Literal {
                    raw: unsafe { sys::copy_literal(raw) },
                })
            }
        })
    }

    pub fn to_term(&self) -> Option<Term> {
        if self.raw.is_null() {
            None
        } else {
            Some(with_ladr(|| Term {
                raw: unsafe { sys::literals_to_term(self.raw) },
            }))
        }
    }

    pub(crate) fn from_raw_copy(raw: sys::Literals) -> Self {
        Self {
            raw: unsafe { sys::copy_literals(raw) },
        }
    }
}

impl Clone for Literal {
    fn clone(&self) -> Self {
        with_ladr(|| Self {
            raw: unsafe { sys::copy_literal(self.raw) },
        })
    }
}

impl fmt::Debug for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Literal")
            .field(&self.to_term().to_string())
            .finish()
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_term().to_string())
    }
}

impl Drop for Literal {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            with_ladr(|| unsafe {
                sys::zap_literal(self.raw);
            });
        }
    }
}

impl Clone for LiteralList {
    fn clone(&self) -> Self {
        with_ladr(|| Self {
            raw: unsafe { sys::copy_literals(self.raw) },
        })
    }
}

impl fmt::Debug for LiteralList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rendered: Vec<_> = (0..self.len())
            .filter_map(|i| self.get(i))
            .map(|lit| lit.to_string())
            .collect();
        f.debug_tuple("LiteralList").field(&rendered).finish()
    }
}

impl Drop for LiteralList {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            with_ladr(|| unsafe {
                sys::zap_literals(self.raw);
            });
        }
    }
}
