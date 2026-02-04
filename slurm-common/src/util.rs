use chrono::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub struct SlurmQuantity(u64);

impl<'de> Deserialize<'de> for SlurmQuantity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(SlurmQuantity(deserialize_quantity(deserializer)?))
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub struct SlurmDuration(chrono::Duration);

impl<'de> Deserialize<'de> for SlurmDuration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(SlurmDuration(deserialize_duration(deserializer)?))
    }
}

impl Serialize for SlurmQuantity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl Serialize for SlurmDuration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

pub fn deserialize_quantity<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct ResVisitor;

    impl<'de> serde::de::Visitor<'de> for ResVisitor {
        type Value = u64;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string like '100M' or '1G' or a raw number")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let mut value = v.trim();
            let mut multiplier = 1;
            if value.ends_with('M') {
                multiplier = 1000 * 1000;
                value = &value[..value.len() - 1];
            } else if value.ends_with('G') {
                multiplier = 1000 * 1000 * 1000;
                value = &value[..value.len() - 1];
            }
            Ok((value.parse::<f64>().map_err(|_| {
                E::custom(format!(
                    "Invalid resource quantity: {} in specifier {}",
                    value, v
                ))
            })? * multiplier as f64) as u64)
        }
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if v < 0 {
                return Err(E::custom(format!(
                    "Invalid resource quantity (negative): {}",
                    v
                )));
            }
            Ok(v as u64)
        }
    }
    deserializer.deserialize_any(ResVisitor)
}

pub fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct DurationVisitor;
    impl<'de> serde::de::Visitor<'de> for DurationVisitor {
        type Value = Duration;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a duration like 1-02:00:00 or 08:00:00")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            println!("visit_str {}", v);
            let s = v.trim();
            // Slurm format: [days-]hours:minutes:seconds
            let (days, time_str) = if let Some((days_str, time_str)) = s.split_once('-') {
                let days: i64 = days_str
                    .trim()
                    .parse()
                    .map_err(|_| serde::de::Error::custom(format!("Invalid time {}", s)))?;
                (Duration::days(days), time_str.trim())
            } else {
                (Duration::zero(), s.trim())
            };
            // Handle "08:00:00" or "00:15:41" (HH:MM:SS)
            let mut duration = days;
            for (i, part) in time_str.split(':').rev().enumerate() {
                let value: i64 = part
                    .trim()
                    .parse()
                    .map_err(|_| serde::de::Error::custom(format!("Invalid time {}", s)))?;
                duration = duration
                    + match i {
                        0 => Duration::seconds(value),
                        1 => Duration::minutes(value),
                        2 => Duration::hours(value),
                        _ => return Err(serde::de::Error::custom(format!("Invalid time {}", s))),
                    };
            }
            Ok(duration)
        }
        fn visit_seq<E>(self, seq: E) -> Result<Self::Value, E::Error>
        where
            E: serde::de::SeqAccess<'de>,
        {
            Duration::deserialize(serde::de::value::SeqAccessDeserializer::new(seq))
        }
    }
    deserializer.deserialize_any(DurationVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_quantity_parsing() {
        let q: SlurmQuantity = serde_json::from_str("\"100M\"").unwrap();
        assert_eq!(q.0, 100_000_000);

        let q: SlurmQuantity = serde_json::from_str("\"1G\"").unwrap();
        assert_eq!(q.0, 1_000_000_000);

        let q: SlurmQuantity = serde_json::from_str("\"500\"").unwrap();
        assert_eq!(q.0, 500);
    }

    #[test]
    fn test_resource_quantity_autoencode() {
        let q = SlurmQuantity(100_000_000);
        let s = serde_json::to_string(&q).unwrap();
        let v = serde_json::from_str::<SlurmQuantity>(&s).unwrap();
        assert_eq!(v, q);
    }

    #[test]
    fn test_slurm_duration_parsing() {
        let d: SlurmDuration = serde_json::from_str("\"08:00:00\"").unwrap();
        assert_eq!(d.0, chrono::Duration::hours(8));

        let d: SlurmDuration = serde_json::from_str("\"00:15:41\"").unwrap();
        assert_eq!(
            d.0,
            chrono::Duration::minutes(15) + chrono::Duration::seconds(41)
        );

        let d: SlurmDuration = serde_json::from_str("\"1-02:00:00\"").unwrap();
        assert_eq!(d.0, chrono::Duration::days(1) + chrono::Duration::hours(2));
    }

    #[test]
    fn test_slurm_duration_autoencode() {
        let d = SlurmDuration(chrono::Duration::hours(8));
        let s = serde_json::to_string(&d).unwrap();
        let v = serde_json::from_str::<SlurmDuration>(&s).unwrap();
        assert_eq!(v, d);
    }
}
