use speedy::{Readable, Writable};
use std::convert::From;
use std::time::Duration;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Readable, Writable)]
pub struct Duration_t {
    seconds: i32,
    fraction: u32,
}

impl Duration_t {
    pub const DURATION_ZERO: Duration_t = Duration_t {
        seconds: 0,
        fraction: 0,
    };
    pub const DURATION_INVALID: Duration_t = Duration_t {
        seconds: -1,
        fraction: 0xFFFFFFFF,
    };
    pub const DURATION_INFINITE: Duration_t = Duration_t {
        seconds: 0x7FFFFFFF,
        fraction: 0xFFFFFFFF,
    };
}

impl From<Duration> for Duration_t {
    fn from(duration: Duration) -> Self {
        Duration_t {
            seconds: duration.as_secs() as i32,
            fraction: duration.subsec_nanos() as u32,
        }
    }
}

impl From<Duration_t> for Duration {
    fn from(duration: Duration_t) -> Self {
        Duration::new(duration.seconds as u64, duration.fraction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    serialization_test!( type = Duration_t,
    {
        duration_zero,
        Duration_t::DURATION_ZERO,
        le = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        be = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    },
    {
        duration_invalid,
        Duration_t::DURATION_INVALID,
        le = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        be = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
    },
    {
        duration_infinite,
        Duration_t::DURATION_INFINITE,
        le = [0xFF, 0xFF, 0xFF, 0x7F, 0xFF, 0xFF, 0xFF, 0xFF],
        be = [0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
    },
    {
        duration_current_empty_fraction,
        Duration_t { seconds: 1_537_045_491, fraction: 0 },
        le = [0xF3, 0x73, 0x9D, 0x5B, 0x00, 0x00, 0x00, 0x00],
        be = [0x5B, 0x9D, 0x73, 0xF3, 0x00, 0x00, 0x00, 0x00]
    },
    {
        duration_from_wireshark,
        Duration_t { seconds: 1_519_152_760, fraction: 1_328_210_046 },
        le = [0x78, 0x6E, 0x8C, 0x5A, 0x7E, 0xE0, 0x2A, 0x4F],
        be = [0x5A, 0x8C, 0x6E, 0x78, 0x4F, 0x2A, 0xE0, 0x7E]
    });

    const NANOS_PER_SEC: u64 = 1_000_000_000;

    #[test]
    fn convert_from_duration() {
        let duration = Duration::from_nanos(1_519_152_761 * NANOS_PER_SEC + 328_210_046);
        let duration: Duration_t = duration.into();

        assert_eq!(
            duration,
            Duration_t {
                seconds: 1_519_152_761,
                fraction: 328_210_046,
            }
        );
    }

    #[test]
    fn convert_to_duration() {
        let duration = Duration_t {
            seconds: 1_519_152_760,
            fraction: 1_328_210_046,
        };
        let duration: Duration = duration.into();

        assert_eq!(
            duration,
            Duration::from_nanos(1_519_152_760 * NANOS_PER_SEC + 1_328_210_046)
        );
    }
}
