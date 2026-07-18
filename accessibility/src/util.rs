use std::{mem::MaybeUninit, ptr::NonNull};

use objc2_core_foundation::{CFRetained, Type};

use crate::{AXError, Error};

pub(crate) unsafe fn ax_call<F, V>(f: F) -> Result<V, AXError>
where
    F: FnOnce(NonNull<V>) -> accessibility_sys::AXError,
{
    let mut result = MaybeUninit::uninit();
    let err = (f)(NonNull::new_unchecked(result.as_mut_ptr()));

    match AXError::from_raw(err) {
        Some(err) => Err(err),
        None => Ok(result.assume_init()),
    }
}

pub(crate) unsafe fn ax_call_void<F>(f: F) -> Result<(), AXError>
where
    F: FnOnce() -> accessibility_sys::AXError,
{
    match AXError::from_raw((f)()) {
        Some(err) => Err(err),
        None => Ok(()),
    }
}

/// Calls a create-rule function and retains its result.
pub(crate) unsafe fn ax_call_retained<F, T>(f: F) -> Result<CFRetained<T>, Error>
where
    F: FnOnce(NonNull<*const T>) -> accessibility_sys::AXError,
    T: Type,
{
    let ptr: *const T = ax_call(f).map_err(Error::Ax)?;
    // The system should never hand us a NULL value along with a success code.
    let ptr = NonNull::new(ptr.cast_mut()).ok_or(Error::Ax(AXError::Failure))?;

    Ok(CFRetained::from_raw(ptr))
}
