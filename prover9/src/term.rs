use std::ffi::{CStr, CString};
use std::fmt;
use std::marker::PhantomData;
use std::ptr;

use prover9_sys as sys;

use crate::error::Error;
use crate::raw::{collect_substitution, copy_term, ensure_var_range};
use crate::runtime::with_ladr;
use crate::types::{TermOrdering, Unification};

pub struct Term {
    pub(crate) raw: sys::Term,
}

#[derive(Copy, Clone)]
pub struct TermRef<'a> {
    pub(crate) raw: sys::Term,
    pub(crate) _marker: PhantomData<&'a Term>,
}

pub struct WildDiscrimIndex {
    root: sys::Discrim,
    objects: Vec<sys::Term>,
}

impl Term {
    pub fn parse(input: &str) -> Result<Self, Error> {
        let c_input = CString::new(input).map_err(|_| Error::InteriorNul)?;
        Ok(with_ladr(|| Self {
            raw: unsafe { sys::parse_term_from_string(c_input.as_ptr().cast_mut()) },
        }))
    }

    pub fn var(var: usize) -> Result<Self, Error> {
        ensure_var_range(var)?;
        Ok(with_ladr(|| Self {
            raw: unsafe { sys::get_variable_term(var as i32) },
        }))
    }

    pub fn atom(name: &str) -> Result<Self, Error> {
        Self::app_raw(name, [])
    }

    pub fn app<const N: usize>(name: &str, children: [Term; N]) -> Result<Self, Error> {
        Self::app_raw(name, children)
    }

    pub fn app_vec(name: &str, children: Vec<Term>) -> Result<Self, Error> {
        let c_name = CString::new(name).map_err(|_| Error::InteriorNul)?;
        Ok(with_ladr(|| {
            let raw =
                unsafe { sys::get_rigid_term(c_name.as_ptr().cast_mut(), children.len() as i32) };
            for (index, child) in children.into_iter().enumerate() {
                unsafe {
                    sys::prover9_term_set_arg(raw, index as i32, child.into_raw());
                }
            }
            Self { raw }
        }))
    }

    fn app_raw<const N: usize>(name: &str, children: [Term; N]) -> Result<Self, Error> {
        let c_name = CString::new(name).map_err(|_| Error::InteriorNul)?;
        Ok(with_ladr(|| {
            let raw = unsafe { sys::get_rigid_term(c_name.as_ptr().cast_mut(), N as i32) };
            for (index, child) in children.into_iter().enumerate() {
                unsafe {
                    sys::prover9_term_set_arg(raw, index as i32, child.into_raw());
                }
            }
            Self { raw }
        }))
    }

    pub fn as_ref(&self) -> TermRef<'_> {
        TermRef {
            raw: self.raw,
            _marker: PhantomData,
        }
    }

    pub fn children(&self) -> Vec<TermRef<'_>> {
        self.as_ref().children()
    }

    pub fn is_ground(&self) -> bool {
        with_ladr(|| unsafe { sys::ground_term(self.raw) != 0 })
    }

    pub fn depth(&self) -> usize {
        with_ladr(|| unsafe { sys::term_depth(self.raw) as usize })
    }

    pub fn symbol_count(&self) -> usize {
        with_ladr(|| unsafe { sys::symbol_count(self.raw) as usize })
    }

    pub fn occurs_in(&self, other: &Term) -> bool {
        with_ladr(|| unsafe { sys::occurs_in(self.raw, other.raw) != 0 })
    }

    pub fn unify(&self, other: &Term) -> Option<Unification> {
        with_ladr(|| unsafe {
            let left_ctx = sys::get_context();
            let right_ctx = sys::get_context();
            let mut trail: sys::Trail = ptr::null_mut();
            let ok = sys::unify(self.raw, left_ctx, other.raw, right_ctx, &mut trail);
            if ok == 0 {
                sys::free_context(left_ctx);
                sys::free_context(right_ctx);
                return None;
            }

            let left = Term {
                raw: sys::apply(self.raw, left_ctx),
            };
            let right = Term {
                raw: sys::apply(other.raw, right_ctx),
            };
            let left_substitution = collect_substitution(self.raw, left_ctx);
            let right_substitution = collect_substitution(other.raw, right_ctx);

            sys::undo_subst(trail);
            sys::free_context(left_ctx);
            sys::free_context(right_ctx);

            Some(Unification {
                left,
                right,
                left_substitution,
                right_substitution,
            })
        })
    }

    pub fn matches(&self, target: &Term) -> Option<Vec<crate::Substitution>> {
        with_ladr(|| unsafe {
            let context = sys::get_context();
            let mut trail: sys::Trail = ptr::null_mut();
            let ok = sys::prover9_match(self.raw, context, target.raw, &mut trail);
            if ok == 0 {
                sys::free_context(context);
                return None;
            }

            let substitution = collect_substitution(self.raw, context);
            sys::undo_subst(trail);
            sys::free_context(context);
            Some(substitution)
        })
    }

    pub fn order(&self, other: &Term) -> TermOrdering {
        with_ladr(|| unsafe { TermOrdering::from_raw(sys::term_order(self.raw, other.raw)) })
    }

    pub fn lrpo_gt(&self, other: &Term) -> bool {
        with_ladr(|| unsafe { sys::lrpo(self.raw, other.raw, 0) != 0 })
    }

    pub fn kbo_gt(&self, other: &Term) -> bool {
        with_ladr(|| unsafe { sys::kbo(self.raw, other.raw, 0) != 0 })
    }

    pub fn greater_than_current_ordering(&self, other: &Term) -> bool {
        with_ladr(|| unsafe { sys::term_greater(self.raw, other.raw, 0) != 0 })
    }

    pub(crate) fn from_ref(term: TermRef<'_>) -> Self {
        with_ladr(|| Self::from_ref_unlocked(term))
    }

    pub(crate) fn from_ref_unlocked(term: TermRef<'_>) -> Self {
        Self {
            raw: unsafe { copy_term(term.raw) },
        }
    }

    pub(crate) fn into_raw(mut self) -> sys::Term {
        let raw = self.raw;
        self.raw = ptr::null_mut();
        raw
    }
}

impl WildDiscrimIndex {
    pub fn new() -> Self {
        with_ladr(|| Self {
            root: unsafe { sys::discrim_init() },
            objects: Vec::new(),
        })
    }

    pub fn insert(&mut self, term: &Term) {
        with_ladr(|| unsafe {
            let stored = copy_term(term.raw);
            sys::discrim_wild_update(stored, self.root, stored.cast(), sys::Indexop_INSERT);
            self.objects.push(stored);
        });
    }

    pub fn retrieve_generalizations(&self, query: &Term) -> Vec<Term> {
        with_ladr(|| unsafe {
            let mut results = Vec::new();
            let mut pos: sys::Discrim_pos = ptr::null_mut();
            let mut object = sys::discrim_wild_retrieve_first(query.raw, self.root, &mut pos);

            while !object.is_null() {
                let candidate: sys::Term = object.cast();
                let context = sys::get_context();
                let mut trail: sys::Trail = ptr::null_mut();
                if sys::prover9_match(candidate, context, query.raw, &mut trail) != 0 {
                    results.push(Term {
                        raw: copy_term(candidate),
                    });
                    sys::undo_subst(trail);
                }
                sys::free_context(context);
                object = sys::discrim_wild_retrieve_next(pos);
            }

            results
        })
    }
}

impl Default for WildDiscrimIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> TermRef<'a> {
    pub fn is_variable(self) -> bool {
        with_ladr(|| unsafe { sys::prover9_term_is_variable(self.raw) != 0 })
    }

    pub fn var_num(self) -> Option<usize> {
        with_ladr(|| {
            (unsafe { sys::prover9_term_is_variable(self.raw) != 0 })
                .then(|| unsafe { sys::prover9_term_var_num(self.raw) as usize })
        })
    }

    pub fn symbol(self) -> Option<&'a str> {
        with_ladr(|| {
            let ptr = unsafe { sys::prover9_term_symbol(self.raw) };
            if ptr.is_null() {
                None
            } else {
                Some(
                    unsafe { CStr::from_ptr(ptr) }
                        .to_str()
                        .expect("symbol should be utf-8"),
                )
            }
        })
    }

    pub fn arity(self) -> usize {
        with_ladr(|| unsafe { sys::prover9_term_arity(self.raw) as usize })
    }

    pub fn children(self) -> Vec<TermRef<'a>> {
        with_ladr(|| {
            let arity = unsafe { sys::prover9_term_arity(self.raw) as usize };
            (0..arity)
                .map(|index| TermRef {
                    raw: unsafe { sys::prover9_term_arg(self.raw, index as i32) },
                    _marker: PhantomData,
                })
                .collect()
        })
    }

    pub fn to_owned(self) -> Term {
        with_ladr(|| Term::from_ref_unlocked(self))
    }

    pub fn is_ground(self) -> bool {
        with_ladr(|| unsafe { sys::ground_term(self.raw) != 0 })
    }

    pub fn depth(self) -> usize {
        with_ladr(|| unsafe { sys::term_depth(self.raw) as usize })
    }

    pub fn symbol_count(self) -> usize {
        with_ladr(|| unsafe { sys::symbol_count(self.raw) as usize })
    }

    fn render(self) -> String {
        with_ladr(|| unsafe {
            let ptr = sys::term_to_string(self.raw);
            let out = CStr::from_ptr(ptr)
                .to_str()
                .expect("term should be utf-8")
                .to_owned();
            libc::free(ptr.cast());
            out
        })
    }

    pub fn order(self, other: TermRef<'_>) -> TermOrdering {
        with_ladr(|| unsafe { TermOrdering::from_raw(sys::term_order(self.raw, other.raw)) })
    }

    pub fn occurs_in(self, other: TermRef<'_>) -> bool {
        with_ladr(|| unsafe { sys::occurs_in(self.raw, other.raw) != 0 })
    }
}

impl fmt::Display for TermRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str((*self).render().as_str())
    }
}

impl Clone for Term {
    fn clone(&self) -> Self {
        Term::from_ref(self.as_ref())
    }
}

impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for Term {}

impl fmt::Debug for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Term").field(&self.to_string()).finish()
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_ref().to_string())
    }
}

impl Drop for Term {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            with_ladr(|| unsafe {
                sys::zap_term(self.raw);
            });
        }
    }
}

impl Drop for WildDiscrimIndex {
    fn drop(&mut self) {
        with_ladr(|| unsafe {
            if !self.root.is_null() {
                sys::destroy_discrim_tree(self.root);
                self.root = ptr::null_mut();
            }
            for raw in self.objects.drain(..) {
                sys::zap_term(raw);
            }
        });
    }
}
