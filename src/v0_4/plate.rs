use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::util::ZPath;

pub type AcquisitionId = u64;
pub type Timestamp = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Acquisition {
    id: AcquisitionId,
    name: Option<String>,
    #[serde(rename = "maximumfieldcount")]
    maximum_field_count: Option<usize>,
    description: Option<String>,
    #[serde(rename = "starttime")]
    start_time: Option<Timestamp>,
    #[serde(rename = "endtime")]
    end_time: Option<Timestamp>,
}

fn validate_acquisitions(acquisitions: &[Acquisition]) -> Result<(), InvalidPlate> {
    let mut ids = HashSet::with_capacity(acquisitions.len());
    for acq in acquisitions.iter() {
        if !ids.insert(acq.id) {
            return Err(InvalidPlate::NonUniqueAcquisitionId);
        }
        let Some(start) = acq.start_time else {continue};
        let Some(end) = acq.end_time else {continue};
        if end < start {
            return Err(InvalidPlate::AcquisitionTime);
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlateWell {
    path: ZPath,
    row_index: usize,
    column_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plate {
    acquisitions: Option<Vec<Acquisition>>,
    columns: Vec<Index>,
    field_count: Option<usize>,
    name: Option<String>,
    rows: Vec<Index>,
    version: Option<String>,
    wells: Vec<PlateWell>,
}

#[derive(Debug, Clone, Error)]
pub enum InvalidPlate {
    #[error("Well indices are not consistent with their names")]
    InconsistentWells,
    #[error("No well at index {0}")]
    NonexistentWell(usize),
    #[error("Row or column indices are not unique")]
    NonUniqueIndex,
    #[error("Index names must be alphanumeric")]
    InvalidIndex,
    #[error("Acquisition IDs are not unique")]
    NonUniqueAcquisitionId,
    #[error("Acquisition ends before it starts")]
    AcquisitionTime,
}

fn validate_index(idxs: &[Index]) -> Result<(), InvalidPlate> {
    let mut names = HashSet::with_capacity(idxs.len());
    for name in idxs.iter().map(|idx| idx.name.as_str()) {
        if !names.insert(name) {
            return Err(InvalidPlate::NonUniqueIndex);
        }
        if !name.chars().all(char::is_alphanumeric) {
            return Err(InvalidPlate::InvalidIndex);
        }
    }
    Ok(())
}

impl Plate {
    pub fn validate(&self) -> Result<(), InvalidPlate> {
        validate_index(self.rows.as_slice())?;
        validate_index(self.columns.as_slice())?;
        if let Some(acqs) = self.acquisitions.as_ref() {
            validate_acquisitions(acqs.as_slice())?;
        }
        for well in self.wells.iter() {
            let row_name = self
                .rows
                .get(well.row_index)
                .ok_or(InvalidPlate::NonexistentWell(well.row_index))?
                .name
                .as_str();
            let col_name = self
                .columns
                .get(well.column_index)
                .ok_or(InvalidPlate::NonexistentWell(well.column_index))?
                .name
                .as_str();

            if well.path != format!("{row_name}/{col_name}") {
                return Err(InvalidPlate::InconsistentWells);
            }
        }
        Ok(())
    }

    pub fn acquisition_ids(&self) -> HashSet<AcquisitionId> {
        self.acquisitions
            .as_ref()
            .map(|acs| acs.iter().map(|a| a.id).collect())
            .unwrap_or(HashSet::with_capacity(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    const EXAMPLE1: &'static str = r#"
        {
            "acquisitions": [
                {
                    "id": 1,
                    "maximumfieldcount": 2,
                    "name": "Meas_01(2012-07-31_10-41-12)",
                    "starttime": 1343731272000
                },
                {
                    "id": 2,
                    "maximumfieldcount": 2,
                    "name": "Meas_02(201207-31_11-56-41)",
                    "starttime": 1343735801000
                }
            ],
            "columns": [
                {
                    "name": "1"
                },
                {
                    "name": "2"
                },
                {
                    "name": "3"
                }
            ],
            "field_count": 4,
            "name": "test",
            "rows": [
                {
                    "name": "A"
                },
                {
                    "name": "B"
                }
            ],
            "version": "0.4",
            "wells": [
                {
                    "path": "A/1",
                    "rowIndex": 0,
                    "columnIndex": 0
                },
                {
                    "path": "A/2",
                    "rowIndex": 0,
                    "columnIndex": 1
                },
                {
                    "path": "A/3",
                    "rowIndex": 0,
                    "columnIndex": 2
                },
                {
                    "path": "B/1",
                    "rowIndex": 1,
                    "columnIndex": 0
                },
                {
                    "path": "B/2",
                    "rowIndex": 1,
                    "columnIndex": 1
                },
                {
                    "path": "B/3",
                    "rowIndex": 1,
                    "columnIndex": 2
                }
            ]
        }
    "#;

    const EXAMPLE2: &'static str = r#"
        {
            "acquisitions": [
                {
                    "id": 1,
                    "maximumfieldcount": 1,
                    "name": "single acquisition",
                    "starttime": 1343731272000
                }
            ],
            "columns": [
                {
                    "name": "1"
                },
                {
                    "name": "2"
                },
                {
                    "name": "3"
                },
                {
                    "name": "4"
                },
                {
                    "name": "5"
                },
                {
                    "name": "6"
                },
                {
                    "name": "7"
                },
                {
                    "name": "8"
                },
                {
                    "name": "9"
                },
                {
                    "name": "10"
                },
                {
                    "name": "11"
                },
                {
                    "name": "12"
                }
            ],
            "field_count": 1,
            "name": "sparse test",
            "rows": [
                {
                    "name": "A"
                },
                {
                    "name": "B"
                },
                {
                    "name": "C"
                },
                {
                    "name": "D"
                },
                {
                    "name": "E"
                },
                {
                    "name": "F"
                },
                {
                    "name": "G"
                },
                {
                    "name": "H"
                }
            ],
            "version": "0.4",
            "wells": [
                {
                    "path": "C/5",
                    "rowIndex": 2,
                    "columnIndex": 4
                },
                {
                    "path": "D/7",
                    "rowIndex": 3,
                    "columnIndex": 6
                }
            ]
        }
    "#;

    #[test]
    fn examples() {
        let p1: Plate = serde_json::from_str(EXAMPLE1).unwrap();
        p1.validate().unwrap();

        let p2: Plate = serde_json::from_str(EXAMPLE2).unwrap();
        p2.validate().unwrap();
    }
}
