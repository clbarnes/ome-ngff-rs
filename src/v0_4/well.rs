use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::util::ZPath;

use super::plate::AcquisitionId;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Well {
    version: Option<String>,
    images: Vec<FieldOfView>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct FieldOfView {
    path: ZPath,
    acquisition: Option<AcquisitionId>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Error)]
pub enum InvalidWell {
    #[error("Field of view paths are not unique")]
    NonUniquePaths,
    #[error("Unknown acquisition ID {0}")]
    UnknownAcquisition(AcquisitionId),
    #[error("Acquisition ID required but not present")]
    NoAcquisition,
    #[error("Path must be alphanumeric")]
    InvalidPath,
}

impl Well {
    pub fn validate(
        &self,
        acquisitions: Option<HashSet<AcquisitionId>>,
    ) -> Result<(), InvalidWell> {
        let mut paths = HashSet::with_capacity(self.images.len());
        for im in self.images.iter() {
            if !im.path.chars().all(char::is_alphanumeric) {
                return Err(InvalidWell::InvalidPath);
            }

            if !paths.insert(im.path.as_str()) {
                return Err(InvalidWell::NonUniquePaths);
            }

            if let Some(acqs) = acquisitions.as_ref() {
                if let Some(acq) = im.acquisition.as_ref() {
                    if !acqs.contains(acq) {
                        return Err(InvalidWell::UnknownAcquisition(*acq));
                    }
                } else {
                    return Err(InvalidWell::NoAcquisition);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    const EXAMPLE1: &'static str = r#"
    {
        "images": [
            {
                "acquisition": 1,
                "path": "0"
            },
            {
                "acquisition": 1,
                "path": "1"
            },
            {
                "acquisition": 2,
                "path": "2"
            },
            {
                "acquisition": 2,
                "path": "3"
            }
        ],
        "version": "0.4"
    }
    "#;

    const EXAMPLE2: &'static str = r#"
    {
        "images": [
            {
                "acquisition": 0,
                "path": "0"
            },
            {
                "acquisition": 3,
                "path": "1"
            }
        ],
        "version": "0.4"
    }
    "#;

    #[test]
    fn examples() {
        let w1: Well = serde_json::from_str(EXAMPLE1).unwrap();
        w1.validate(None).unwrap();

        let w2: Well = serde_json::from_str(EXAMPLE2).unwrap();
        w2.validate(None).unwrap();
    }
}
