//! Validity [`Time`] as defined in RFC 5280

use core::time::Duration;
use der::{
    asn1::{GeneralizedTime, UtcTime},
    Choice,
};

/// Validity [`Time`] as defined in [RFC 5280 Section 4.1.2.5].
///
/// Schema definition from [RFC 5280 Appendix A]:
///
/// ```text
/// Time ::= CHOICE {
///      utcTime        UTCTime,
///      generalTime    GeneralizedTime }
/// ```
///
/// [RFC 5280 Section 4.1.2.5]: https://tools.ietf.org/html/rfc5280#section-4.1.2.5
/// [RFC 5280 Appendix A]: https://tools.ietf.org/html/rfc5280#page-117
#[derive(Choice, Copy, Clone, Debug)]
pub enum Time {
    /// Legacy UTC time (has 2-digit year, valid only through 2050).
    #[asn1(type = "UTCTime")]
    UtcTime(UtcTime),

    /// Modern [`GeneralizedTime`] encoding with 4-digit year.
    #[asn1(type = "GeneralizedTime")]
    GeneralTime(GeneralizedTime),
}

impl Time {
    /// Get duration since `UNIX_EPOCH`.
    pub fn unix_duration(self) -> Duration {
        match self {
            Time::UtcTime(t) => t.unix_duration(),
            Time::GeneralTime(t) => t.unix_duration(),
        }
    }
}
