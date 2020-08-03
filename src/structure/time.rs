use speedy::{Readable, Writable};
use std::convert::{From, TryFrom};
use std::time::{Duration, SystemTime};

/// The representation of the time is the one defined by the IETF Network Time
/// Protocol (NTP) Standard (IETF RFC 1305). In this representation, time is
/// expressed in seconds and fraction of seconds using the formula:
/// time = seconds + (fraction / 2^(32))
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Readable, Writable)]
pub struct Time_t {
    seconds: i32,
    fraction: u32,
}

pub type Timestamp = Time_t;

impl Time_t {
    pub const TIME_ZERO: Time_t = Time_t {
        seconds: 0,
        fraction: 0,
    };
    pub const TIME_INVALID: Time_t = Time_t {
        seconds: -1,
        fraction: 0xFFFF_FFFF,
    };
    pub const TIME_INFINITE: Time_t = Time_t {
        seconds: 0x7FFF_FFFF,
        fraction: 0xFFFF_FFFF,
    };
}

const NANOS_PER_SEC: i64 = 1_000_000_000;

impl From<SystemTime> for Time_t {
    fn from(sys_time: SystemTime) -> Self {
        sys_time
            .duration_since(std::time::UNIX_EPOCH)
            .and_then(|dur_since_epoch| {
                let fraction = (i64::from(dur_since_epoch.subsec_nanos()) << 32) / NANOS_PER_SEC;
                Ok(Time_t {
                    seconds: dur_since_epoch.as_secs() as i32,
                    fraction: fraction as u32,
                })
            })
            .unwrap_or(Time_t::TIME_INVALID)
    }
}

impl TryFrom<Time_t> for SystemTime {
    type Error = &'static str;

    fn try_from(time: Time_t) -> Result<Self, Self::Error> {
        match time {
            Time_t::TIME_INVALID => Err("Conversion from Time_t::TIME_INVALID"),
            _ => {
                let fraction = (i64::from(time.fraction) * NANOS_PER_SEC) >> 32;
                Ok(std::time::UNIX_EPOCH + Duration::new(time.seconds as u64, fraction as u32))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn conversion_from_invalid_time_caused_an_error() {
        matches!(SystemTime::try_from(Time_t::TIME_INVALID), Err(_));
    }

    macro_rules! conversion_test {
        ($({ $name:ident, time = $time:expr, systime = $systime:expr, }),+) => {
            $(mod $name {
                use super::*;

                const FRACS_PER_SEC: i64 = 0x100000000;

                macro_rules! assert_ge_at_most_by {
                    ($e:expr, $x:expr, $y:expr) => {
                        assert!($x >= $y);
                        assert!($x - $y <= $e);
                    }
                }

                #[test]
                fn time_from_systime() {
                    let time = Time_t::from($systime);
                    let epsilon = (FRACS_PER_SEC / NANOS_PER_SEC) as u32 + 1;

                    assert_eq!($time.seconds, time.seconds);
                    assert_ge_at_most_by!(epsilon, $time.fraction, time.fraction);
                }

                #[test]
                fn time_from_eq_into_time() {
                    let time_from = Time_t::from($systime);
                    let into_time: Time_t = $systime.into();

                    assert_eq!(time_from, into_time);
                }

                #[test]
                fn systime_from_time() {
                    let systime = SystemTime::try_from($time).unwrap();
                    let epsilon = (NANOS_PER_SEC / FRACS_PER_SEC) as u32 + 1;

                    let extract_nanos = |systime: SystemTime| {
                        systime
                            .duration_since(std::time::UNIX_EPOCH)
                            .and_then(|duration| Ok(duration.subsec_nanos()))
                            .unwrap()
                    };

                    assert_eq!($systime, systime);
                    assert_ge_at_most_by!(epsilon,
                        extract_nanos($systime),
                        extract_nanos(systime)
                    );
                }

                #[test]
                fn systime_from_eq_into_systime() {
                    let systime_from = SystemTime::try_from($time).unwrap();
                    let into_systime: SystemTime = $time.try_into().unwrap();

                    assert_eq!(systime_from, into_systime);
                }
            })+
        }
    }

    conversion_test!(
    {
        convert_time_zero,
        time = Time_t::TIME_ZERO,
        systime = std::time::UNIX_EPOCH + Duration::new(0, 0),
    },
    {
        convert_time_non_zero,
        time = Time_t {
            seconds: 1,
            fraction: 5,
        },
        systime = std::time::UNIX_EPOCH + Duration::new(1, 1),
    },
    {
        convert_time_infinite,
        time = Time_t::TIME_INFINITE,
        systime = std::time::UNIX_EPOCH + Duration::new(0x7FFFFFFF, 999_999_999),
    },
    {
        convert_time_non_infinite,
        time = Time_t {
            seconds: 0x7FFFFFFF,
            fraction: 0xFFFFFFFA,
        },
        systime = std::time::UNIX_EPOCH + Duration::new(0x7FFFFFFF, 999_999_998),
    },
    {
        convert_time_half_range,
        time = Time_t {
            seconds: 0x40000000,
            fraction: 0x80000000,
        },
        systime = std::time::UNIX_EPOCH + Duration::new(0x40000000, 500_000_000),
    });

    serialization_test!( type = Time_t,
    {
        time_zero,
        Time_t::TIME_ZERO,
        le = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        be = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    },
    {
        time_invalid,
        Time_t::TIME_INVALID,
        le = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        be = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
    },
    {
        time_infinite,
        Time_t::TIME_INFINITE,
        le = [0xFF, 0xFF, 0xFF, 0x7F, 0xFF, 0xFF, 0xFF, 0xFF],
        be = [0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
    },
    {
        time_current_empty_fraction,
        Time_t { seconds: 1_537_045_491, fraction: 0 },
        le = [0xF3, 0x73, 0x9D, 0x5B, 0x00, 0x00, 0x00, 0x00],
        be = [0x5B, 0x9D, 0x73, 0xF3, 0x00, 0x00, 0x00, 0x00]
    },
    {
        time_from_wireshark,
        Time_t { seconds: 1_519_152_760, fraction: 1_328_210_046 },
        le = [0x78, 0x6E, 0x8C, 0x5A, 0x7E, 0xE0, 0x2A, 0x4F],
        be = [0x5A, 0x8C, 0x6E, 0x78, 0x4F, 0x2A, 0xE0, 0x7E]
    });
}
