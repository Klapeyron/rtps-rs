use crate::structure::time::Timestamp;

/// This message modifies the logical source of the Submessages
/// that follow.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InfoTimestamp {
    /// Contains the timestamp that should be used to interpret the
    /// subsequent Submessages
    ///
    /// Present only if the InvalidateFlag is not set in the header.
    pub timestamp: Option<Timestamp>,
}
