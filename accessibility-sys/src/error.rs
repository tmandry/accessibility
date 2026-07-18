use objc2_application_services::AXError;

pub fn error_string(error: AXError) -> &'static str {
    match error {
        AXError::Success => "kAXErrorSuccess",
        AXError::Failure => "kAXErrorFailure",
        AXError::IllegalArgument => "kAXErrorIllegalArgument",
        AXError::InvalidUIElement => "kAXErrorInvalidUIElement",
        AXError::InvalidUIElementObserver => "kAXErrorInvalidUIElementObserver",
        AXError::CannotComplete => "kAXErrorCannotComplete",
        AXError::AttributeUnsupported => "kAXErrorAttributeUnsupported",
        AXError::ActionUnsupported => "kAXErrorActionUnsupported",
        AXError::NotificationUnsupported => "kAXErrorNotificationUnsupported",
        AXError::NotImplemented => "kAXErrorNotImplemented",
        AXError::NotificationAlreadyRegistered => "kAXErrorNotificationAlreadyRegistered",
        AXError::NotificationNotRegistered => "kAXErrorNotificationNotRegistered",
        AXError::APIDisabled => "kAXErrorAPIDisabled",
        AXError::NoValue => "kAXErrorNoValue",
        AXError::ParameterizedAttributeUnsupported => "kAXErrorParameterizedAttributeUnsupported",
        AXError::NotEnoughPrecision => "kAXErrorNotEnoughPrecision",
        _ => "unknown error",
    }
}
