use serde::{Deserialize, Deserializer, Serialize, Serializer};
use time::error::ComponentRange;
use time::{Duration, OffsetDateTime};

use super::EventTime;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Milliseconds(i64);

impl TryFrom<OffsetDateTime> for Milliseconds {
    type Error = &'static str;

    fn try_from(value: OffsetDateTime) -> Result<Self, &'static str> {
        value
            .unix_timestamp()
            .checked_mul(1000)
            .ok_or("too many seconds in timestamp")?
            .checked_add(value.millisecond() as i64)
            .ok_or("could not add milliseconds in timestamp")
            .map(Self)
    }
}

impl TryFrom<Milliseconds> for OffsetDateTime {
    type Error = ComponentRange;

    fn try_from(value: Milliseconds) -> Result<Self, ComponentRange> {
        let seconds = value.0 / 1000;
        let ms = value.0 % 1000;

        OffsetDateTime::from_unix_timestamp(seconds).and_then(|t| t.replace_millisecond(ms as u16))
    }
}
impl TryFrom<EventTime> for Milliseconds {
    type Error = &'static str;

    fn try_from(value: EventTime) -> Result<Self, &'static str> {
        Self::try_from(*value)
    }
}

impl TryFrom<Milliseconds> for EventTime {
    type Error = ComponentRange;

    fn try_from(value: Milliseconds) -> Result<Self, ComponentRange> {
        OffsetDateTime::try_from(value).map(EventTime::from)
    }
}

impl TryFrom<Duration> for Milliseconds {
    type Error = &'static str;

    fn try_from(value: Duration) -> Result<Self, Self::Error> {
        value
            .whole_seconds()
            .checked_mul(1000)
            .ok_or("too many seconds in duration")?
            .checked_add(value.subsec_milliseconds() as i64)
            .ok_or("could not add milliseconds in duration")
            .map(Self)
    }
}

impl From<Milliseconds> for Duration {
    fn from(value: Milliseconds) -> Self {
        let seconds = value.0 / 1000;
        let ms = (value.0 % 1000) as i32;
        Duration::new(seconds, ms * 1_000_000)
    }
}

pub mod duration {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use time::Duration;

    use super::Milliseconds;

    pub fn serialize<S: Serializer>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::Error;
        Milliseconds::try_from(*duration)
            .map_err(S::Error::custom)
            .and_then(move |ms| ms.serialize(serializer))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Duration, D::Error> {
        Milliseconds::deserialize(deserializer).map(Duration::from)
    }
}
