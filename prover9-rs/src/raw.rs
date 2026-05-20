use std::collections::BTreeSet;
use std::ffi::CStr;

use prover9_sys as sys;

use crate::error::Error;
use crate::runtime::with_ladr;
use crate::term::Term;
use crate::types::Substitution;

pub(crate) unsafe fn copy_term(raw: sys::Term) -> sys::Term {
    if sys::prover9_term_is_variable(raw) != 0 {
        return sys::get_variable_term(sys::prover9_term_var_num(raw));
    }
    let arity = sys::prover9_term_arity(raw);
    let symbol = CStr::from_ptr(sys::prover9_term_symbol(raw));
    let copy = sys::get_rigid_term(symbol.as_ptr().cast_mut(), arity);
    for index in 0..arity {
        let child = sys::prover9_term_arg(raw, index);
        sys::prover9_term_set_arg(copy, index, copy_term(child));
    }
    copy
}

pub(crate) fn ensure_var_range(var: usize) -> Result<(), Error> {
    let max = with_ladr(|| unsafe { sys::prover9_max_vars() as usize });
    if var >= max {
        Err(Error::TooManyVariables { var, max })
    } else {
        Ok(())
    }
}

pub(crate) fn max_vars() -> usize {
    with_ladr(|| unsafe { sys::prover9_max_vars() as usize })
}

pub(crate) fn collect_substitution(raw: sys::Term, context: sys::Context) -> Vec<Substitution> {
    let mut vars = BTreeSet::new();
    collect_vars(raw, &mut vars);
    vars.into_iter()
        .filter_map(|var| unsafe {
            let bound = sys::prover9_context_binding_term(context, var as i32);
            if bound.is_null() {
                return None;
            }
            let bound_context = sys::prover9_context_binding_context(context, var as i32);
            let term = Term {
                raw: sys::apply(bound, bound_context),
            };
            Some(Substitution { var, term })
        })
        .collect()
}

fn collect_vars(raw: sys::Term, vars: &mut BTreeSet<usize>) {
    if unsafe { sys::prover9_term_is_variable(raw) != 0 } {
        vars.insert(unsafe { sys::prover9_term_var_num(raw) as usize });
        return;
    }
    let arity = unsafe { sys::prover9_term_arity(raw) };
    for index in 0..arity {
        let child = unsafe { sys::prover9_term_arg(raw, index) };
        collect_vars(child, vars);
    }
}
