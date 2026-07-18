use std::fmt;

/// An accessibility API error.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AXError {
    Failure,
    IllegalArgument,
    InvalidUIElement,
    InvalidUIElementObserver,
    CannotComplete,
    AttributeUnsupported,
    ActionUnsupported,
    NotificationUnsupported,
    NotImplemented,
    NotificationAlreadyRegistered,
    NotificationNotRegistered,
    APIDisabled,
    NoValue,
    ParameterizedAttributeUnsupported,
    NotEnoughPrecision,
    Unknown(i32),
}

impl AXError {
    /// Converts a raw code, returning `None` for success.
    pub fn from_raw(error: accessibility_sys::AXError) -> Option<Self> {
        use accessibility_sys::AXError as Raw;

        Some(match error {
            Raw::Success => return None,
            Raw::Failure => Self::Failure,
            Raw::IllegalArgument => Self::IllegalArgument,
            Raw::InvalidUIElement => Self::InvalidUIElement,
            Raw::InvalidUIElementObserver => Self::InvalidUIElementObserver,
            Raw::CannotComplete => Self::CannotComplete,
            Raw::AttributeUnsupported => Self::AttributeUnsupported,
            Raw::ActionUnsupported => Self::ActionUnsupported,
            Raw::NotificationUnsupported => Self::NotificationUnsupported,
            Raw::NotImplemented => Self::NotImplemented,
            Raw::NotificationAlreadyRegistered => Self::NotificationAlreadyRegistered,
            Raw::NotificationNotRegistered => Self::NotificationNotRegistered,
            Raw::APIDisabled => Self::APIDisabled,
            Raw::NoValue => Self::NoValue,
            Raw::ParameterizedAttributeUnsupported => Self::ParameterizedAttributeUnsupported,
            Raw::NotEnoughPrecision => Self::NotEnoughPrecision,
            other => Self::Unknown(other.0),
        })
    }

    /// The raw error code.
    pub fn as_raw(self) -> accessibility_sys::AXError {
        use accessibility_sys::AXError as Raw;

        match self {
            Self::Failure => Raw::Failure,
            Self::IllegalArgument => Raw::IllegalArgument,
            Self::InvalidUIElement => Raw::InvalidUIElement,
            Self::InvalidUIElementObserver => Raw::InvalidUIElementObserver,
            Self::CannotComplete => Raw::CannotComplete,
            Self::AttributeUnsupported => Raw::AttributeUnsupported,
            Self::ActionUnsupported => Raw::ActionUnsupported,
            Self::NotificationUnsupported => Raw::NotificationUnsupported,
            Self::NotImplemented => Raw::NotImplemented,
            Self::NotificationAlreadyRegistered => Raw::NotificationAlreadyRegistered,
            Self::NotificationNotRegistered => Raw::NotificationNotRegistered,
            Self::APIDisabled => Raw::APIDisabled,
            Self::NoValue => Raw::NoValue,
            Self::ParameterizedAttributeUnsupported => Raw::ParameterizedAttributeUnsupported,
            Self::NotEnoughPrecision => Raw::NotEnoughPrecision,
            Self::Unknown(code) => Raw(code),
        }
    }

    /// Returns the corresponding constant name.
    pub fn name(self) -> &'static str {
        accessibility_sys::error_string(self.as_raw())
    }
}

impl fmt::Display for AXError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Failure => "a system error occurred",
            Self::IllegalArgument => "an illegal argument was passed",
            Self::InvalidUIElement => "the element is invalid",
            Self::InvalidUIElementObserver => "the observer is invalid",
            Self::CannotComplete => "the application is busy or unresponsive",
            Self::AttributeUnsupported => "the element does not support this attribute",
            Self::ActionUnsupported => "the element does not support this action",
            Self::NotificationUnsupported => "the element does not support this notification",
            Self::NotImplemented => "the application does not implement the accessibility API",
            Self::NotificationAlreadyRegistered => "the notification is already registered",
            Self::NotificationNotRegistered => "the notification is not registered",
            Self::APIDisabled => {
                "the accessibility API is disabled; \
                 grant accessibility permission to this process"
            }
            Self::NoValue => "the requested value does not exist",
            Self::ParameterizedAttributeUnsupported => {
                "the element does not support this parameterized attribute"
            }
            Self::NotEnoughPrecision => "not enough precision",
            Self::Unknown(code) => return write!(f, "unknown accessibility error {code}"),
        };

        write!(f, "{message} ({})", self.name())
    }
}
