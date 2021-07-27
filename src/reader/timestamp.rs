pub mod floating_timestamp {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let seconds = date.timestamp();
        let ns = date.timestamp_subsec_nanos();
        let floating = seconds as f64 + (ns as f64 / 1_000_000_000f64);
        let s = format!("{}", floating);
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let floating: f64 = s
            .parse()
            .map_err(|err| de::Error::custom(format_args!("Failed to parse timestamp: {}", err)))?;
        let seconds = floating as i64;
        let ns = (floating - seconds as f64) * 1_000_000_000f64;
        Ok(Utc.timestamp(seconds, ns as u32))
    }

    #[cfg(test)]
    mod test {
        use chrono::TimeZone;
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        #[serde(transparent)]
        struct TestDate {
            #[serde(with = "super")]
            pub date: super::DateTime<super::Utc>,
        }

        #[test]
        fn test_serialize() {
            let date: TestDate = TestDate {
                date: super::Utc.timestamp(12345, 123),
            };

            let format = serde_json::to_string(&date).unwrap();
            assert_eq!(format, "\"12345.000000123\"");
        }

        #[test]
        fn test_deserialize() {
            let serialized = "\"12345.000000123\"";
            let date: TestDate = serde_json::from_str(serialized).unwrap();
            assert_eq!(date.date.timestamp(), 12345);
            assert_eq!(date.date.timestamp_subsec_nanos(), 123);
        }
    }
}
