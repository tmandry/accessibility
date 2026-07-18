use std::{
    cell::UnsafeCell,
    ffi::c_void,
    marker::{PhantomData, PhantomPinned},
    mem::MaybeUninit,
    ptr::NonNull,
};

use accessibility_sys::AXValueType;
use objc2_core_foundation::{cf_type, CFRange, CFRetained, CGPoint, CGRect, CGSize};

use crate::Error;

pub trait AXValueKind {
    const TYPE: AXValueType;
}

impl AXValueKind for CGPoint {
    const TYPE: AXValueType = AXValueType::CGPoint;
}
impl AXValueKind for CGSize {
    const TYPE: AXValueType = AXValueType::CGSize;
}
impl AXValueKind for CGRect {
    const TYPE: AXValueType = AXValueType::CGRect;
}
impl AXValueKind for CFRange {
    const TYPE: AXValueType = AXValueType::CFRange;
}

pub(crate) fn value_type_name(kind: AXValueType) -> &'static str {
    match kind {
        AXValueType::CGPoint => "CGPoint",
        AXValueType::CGSize => "CGSize",
        AXValueType::CGRect => "CGRect",
        AXValueType::CFRange => "CFRange",
        AXValueType::AXError => "AXError",
        AXValueType::Illegal => "Illegal",
        _ => "<unknown>",
    }
}

/// A typed structure wrapped as an accessibility value.
#[repr(C)]
pub struct AXValue<T: ?Sized> {
    inner: [u8; 0],
    _p: UnsafeCell<PhantomData<(*const UnsafeCell<()>, PhantomPinned)>>,
    _kind: PhantomData<*mut T>,
}

// SAFETY: AXValue is a CoreFoundation type, declared here as a zero-sized type
// with #[repr(C)]. Every instance of it is an `accessibility_sys::AXValue`,
// which it therefore also dereferences to.
cf_type!(
    unsafe impl<T: ?Sized> AXValue<T>: accessibility_sys::AXValue {}
);

impl<T: AXValueKind> AXValue<T> {
    pub fn new(value: &T) -> Result<CFRetained<Self>, Error> {
        let ptr = NonNull::from(value).cast::<c_void>();
        // SAFETY: `ptr` points to a value of the type named by `T::TYPE`.
        let value = unsafe { accessibility_sys::AXValue::new(T::TYPE, ptr) };
        let value = value.expect("AXValueCreate returned NULL");

        // SAFETY: The value was created with `T::TYPE`, so it wraps a `T`.
        Ok(unsafe { CFRetained::cast_unchecked::<Self>(value) })
    }

    pub fn value(&self) -> Result<T, Error> {
        let mut result = MaybeUninit::<T>::uninit();
        // SAFETY: The pointer from MaybeUninit is non-null.
        let ptr = unsafe { NonNull::new_unchecked(result.as_mut_ptr()) }.cast::<c_void>();

        // SAFETY: `ptr` points to enough space to hold a `T`, which is what
        // `T::TYPE` names.
        if unsafe { accessibility_sys::AXValue::value(self, T::TYPE, ptr) } {
            // SAFETY: AXValueGetValue succeeded, so it initialized the value.
            Ok(unsafe { result.assume_init() })
        } else {
            Err(Error::UnexpectedValueType {
                expected: T::TYPE,
                // SAFETY: No preconditions.
                received: unsafe { accessibility_sys::AXValue::r#type(self) },
            })
        }
    }
}
