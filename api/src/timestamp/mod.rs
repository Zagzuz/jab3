use std::{
    error::Error,
    ops::{Add, Sub},
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::NaiveDateTime;
use compact_str::CompactString;
use derive_more::Display;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod integer;

/// A representation of a timestamp (seconds and nanos since the Unix epoch).
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
    /// The number of seconds since the Unix epoch.
    pub(crate) seconds: i64,

    /// The number of nanoseconds since the Unix epoch.
    pub(crate) nanos: u32,
}

impl Timestamp {
    pub const MIN: Timestamp = Timestamp {
        seconds: 0,
        nanos: u32::MIN,
    };
    pub const MAX: Timestamp = Timestamp {
        seconds: i64::MAX,
        nanos: u32::MAX,
    };

    /// Create a new timestamp from the given number of `seconds` and `nanos`
    /// (nanoseconds).
    ///
    /// The use of the `ts!()` macro in the `unix-ts-macros` crate is advised
    /// in lieu of calling this method directly for most situations.
    ///
    /// Note: For negative timestamps, the `nanos` argument is _always_ a
    /// positive offset. Therefore, the correct way to represent a timestamp
    /// of `-0.25 seconds` is to call `new(-1, 750_000_000)`.
    pub fn new(mut seconds: i64, mut nanos: u32) -> Timestamp {
        while nanos >= 1_000_000_000 {
            seconds += 1;
            nanos -= 1_000_000_000;
        }
        Timestamp { seconds, nanos }
    }

    /// Create a timestamp from the given number of nanoseconds.
    pub fn from_nanos(nanos: impl Into<i128>) -> Timestamp {
        let nanos = nanos.into();
        let seconds: i64 = (nanos / 1_000_000_000).try_into().unwrap();
        let nanos = if seconds >= 0 {
            (nanos % 1_000_000_000) as u32
        } else {
            (1_000_000_000 - (nanos % 1_000_000_000).abs()) as u32
        };
        Timestamp { seconds, nanos }
    }

    /// Create a timestamp from the given number of microseconds.
    pub fn from_micros(micros: impl Into<i128>) -> Timestamp {
        Timestamp::from_nanos(micros.into() * 1_000)
    }

    /// Create a timestamp from the given number of milliseconds.
    pub fn from_millis(millis: impl Into<i128>) -> Timestamp {
        Timestamp::from_nanos(millis.into() * 1_000_000)
    }

    pub fn seconds(&self) -> i64 {
        self.seconds
    }

    pub fn nanos(&self) -> u32 {
        self.nanos
    }

    pub fn millis(&self) -> i128 {
        self.at_precision(3)
    }

    pub fn at_precision(&self, e: u8) -> i128 {
        i128::from(self.seconds) * 10i128.pow(e.into())
            + i128::from(self.nanos) / 10i128.pow(9 - u32::from(e))
    }

    pub fn subsec(&self, e: u8) -> u32 {
        self.nanos / 10u32.pow(9 - u32::from(e))
    }

    pub fn now() -> Self {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        Self::from_millis(since_the_epoch as i128)
    }

    pub fn to_naive_date_time(&self) -> Option<NaiveDateTime> {
        NaiveDateTime::from_timestamp_opt(self.seconds, self.nanos)
    }
}

impl FromStr for Timestamp {
    type Err = TsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut src = s.to_string().trim_start().trim_end().to_owned();
        if src.is_empty() {
            return Err("no ts input".into());
        }

        // If we have a sign bit, deal with it.
        let neg = src.starts_with('-');
        src = src.trim_start_matches('-').trim_start().to_owned();

        // If there is no decimal point, this is an integer;
        // return a timestamp from it.
        if !src.contains('.') {
            let mut sec = src
                .parse::<i64>()
                .map_err(|_| TsError("failed to parse seconds in int ts"))?;
            if neg {
                sec = -sec;
            }
            return Ok(Timestamp::new(sec, 0));
        }

        // If we start with a decimal point, prepend a zero.
        if src.starts_with('.') {
            src = format!("0{}", src);
        }

        // Split into two strings for whole seconds and nanos and return the
        // appropriate Timestamp.
        let src: Vec<&str> = src.split('.').collect();
        if src.len() > 2 {
            return Err("unrecognized ts input".into());
        }
        let mut seconds = src[0]
            .parse::<i64>()
            .map_err(|_| TsError::new("failed to parse seconds in frac ts"))?;
        let mut nanos = src[1].to_owned();
        while nanos.len() < 9 {
            nanos += "0";
        }

        // If nanos is anything other than zero, we actually need to decrement
        // the seconds by one. This is because the nanos is always positive;
        // otherwise representing -0.5 seconds would be impossible.
        //
        // Note: This counter-intuitively means *adding* one here because we are
        // tracking our sign bit separately.
        if neg && nanos != "000000000" {
            seconds += 1;
        }

        // Return the new timestamp.
        if neg {
            seconds = -seconds;
        }
        let nanos = nanos[0..9].to_string().parse::<u32>().unwrap();
        Ok(Timestamp::new(seconds, nanos))
    }
}

#[derive(Debug, Display)]
pub struct TsError(&'static str);

impl TsError {
    fn new(s: &'static str) -> TsError {
        TsError(s)
    }
}

impl Error for TsError {
    fn description(&self) -> &str {
        self.0
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

impl From<&'static str> for TsError {
    fn from(value: &'static str) -> Self {
        Self::new(value)
    }
}

impl Add for Timestamp {
    type Output = Self;

    /// Add two timestamps to one another and return the result.
    fn add(self, other: Timestamp) -> Timestamp {
        Timestamp::new(self.seconds + other.seconds, self.nanos + other.nanos)
    }
}

impl Sub for Timestamp {
    type Output = Self;

    /// Subtract the provided timestamp from this one and return the result.
    fn sub(self, other: Timestamp) -> Timestamp {
        if other.nanos > self.nanos {
            return Timestamp::new(
                self.seconds - other.seconds - 1,
                self.nanos + 1_000_000_000 - other.nanos,
            );
        }
        Timestamp::new(self.seconds - other.seconds, self.nanos - other.nanos)
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let cs = CompactString::deserialize(deserializer)?;
        let ts = Timestamp::from_str(cs.as_str()).map_err(serde::de::Error::custom)?;
        Ok(ts)
    }
}

pub fn deserialize_ts_from_i64<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Timestamp::from(i64::deserialize(deserializer)?))
}

pub fn deserialize_ts_from_millis<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Timestamp::from_millis(i64::deserialize(deserializer)?))
}

pub fn deserialize_ts_from_i64_opt<'de, D>(deserializer: D) -> Result<Option<Timestamp>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<i64>::deserialize(deserializer)?;
    Ok(opt.map(Timestamp::from))
}

pub fn deserialize_ts_from_millis_opt<'de, D>(
    deserializer: D,
) -> Result<Option<Timestamp>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<i64>::deserialize(deserializer)?;
    Ok(opt.map(Timestamp::from_millis))
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.seconds())
    }
}

pub fn serialize_ts_millis<S>(timestamp: &Timestamp, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i128(timestamp.millis())
}

#[cfg(test)]
mod tests {
    use assert2::check;

    use super::*;

    #[test]
    fn test_cmp() {
        check!(Timestamp::from(1335020400) < Timestamp::from(1335024000));
        check!(Timestamp::from(1335020400) == Timestamp::from(1335020400));
        check!(Timestamp::new(1335020400, 500_000_000) < Timestamp::new(1335020400, 750_000_000));
        check!(Timestamp::new(1, 999_999_999) < Timestamp::from(2));
    }

    #[test]
    fn test_from_nanos() {
        check!(
            Timestamp::from_nanos(1_335_020_400_000_000_000_i64) == Timestamp::new(1335020400, 0)
        );
        check!(
            Timestamp::from_nanos(1_335_020_400_500_000_000_i64)
                == Timestamp::new(1335020400, 500_000_000)
        );
        check!(Timestamp::from_nanos(-1_750_000_000) == Timestamp::new(-1, 250_000_000));
    }

    #[test]
    fn test_from_micros() {
        check!(Timestamp::from_micros(1_335_020_400_000_000_i64) == Timestamp::new(1335020400, 0));
        check!(
            Timestamp::from_micros(1_335_020_400_500_000_i64)
                == Timestamp::new(1335020400, 500_000_000)
        );
        check!(Timestamp::from_micros(-1_750_000) == Timestamp::new(-1, 250_000_000));
    }

    #[test]
    fn test_from_millis() {
        check!(Timestamp::from_millis(1_335_020_400_000_i64) == Timestamp::new(1335020400, 0));
        check!(
            Timestamp::from_millis(1_335_020_400_500_i64)
                == Timestamp::new(1335020400, 500_000_000)
        );
        check!(Timestamp::from_millis(-1_750) == Timestamp::new(-1, 250_000_000));
    }

    #[test]
    fn test_seconds() {
        assert_eq!(Timestamp::from(1335020400).seconds, 1335020400);
    }

    #[test]
    fn test_at_precision() {
        let ts = Timestamp::new(1335020400, 123456789);
        assert_eq!(ts.at_precision(3), 1335020400123);
        assert_eq!(ts.at_precision(6), 1335020400123456);
        assert_eq!(ts.at_precision(9), 1335020400123456789);
    }

    #[test]
    fn test_subsec() {
        let ts = Timestamp::new(1335020400, 123456789);
        assert_eq!(ts.subsec(3), 123);
        assert_eq!(ts.subsec(6), 123456);
        assert_eq!(ts.subsec(9), 123456789);
    }

    #[test]
    fn test_add() {
        let ts = Timestamp::from(1335020400) + Timestamp::new(86400, 1_000_000);
        assert_eq!(ts.seconds(), 1335020400 + 86400);
        assert_eq!(ts.subsec(3), 1);
    }

    #[test]
    fn test_sub() {
        let ts = Timestamp::from(1335020400) - Timestamp::new(86400, 0);
        assert_eq!(ts.seconds(), 1335020400 - 86400);
        assert_eq!(ts.nanos, 0);
    }

    #[test]
    fn test_sub_nano_overflow() {
        let ts = Timestamp::from(1335020400) - Timestamp::new(0, 500_000_000);
        assert_eq!(ts.seconds(), 1335020399);
        assert_eq!(ts.subsec(1), 5);
    }

    #[test]
    fn test_deserialize() {
        check!(
            Timestamp::from(1335020400)
                == serde_json::from_str::<Timestamp>(r#""1335020400""#).unwrap()
        );
        check!(
            Timestamp::new(1680905072, 460772000)
                == serde_json::from_str::<Timestamp>(r#""1680905072.460772""#).unwrap()
        );
    }

    #[test]
    fn test_serialize() {
        assert_eq!(
            serde_json::to_string(&Timestamp::from(1335020400)).unwrap(),
            "1335020400".to_string()
        );
    }
}
