use std::ffi::CString;
use std::sync::{Mutex, Once};

use prover9_sys as sys;

use crate::error::Error;
use crate::types::OrderMethod;

static INIT: Once = Once::new();
static LADR_LOCK: Mutex<()> = Mutex::new(());

fn ensure_init_locked() {
    INIT.call_once(|| unsafe {
        sys::prover9_init();
    });
}

pub(crate) fn with_ladr<T>(f: impl FnOnce() -> T) -> T {
    let _guard = LADR_LOCK.lock().unwrap();
    ensure_init_locked();
    f()
}

pub fn set_order_method(method: OrderMethod) {
    with_ladr(|| unsafe {
        sys::assign_order_method(method.into_raw());
    });
}

pub fn set_symbol_precedence(name: &str, arity: usize, precedence: i32) -> Result<(), Error> {
    let c_name = CString::new(name).map_err(|_| Error::InteriorNul)?;
    with_ladr(|| unsafe {
        let symnum = sys::str_to_sn(c_name.as_ptr().cast_mut(), arity as i32);
        sys::set_lex_val(symnum, precedence);
    });
    Ok(())
}
