use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_chainpack::{cpdatetime::CPDateTime, de::from_slice, ser::to_vec};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Event {
    timestamp: CPDateTime,
}

#[test]
fn test_datetime_serialization_round_trip() {
    let test_cases = vec![
        ("2018-02-02T00:00:00.001+00:00"),
        ("2018-02-02T01:00:00.001+01:00"),
        ("2018-12-02T00:00:00+00:00"),
        ("2018-01-01T00:00:00+00:00"),
        ("2019-01-01T00:00:00+00:00"),
        ("2020-01-01T00:00:00+00:00"),
        ("2021-01-01T00:00:00+00:00"),
        ("2031-01-01T00:00:00+00:00"),
        ("2041-01-01T00:00:00+00:00"),
        ("2041-03-04T00:00:00-10:15"),
        ("2041-03-04T00:00:00.123-10:15"),
        ("1970-01-01T00:00:00+00:00"),
        ("2017-05-03T05:52:03+00:00"),
        ("2017-05-03T15:52:03.923Z"),
        ("2017-05-03T15:52:31.123+10:00"),
        ("2017-05-03T15:52:03Z"),
        ("2017-05-03T15:52:03-01:30"),
        ("2017-05-03T15:52:03.923+00:00"),
    ];
    for dt_str in test_cases {
        println!("Testing: {}", dt_str);
        let dt = DateTime::parse_from_rfc3339(dt_str).unwrap();
        let cpdt = CPDateTime(dt);
        {
            let serialized = to_vec(&cpdt).expect("serialization failed");
            println!("Serialized: {serialized:x?}");
            let deserialized: CPDateTime = from_slice(&serialized).expect("deserialization failed");
            assert_eq!(cpdt, deserialized);
        }
        {
            let event = Event { timestamp: cpdt };
            let serialized = to_vec(&event).expect("serialization failed");
            println!("Serialized: {serialized:x?}");
            let deserialized: Event = from_slice(&serialized).expect("deserialization failed");
            assert_eq!(event, deserialized);
        }
    }
}
