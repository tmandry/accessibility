use accessibility_sys::{
    kAXAllowedValuesAttribute, kAXChildrenAttribute, kAXContentsAttribute, kAXDescriptionAttribute,
    kAXElementBusyAttribute, kAXEnabledAttribute, kAXFocusedAttribute, kAXFocusedWindowAttribute,
    kAXFrameAttribute, kAXFrontmostAttribute, kAXHelpAttribute, kAXIdentifierAttribute,
    kAXLabelValueAttribute, kAXMainAttribute, kAXMainWindowAttribute, kAXMaxValueAttribute,
    kAXMinValueAttribute, kAXMinimizedAttribute, kAXParentAttribute, kAXPlaceholderValueAttribute,
    kAXPositionAttribute, kAXRoleAttribute, kAXRoleDescriptionAttribute,
    kAXSelectedChildrenAttribute, kAXSizeAttribute, kAXSubroleAttribute, kAXTitleAttribute,
    kAXTitleUIElementAttribute, kAXTopLevelUIElementAttribute, kAXValueAttribute,
    kAXValueDescriptionAttribute, kAXValueIncrementAttribute, kAXVisibleChildrenAttribute,
    kAXWindowAttribute, kAXWindowsAttribute,
};
use objc2_core_foundation::{
    CFArray, CFBoolean, CFGetTypeID, CFNumber, CFRetained, CFString, CFType, CFTypeID, CGPoint,
    CGRect, CGSize, ConcreteType, Type,
};
use std::marker::PhantomData;

use crate::{value::AXValue, AXUIElement, ElementFinder, Error};

/// A Core Foundation accessibility attribute value.
///
/// # Safety
///
/// `type_id` must identify `Self`, or return `None` to accept any type.
pub unsafe trait AXAttributeValue: Type + Sized + AsRef<CFType> {
    /// Returns the required type ID, or `None` to accept any type.
    fn type_id() -> Option<CFTypeID>;

    /// Downcasts `value` after checking its type ID.
    fn downcast(value: CFRetained<CFType>) -> Result<CFRetained<Self>, Error> {
        if let Some(expected) = Self::type_id() {
            let received = CFGetTypeID(Some(&value));

            if received != expected {
                return Err(Error::UnexpectedType { expected, received });
            }
        }

        // SAFETY: The value's type id matches Self, per the trait's contract.
        Ok(unsafe { CFRetained::cast_unchecked::<Self>(value) })
    }
}

macro_rules! impl_attribute_value {
    ($($typ:ty),*,) => {
        $(
            // SAFETY: The type id is the one of the implementing type.
            unsafe impl AXAttributeValue for $typ {
                fn type_id() -> Option<CFTypeID> {
                    Some(<$typ as ConcreteType>::type_id())
                }
            }
        )*
    };
}

impl_attribute_value![CFBoolean, CFNumber, CFString, AXUIElement,];

// SAFETY: Any type is acceptable as a CFType.
unsafe impl AXAttributeValue for CFType {
    fn type_id() -> Option<CFTypeID> {
        None
    }
}

// SAFETY: The type id is that of CFArray, whatever the element type. As with
// the untyped `CFArray`, the element type is not checked.
unsafe impl<T: Type> AXAttributeValue for CFArray<T> {
    fn type_id() -> Option<CFTypeID> {
        Some(<CFArray as ConcreteType>::type_id())
    }
}

// SAFETY: The type id is that of AXValue, whatever the wrapped structure. The
// structure's type is checked when the value is read out with `AXValue::value`.
unsafe impl<T> AXAttributeValue for AXValue<T> {
    fn type_id() -> Option<CFTypeID> {
        Some(accessibility_sys::AXValue::type_id())
    }
}

pub trait TAXAttribute {
    type Value: AXAttributeValue;
}

#[derive(Clone, Debug)]
pub struct AXAttribute<T>(CFRetained<CFString>, PhantomData<*const T>);

impl<T: AXAttributeValue> TAXAttribute for AXAttribute<T> {
    type Value = T;
}

impl<T> AXAttribute<T> {
    #[allow(non_snake_case)]
    pub fn as_CFString(&self) -> &CFString {
        &self.0
    }
}

macro_rules! constructor {
    ($name:ident, $typ:ty, $const:ident $(,$setter:ident)?) => {
        pub fn $name() -> AXAttribute<$typ> {
            AXAttribute(CFString::from_static_str($const), PhantomData)
        }
    };
}

macro_rules! accessor {
    (@decl $name:ident, AXValue<$typ:ty>, $const:ident, $setter:ident) => {
        accessor!(@decl $name, AXValue<$typ>, $const);
        fn $setter(&self, value: impl Into<$typ>) -> Result<(), Error>;
    };
    (@decl $name:ident, CFBoolean, $const:ident, $setter:ident) => {
        accessor!(@decl $name, CFBoolean, $const);
        fn $setter(&self, value: bool) -> Result<(), Error>;
    };
    (@decl $name:ident, $typ:ty, $const:ident, $setter:ident) => {
        accessor!(@decl $name, $typ, $const);
        fn $setter(&self, value: &$typ) -> Result<(), Error>;
    };
    (@decl $name:ident, AXValue<$typ:ty>, $const:ident) => {
        fn $name(&self) -> Result<$typ, Error>;
    };
    (@decl $name:ident, $typ:ty, $const:ident) => {
        fn $name(&self) -> Result<CFRetained<$typ>, Error>;
    };
    (@impl $name:ident, AXValue<$typ:ty>, $const:ident, $setter:ident) => {
        accessor!(@impl $name, AXValue<$typ>, $const);
        fn $setter(&self, value: impl Into<$typ>) -> Result<(), Error> {
            self.set_attribute(&AXAttribute::$name(), &*AXValue::new(&value.into())?)
        }
    };
    (@impl $name:ident, CFBoolean, $const:ident, $setter:ident) => {
        accessor!(@impl $name, CFBoolean, $const);
        fn $setter(&self, value: bool) -> Result<(), Error> {
            self.set_attribute(&AXAttribute::$name(), CFBoolean::new(value))
        }
    };
    (@impl $name:ident, $typ:ty, $const:ident, $setter:ident) => {
        accessor!(@impl $name, $typ, $const);
        fn $setter(&self, value: &$typ) -> Result<(), Error> {
            self.set_attribute(&AXAttribute::$name(), value)
        }
    };
    (@impl $name:ident, AXValue<$typ:ty>, $const:ident) => {
        fn $name(&self) -> Result<$typ, Error> {
            self.attribute(&AXAttribute::$name()).and_then(|v| v.value())
        }
    };
    (@impl $name:ident, $typ:ty, $const:ident) => {
        fn $name(&self) -> Result<CFRetained<$typ>, Error> {
            self.attribute(&AXAttribute::$name())
        }
    };
}

macro_rules! define_attributes {
    ($(($($args:tt)*)),*,) => {
        impl AXAttribute<()> {
            $(constructor!($($args)*);)*
        }

        pub trait AXUIElementAttributes {
            $(accessor!(@decl $($args)*);)*
        }

        impl AXUIElementAttributes for AXUIElement {
            $(accessor!(@impl $($args)*);)*
        }

        impl AXUIElementAttributes for ElementFinder {
            $(accessor!(@impl $($args)*);)*
        }
    }
}

impl AXAttribute<CFType> {
    pub fn new(name: &CFString) -> Self {
        AXAttribute(name.retain(), PhantomData)
    }
}

define_attributes![
    (allowed_values, CFArray<CFType>, kAXAllowedValuesAttribute),
    (children, CFArray<AXUIElement>, kAXChildrenAttribute),
    (contents, AXUIElement, kAXContentsAttribute),
    (description, CFString, kAXDescriptionAttribute),
    (element_busy, CFBoolean, kAXElementBusyAttribute),
    (enabled, CFBoolean, kAXEnabledAttribute),
    (focused, CFBoolean, kAXFocusedAttribute),
    (focused_window, AXUIElement, kAXFocusedWindowAttribute),
    (frontmost, CFBoolean, kAXFrontmostAttribute, set_frontmost),
    (frame, AXValue<CGRect>, kAXFrameAttribute),
    (help, CFString, kAXHelpAttribute),
    (identifier, CFString, kAXIdentifierAttribute),
    (label_value, CFString, kAXLabelValueAttribute),
    (main, CFBoolean, kAXMainAttribute, set_main),
    (main_window, AXUIElement, kAXMainWindowAttribute),
    (max_value, CFType, kAXMaxValueAttribute),
    (min_value, CFType, kAXMinValueAttribute),
    (minimized, CFBoolean, kAXMinimizedAttribute),
    (parent, AXUIElement, kAXParentAttribute),
    (placeholder_value, CFString, kAXPlaceholderValueAttribute),
    (
        position,
        AXValue<CGPoint>,
        kAXPositionAttribute,
        set_position
    ),
    (role, CFString, kAXRoleAttribute),
    (role_description, CFString, kAXRoleDescriptionAttribute),
    (
        selected_children,
        CFArray<AXUIElement>,
        kAXSelectedChildrenAttribute
    ),
    (size, AXValue<CGSize>, kAXSizeAttribute, set_size),
    (subrole, CFString, kAXSubroleAttribute),
    (title, CFString, kAXTitleAttribute),
    (title_ui_element, AXUIElement, kAXTitleUIElementAttribute),
    (
        top_level_ui_element,
        AXUIElement,
        kAXTopLevelUIElementAttribute
    ),
    (value, CFType, kAXValueAttribute, set_value),
    (value_description, CFString, kAXValueDescriptionAttribute),
    (value_increment, CFType, kAXValueIncrementAttribute),
    (
        visible_children,
        CFArray<AXUIElement>,
        kAXVisibleChildrenAttribute
    ),
    (window, AXUIElement, kAXWindowAttribute),
    (windows, CFArray<AXUIElement>, kAXWindowsAttribute),
];
