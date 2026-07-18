use std::{
    cell::UnsafeCell,
    marker::{PhantomData, PhantomPinned},
    thread,
    time::{Duration, Instant},
};

use accessibility_sys::pid_t;
use objc2_app_kit::NSRunningApplication;
use objc2_core_foundation::{cf_type, CFArray, CFRetained, CFString, CFTypeID, ConcreteType};
use objc2_foundation::NSString;

use crate::{
    util::{ax_call, ax_call_retained, ax_call_void},
    AXAttribute, AXAttributeValue, Error,
};

/// An element in the macOS accessibility hierarchy.
#[repr(C)]
pub struct AXUIElement {
    inner: [u8; 0],
    _p: UnsafeCell<PhantomData<(*const UnsafeCell<()>, PhantomPinned)>>,
}

// SAFETY: AXUIElement is a CoreFoundation type, declared here as an opaque type
// with #[repr(C)], in the way objc2 declares them (pending `extern type`).
cf_type!(
    unsafe impl AXUIElement {}
);

// SAFETY: Instances of this type are `accessibility_sys::AXUIElement`s, so they
// share its type id.
unsafe impl ConcreteType for AXUIElement {
    fn type_id() -> CFTypeID {
        accessibility_sys::AXUIElement::type_id()
    }
}

impl AXUIElement {
    /// Returns this element as the raw API type.
    pub fn as_sys(&self) -> &accessibility_sys::AXUIElement {
        let ptr: *const Self = self;

        // SAFETY: Both types are opaque declarations of the same CoreFoundation
        // object, so a reference to one is a reference to the other.
        unsafe { &*ptr.cast::<accessibility_sys::AXUIElement>() }
    }

    pub fn system_wide() -> CFRetained<Self> {
        // SAFETY: No preconditions.
        let element = unsafe { accessibility_sys::AXUIElement::new_system_wide() };

        // SAFETY: The element came from AXUIElementCreateSystemWide.
        unsafe { CFRetained::cast_unchecked::<Self>(element) }
    }

    pub fn application(pid: pid_t) -> CFRetained<Self> {
        // SAFETY: No preconditions.
        let element = unsafe { accessibility_sys::AXUIElement::new_application(pid) };

        // SAFETY: The element came from AXUIElementCreateApplication.
        unsafe { CFRetained::cast_unchecked::<Self>(element) }
    }

    pub fn application_with_bundle(bundle_id: &str) -> Result<CFRetained<Self>, Error> {
        let bundle_id = NSString::from_str(bundle_id);
        let apps = NSRunningApplication::runningApplicationsWithBundleIdentifier(&bundle_id);
        let app = apps.firstObject().ok_or(Error::NotFound)?;

        Ok(Self::application(app.processIdentifier()))
    }

    pub fn application_with_bundle_timeout(
        bundle_id: &str,
        timeout: Duration,
    ) -> Result<CFRetained<Self>, Error> {
        let deadline = Instant::now() + timeout;

        loop {
            match Self::application_with_bundle(bundle_id) {
                Ok(result) => return Ok(result),
                Err(e) => {
                    let now = Instant::now();

                    if now >= deadline {
                        return Err(e);
                    } else {
                        let time_left = deadline.saturating_duration_since(now);
                        thread::sleep(std::cmp::min(time_left, Duration::from_millis(250)));
                    }
                }
            }
        }
    }

    pub fn attribute_names(&self) -> Result<CFRetained<CFArray<CFString>>, Error> {
        // SAFETY: The out parameter is passed on from `ax_call_retained`.
        let names: CFRetained<CFArray> =
            unsafe { ax_call_retained(|names| self.as_sys().copy_attribute_names(names)) }?;

        // SAFETY: AXUIElementCopyAttributeNames returns an array of strings.
        Ok(unsafe { CFRetained::cast_unchecked::<CFArray<CFString>>(names) })
    }

    pub fn attribute<T: AXAttributeValue>(
        &self,
        attribute: &AXAttribute<T>,
    ) -> Result<CFRetained<T>, Error> {
        // SAFETY: The out parameter is passed on from `ax_call_retained`.
        let value = unsafe {
            ax_call_retained(|value| {
                self.as_sys()
                    .copy_attribute_value(attribute.as_CFString(), value)
            })
        }?;

        T::downcast(value)
    }

    pub fn set_attribute<T: AXAttributeValue>(
        &self,
        attribute: &AXAttribute<T>,
        value: &T,
    ) -> Result<(), Error> {
        // SAFETY: No preconditions.
        unsafe {
            ax_call_void(|| {
                self.as_sys()
                    .set_attribute_value(attribute.as_CFString(), value.as_ref())
            })
        }
        .map_err(Error::Ax)
    }

    pub fn is_settable<T: AXAttributeValue>(
        &self,
        attribute: &AXAttribute<T>,
    ) -> Result<bool, Error> {
        // SAFETY: The out parameter is passed on from `ax_call`.
        let settable = unsafe {
            ax_call(|settable| {
                self.as_sys()
                    .is_attribute_settable(attribute.as_CFString(), settable)
            })
        }
        .map_err(Error::Ax)?;

        Ok(settable != 0)
    }

    pub fn action_names(&self) -> Result<CFRetained<CFArray<CFString>>, Error> {
        // SAFETY: The out parameter is passed on from `ax_call_retained`.
        let names: CFRetained<CFArray> =
            unsafe { ax_call_retained(|names| self.as_sys().copy_action_names(names)) }?;

        // SAFETY: AXUIElementCopyActionNames returns an array of strings.
        Ok(unsafe { CFRetained::cast_unchecked::<CFArray<CFString>>(names) })
    }

    pub fn perform_action(&self, name: &CFString) -> Result<(), Error> {
        // SAFETY: No preconditions.
        unsafe { ax_call_void(|| self.as_sys().perform_action(name)) }.map_err(Error::Ax)
    }

    pub fn set_messaging_timeout(&self, timeout: f32) -> Result<(), Error> {
        // SAFETY: No preconditions.
        unsafe { ax_call_void(|| self.as_sys().set_messaging_timeout(timeout)) }.map_err(Error::Ax)
    }
}
