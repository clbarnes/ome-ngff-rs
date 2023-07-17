use std::collections::HashSet;

use crate::util::variant_from_data;
use serde::{Deserialize, Serialize};
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
use thiserror::Error;

// #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
// pub struct SpaceAxis {
//     name: String,
//     unit: Option<SpaceUnit>,
// }

// #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
// pub struct TimeAxis {
//     name: String,
//     unit: Option<TimeUnit>,
// }

// #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
// pub struct ChannelAxis {
//     name: String,
//     unit: Option<String>,
// }

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CoreAxis {
    // may need to un-pack these if we want to add distinct functionality to axes, e.g. impl traits
    Space {
        name: String,
        unit: Option<SpaceUnit>,
    },
    Time {
        name: String,
        unit: Option<TimeUnit>,
    },
    Channel {
        name: String,
        unit: Option<String>,
    },
}

// variant_from_data!(KnownAxis, Space, SpaceAxis);
// variant_from_data!(KnownAxis, Time, TimeAxis);
// variant_from_data!(KnownAxis, Channel, ChannelAxis);

// #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
// pub struct UnknownAxis {
//     name: String,
//     #[serde(rename = "type")]
//     axis_type: Option<String>,
//     unit: Option<String>,
// }

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Axis {
    Core(CoreAxis),
    Custom {
        name: String,
        #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
        axis_type: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        unit: Option<String>,
    },
}

impl Axis {
    pub fn name(&self) -> &str {
        match self {
            Axis::Core(k) => match k {
                CoreAxis::Space { name, .. } => name.as_str(),
                CoreAxis::Time { name, .. } => name.as_str(),
                CoreAxis::Channel { name, .. } => name.as_str(),
            },
            Axis::Custom { name, .. } => name.as_str(),
        }
    }
}

variant_from_data!(Axis, Core, CoreAxis);
// variant_from_data!(Axis, Unknown, UnknownAxis);
// transitive_into!(Axis, KnownAxis);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize_enum_str, Deserialize_enum_str)]
#[serde(rename_all = "lowercase")]
pub enum SpaceUnit {
    Angstrom,
    Attometer,
    Centimeter,
    Decimeter,
    Exameter,
    Femtometer,
    Foot,
    Gigameter,
    Hectometer,
    Inch,
    Kilometer,
    Megameter,
    Meter,
    Micrometer,
    Mile,
    Millimeter,
    Nanometer,
    Parsec,
    Petameter,
    Picometer,
    Terameter,
    Yard,
    Yoctometer,
    Yottameter,
    Zeptometer,
    Zettameter,
    #[serde(other)]
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize_enum_str, Deserialize_enum_str)]
#[serde(rename_all = "lowercase")]
pub enum TimeUnit {
    Attosecond,
    Centisecond,
    Day,
    Decisecond,
    Exasecond,
    Femtosecond,
    Gigasecond,
    Hectosecond,
    Hour,
    Kilosecond,
    Megasecond,
    Microsecond,
    Millisecond,
    Minute,
    Nanosecond,
    Parsec,
    Petasecond,
    Picosecond,
    Second,
    Terasecond,
    Yoctosecond,
    Yottasecond,
    Zeptosecond,
    Zettasecond,
    #[serde(other)]
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InvalidAxes {
    #[error("Expected 2-5 axes, got {0}")]
    Count(usize),
    #[error("Expected 2-3 space axes")]
    NSpace,
    #[error("Got >1 time axes")]
    NTime,
    #[error("Got >1 channel/null/custom axes")]
    NOther,
    #[error("Invalid order: expected [time], [channel/custom], space, space, [space]")]
    Order,
    #[error("Names not unique")]
    NonUniqueName,
}

impl InvalidAxes {
    pub fn validate(axes: &[Axis]) -> Result<(), InvalidAxes> {
        use InvalidAxes::*;

        if axes.len() < 2 || axes.len() > 5 {
            return Err(Count(axes.len()));
        }
        let mut space_count = 0;
        let mut has_time = false;
        let mut has_other = false;
        let mut names = HashSet::with_capacity(axes.len());

        for a in axes.iter() {
            let n = a.name();
            if names.contains(n) {
                return Err(NonUniqueName);
            }
            names.insert(n);
            match a {
                Axis::Core(ak) => match ak {
                    CoreAxis::Space { .. } => {
                        if space_count >= 3 {
                            return Err(NSpace);
                        }
                        space_count += 1;
                    }
                    CoreAxis::Time { .. } => {
                        if space_count > 0 || has_other {
                            return Err(Order);
                        }
                        if has_time {
                            return Err(NTime);
                        }
                        has_time = true;
                    }
                    CoreAxis::Channel { .. } => {
                        if space_count > 0 {
                            return Err(Order);
                        }
                        if has_other {
                            return Err(NOther);
                        }
                        has_other = true;
                    }
                },
                Axis::Custom { .. } => {
                    if space_count > 0 {
                        return Err(Order);
                    }
                    if has_other {
                        return Err(NOther);
                    }
                    has_other = true;
                }
            }
        }
        if space_count < 2 {
            return Err(NSpace);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    fn str2ax(s: &str) -> Axis {
        serde_json::from_str(s).unwrap()
    }

    #[test]
    fn test_unit() {
        assert_eq!(
            str2ax(r#"{"name": "a", "type": "space", "unit": "foot"}"#),
            Axis::Core(CoreAxis::Space {
                name: "a".to_owned(),
                unit: Some(SpaceUnit::Foot)
            })
        );

        assert_eq!(
            str2ax(r#"{"name": "a", "type": "time", "unit": "second"}"#),
            Axis::Core(CoreAxis::Time {
                name: "a".to_owned(),
                unit: Some(TimeUnit::Second)
            })
        );

        assert_eq!(
            str2ax(r#"{"name": "a", "type": "time", "unit": "foot"}"#),
            Axis::Core(CoreAxis::Time {
                name: "a".to_owned(),
                unit: Some(TimeUnit::Other("foot".to_owned()))
            })
        );

        assert_eq!(
            str2ax(r#"{"name": "a", "type": "channel"}"#),
            Axis::Core(CoreAxis::Channel {
                name: "a".to_owned(),
                unit: None
            })
        );

        assert_eq!(
            str2ax(r#"{"name": "a", "type": "something", "unit": "somethingelse"}"#),
            Axis::Custom {
                name: "a".to_owned(),
                axis_type: Some("something".to_owned()),
                unit: Some("somethingelse".to_owned()),
            }
        );
    }
}
