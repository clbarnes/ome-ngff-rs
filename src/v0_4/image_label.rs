use std::collections::HashMap;
use thiserror::Error;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type LabelType = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    #[serde(rename = "label-value")]
    label_value: LabelType,
    #[serde(skip_serializing_if = "Option::is_none")]
    rgba: Option<[u8; 4]>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageLabel {
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    colors: Option<Vec<Color>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<Vec<Properties>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<Source>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Error)]
pub enum InvalidImageLabel {
    #[error("Label values are not unique")]
    NonUniqueLabels,
}

impl ImageLabel {
    pub fn validate(&self) -> Result<(), InvalidImageLabel> {
        let lcs = self.label_colors();
        if !lcs.is_empty() && self.colors.as_ref().unwrap().len() != lcs.len() {
            return Err(InvalidImageLabel::NonUniqueLabels);
        }

        let lps = self.label_properties();
        if !lps.is_empty() && self.properties.as_ref().unwrap().len() != lps.len() {
            return Err(InvalidImageLabel::NonUniqueLabels);
        }
        Ok(())
    }

    pub fn label_colors(&self) -> HashMap<LabelType, &[u8; 4]> {
        let Some(cols) = &self.colors else {
            return HashMap::with_capacity(0);
        };
        cols.iter()
            .fold(HashMap::with_capacity(cols.len()), |mut accum, el| {
                if let Some(rgba) = el.rgba.as_ref() {
                    accum.insert(el.label_value, rgba);
                }
                accum
            })
    }

    pub fn label_properties(&self) -> HashMap<LabelType, &HashMap<String, Value>> {
        let Some(props) = &self.properties else {
            return HashMap::with_capacity(0);
        };
        props
            .iter()
            .fold(HashMap::with_capacity(props.len()), |mut accum, el| {
                accum.insert(el.label_value, &el.metadata);
                accum
            })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Properties {
    #[serde(rename = "label-value")]
    label_value: LabelType,
    #[serde(flatten)]
    metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
}

impl Default for Source {
    fn default() -> Self {
        Self {
            image: Some("../../".to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    const EXAMPLE: &str = r#"
    {
        "version": "0.4",
        "colors": [
        {
            "label-value": 1,
            "rgba": [255, 255, 255, 255]
        },
        {
            "label-value": 4,
            "rgba": [0, 255, 255, 128]
        }
        ],
        "properties": [
        {
            "label-value": 1,
            "area (pixels)": 1200,
            "class": "foo"
        },
        {
            "label-value": 4,
            "area (pixels)": 1650
        }
        ],
        "source": {
        "image": "../../"
        }
    }
    "#;

    #[test]
    fn test_example() {
        let im: ImageLabel = serde_json::from_str(EXAMPLE).unwrap();
        im.validate().unwrap();
    }
}
