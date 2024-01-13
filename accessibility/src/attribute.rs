use accessibility_sys::{
    kAXAllowedValuesAttribute, kAXChildrenAttribute, kAXContentsAttribute, kAXDescriptionAttribute,
    kAXElementBusyAttribute, kAXEnabledAttribute, kAXFocusedAttribute, kAXFrameAttribute,
    kAXHelpAttribute, kAXIdentifierAttribute, kAXLabelValueAttribute, kAXMainAttribute,
    kAXMaxValueAttribute, kAXMinValueAttribute, kAXMinimizedAttribute, kAXParentAttribute,
    kAXPlaceholderValueAttribute, kAXPositionAttribute, kAXRoleAttribute,
    kAXRoleDescriptionAttribute, kAXSelectedChildrenAttribute, kAXSizeAttribute,
    kAXSubroleAttribute, kAXTitleAttribute, kAXTopLevelUIElementAttribute, kAXValueAttribute,
    kAXValueDescriptionAttribute, kAXValueIncrementAttribute, kAXVisibleChildrenAttribute,
    kAXWindowAttribute, kAXWindowsAttribute,
};
use core_foundation::{
    array::CFArray,
    base::{CFType, TCFType},
    boolean::CFBoolean,
    string::CFString,
};
use core_graphics_types::geometry::{CGPoint, CGRect, CGSize};
use std::{fmt::Debug, marker::PhantomData};

use crate::{value::AXValue, AXUIElement, ElementFinder, Error};

pub trait TAXAttribute {
    type Value: TCFType;
}

#[derive(Clone)]
pub struct AXAttribute<T>(CFString, PhantomData<*const T>);

impl<T> Debug for AXAttribute<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: TCFType> TAXAttribute for AXAttribute<T> {
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
            AXAttribute(CFString::from_static_string($const), PhantomData)
        }
    };
}

macro_rules! accessor {
    (@decl $name:ident, AXValue<$typ:ty>, $const:ident, $setter:ident) => {
        accessor!(@decl $name, AXValue<$typ>, $const);
        fn $setter(&self, value: impl Into<$typ>) -> Result<(), Error>;
    };
    (@decl $name:ident, $typ:ty, $const:ident, $setter:ident) => {
        accessor!(@decl $name, $typ, $const);
        fn $setter(&self, value: impl Into<$typ>) -> Result<(), Error>;
    };
    (@decl $name:ident, AXValue<$typ:ty>, $const:ident) => {
        fn $name(&self) -> Result<$typ, Error>;
    };
    (@decl $name:ident, $typ:ty, $const:ident) => {
        fn $name(&self) -> Result<$typ, Error>;
    };
    (@impl $name:ident, AXValue<$typ:ty>, $const:ident, $setter:ident) => {
        accessor!(@impl $name, AXValue<$typ>, $const);
        fn $setter(&self, value: impl Into<$typ>) -> Result<(), Error> {
            self.set_attribute(&AXAttribute::$name(), AXValue::new(&value.into()).expect("wrong type"))
        }
    };
    (@impl $name:ident, $typ:ty, $const:ident, $setter:ident) => {
        accessor!(@impl $name, $typ, $const);
        fn $setter(&self, value: impl Into<$typ>) -> Result<(), Error> {
            self.set_attribute(&AXAttribute::$name(), value)
        }
    };
    (@impl $name:ident, AXValue<$typ:ty>, $const:ident) => {
        fn $name(&self) -> Result<$typ, Error> {
            self.attribute(&AXAttribute::$name()).map(|v| v.value().expect("wrong type"))
        }
    };
    (@impl $name:ident, $typ:ty, $const:ident) => {
        fn $name(&self) -> Result<$typ, Error> {
            self.attribute(&AXAttribute::$name())
        }
    };
}

macro_rules! debug_field {
    ($self:ident, $fmt:ident, $name:ident, AXUIElement, $($rest:tt)*) => {
        debug_field!(@short $self, $fmt, $name, AXUIElement, $long_name);
    };
    ($self:ident, $fmt:ident, $name:ident, CFArray<AXUIElement>, $($rest:tt)*) => {
            let $name = $self.$name();
            if let Ok(value) = &$name {
                $fmt.field(stringify!($name), &NoAlternate(Forward(&value)));
            }
    };
    ($self:ident, $fmt:ident, $name:ident, CFBoolean, $($rest:tt)*) => {
            let $name = $self.$name();
            if let Ok(value) = &$name {
                let value: bool = value.clone().into();
                $fmt.field(stringify!($name), &value);
            }
    };
    (@short $self:ident, $fmt:ident, $name:ident, $type:ty, $($rest:tt)*) => {
        let $name = $self.$name();
        if let Ok(value) = &$name {
            $fmt.field(stringify!($name), &NoAlternate(value));
        }
    };
    ($self:ident, $fmt:ident, $name:ident, $type:ty, $($rest:tt)*) => {
        let $name = $self.$name();
        if let Ok(value) = &$name {
            $fmt.field(stringify!($name), &value);
        }
    };
}

struct NoAlternate<T>(T);
impl<T: Debug> Debug for NoAlternate<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

struct Forward<'a>(&'a CFArray<AXUIElement>);
impl<'a> Debug for Forward<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();
        for elem in self.0 {
            let elem: &AXUIElement = &*elem;
            list.entry(elem);
        }
        list.finish()
    }
}

macro_rules! define_attributes {
    (@get_sys_name $name:ident, $type:ty, $sys_name:ident $(, $rest:tt)*) => {
        $sys_name
    };

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

        impl AXUIElement {
            pub(crate) fn debug_all(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut fmt = f.debug_struct("AXUIElement");

                $(debug_field!(self, fmt, $($args)*);)*

                let Ok(attr_names) = self.attribute_names() else {
                    return fmt.finish();
                };
                let attr_names: Vec<CFString> = attr_names.iter().filter(|name| {
                    $(**name != define_attributes!(@get_sys_name $($args)*) &&)* true
                }).map(|n| n.clone()).collect();
                for name in attr_names {
                    let attr = AXAttribute(name, PhantomData);
                    if let Ok(val) = self.attribute::<CFType>(&attr) {
                        fmt.field(&attr.as_CFString().to_string(), &val);
                    }
                }
                fmt.finish()
            }
        }
    }
}

impl AXAttribute<CFType> {
    pub fn new(name: &CFString) -> Self {
        AXAttribute(name.to_owned(), PhantomData)
    }
}

define_attributes![
    // These we want to appear first in debug output.
    (role, CFString, kAXRoleAttribute),
    (subrole, CFString, kAXSubroleAttribute),
    // The rest are in alphabetical order.
    (allowed_values, CFArray<CFType>, kAXAllowedValuesAttribute),
    (children, CFArray<AXUIElement>, kAXChildrenAttribute),
    (contents, AXUIElement, kAXContentsAttribute),
    (description, CFString, kAXDescriptionAttribute),
    (element_busy, CFBoolean, kAXElementBusyAttribute),
    (enabled, CFBoolean, kAXEnabledAttribute),
    (focused, CFBoolean, kAXFocusedAttribute),
    (frame, AXValue<CGRect>, kAXFrameAttribute),
    (help, CFString, kAXHelpAttribute),
    (identifier, CFString, kAXIdentifierAttribute),
    (label_value, CFString, kAXLabelValueAttribute),
    (main, CFBoolean, kAXMainAttribute, set_main),
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
    (role_description, CFString, kAXRoleDescriptionAttribute),
    (
        selected_children,
        CFArray<AXUIElement>,
        kAXSelectedChildrenAttribute
    ),
    (size, AXValue<CGSize>, kAXSizeAttribute, set_size),
    (title, CFString, kAXTitleAttribute),
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
