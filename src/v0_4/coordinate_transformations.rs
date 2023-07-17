use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::util::{InconsistentDimensionality, MaybeNdim};

pub trait Transform {
    fn transform(&self, coord: &mut [f64]) -> Result<(), InconsistentDimensionality>;

    fn rev_transform(&self, coord: &mut [f64]) -> Result<(), InconsistentDimensionality>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TranslationOrPath {
    Path(String),
    Translation(Vec<f64>),
}

impl MaybeNdim for TranslationOrPath {
    fn maybe_ndim(&self) -> Option<usize> {
        match self {
            Self::Translation(v) => Some(v.len()),
            _ => None,
        }
    }
}

impl Transform for TranslationOrPath {
    fn transform(&self, coord: &mut [f64]) -> Result<(), InconsistentDimensionality> {
        InconsistentDimensionality::check_dim_opts(self.maybe_ndim(), Some(coord.len()))?;
        match self {
            Self::Path(_) => unimplemented!(),
            Self::Translation(v) => {
                for (c, t) in coord.iter_mut().zip(v.iter()) {
                    *c += t;
                }
            }
        };
        Ok(())
    }

    fn rev_transform(&self, coord: &mut [f64]) -> Result<(), InconsistentDimensionality> {
        InconsistentDimensionality::check_dim_opts(self.maybe_ndim(), Some(coord.len()))?;
        match self {
            Self::Path(_) => unimplemented!(),
            Self::Translation(v) => {
                for (c, t) in coord.iter_mut().zip(v.iter()) {
                    *c -= t;
                }
            }
        };
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScaleOrPath {
    Path(String),
    Scale(Vec<f64>),
}

impl MaybeNdim for ScaleOrPath {
    fn maybe_ndim(&self) -> Option<usize> {
        match self {
            Self::Scale(v) => Some(v.len()),
            _ => None,
        }
    }
}

impl Transform for ScaleOrPath {
    fn transform(&self, coord: &mut [f64]) -> Result<(), InconsistentDimensionality> {
        InconsistentDimensionality::check_dim_opts(self.maybe_ndim(), Some(coord.len()))?;
        match self {
            Self::Path(_) => unimplemented!(),
            Self::Scale(v) => {
                for (c, t) in coord.iter_mut().zip(v.iter()) {
                    *c *= t;
                }
            }
        };
        Ok(())
    }

    fn rev_transform(&self, coord: &mut [f64]) -> Result<(), InconsistentDimensionality> {
        InconsistentDimensionality::check_dim_opts(self.maybe_ndim(), Some(coord.len()))?;
        match self {
            Self::Path(_) => unimplemented!(),
            Self::Scale(v) => {
                for (c, t) in coord.iter_mut().zip(v.iter()) {
                    *c /= t;
                }
            }
        };
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CoordinateTransformation {
    Identity,
    Translation(TranslationOrPath),
    Scale(ScaleOrPath),
}

impl Default for CoordinateTransformation {
    fn default() -> Self {
        Self::Identity
    }
}

impl MaybeNdim for CoordinateTransformation {
    fn maybe_ndim(&self) -> Option<usize> {
        match self {
            Self::Translation(t) => t.maybe_ndim(),
            Self::Scale(t) => t.maybe_ndim(),
            _ => None,
        }
    }
}

impl Transform for CoordinateTransformation {
    fn transform(&self, coord: &mut [f64]) -> Result<(), InconsistentDimensionality> {
        match self {
            Self::Identity => Ok(()),
            Self::Translation(t) => t.transform(coord),
            Self::Scale(t) => t.transform(coord),
        }
    }

    fn rev_transform(&self, coord: &mut [f64]) -> Result<(), InconsistentDimensionality> {
        match self {
            Self::Identity => Ok(()),
            Self::Translation(t) => t.rev_transform(coord),
            Self::Scale(t) => t.rev_transform(coord),
        }
    }
}

impl Transform for &[CoordinateTransformation] {
    fn transform(&self, coord: &mut [f64]) -> Result<(), InconsistentDimensionality> {
        self.iter().try_for_each(|t| t.transform(coord))
    }

    fn rev_transform(&self, coord: &mut [f64]) -> Result<(), InconsistentDimensionality> {
        self.iter().rev().try_for_each(|t| t.transform(coord))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InvalidCoordinateTransforms {
    #[error("Missing scale transform")]
    MissingScale,
    #[error("Transformations are ordered incorrectly")]
    Order,
    #[error("Unsupported translation: {0}")]
    Unsupported(String),
    #[error("Invalid count: {0}")]
    Count(String),
    #[error(transparent)]
    Dimensions(#[from] InconsistentDimensionality),
}

impl InvalidCoordinateTransforms {
    pub fn validate(
        cs: &[CoordinateTransformation],
        require_scale: bool,
        mut ndim: Option<usize>,
    ) -> Result<Option<usize>, Self> {
        if require_scale && cs.is_empty() {
            return Err(InvalidCoordinateTransforms::MissingScale);
        }
        let mut has_scale = false;
        let mut has_transl = false;

        for c in cs.iter() {
            ndim = InconsistentDimensionality::check_dim_opts(ndim, c.maybe_ndim())?;
            match c {
                CoordinateTransformation::Identity => {
                    return Err(
                        InvalidCoordinateTransforms::Unsupported("identity".to_owned()),
                    )
                }
                CoordinateTransformation::Translation(_) => {
                    if !has_scale {
                        return Err(InvalidCoordinateTransforms::Order);
                    }
                    if has_transl {
                        return Err(InvalidCoordinateTransforms::Count(
                            "Multiple translations found".to_owned(),
                        ));
                    } else {
                        has_transl = true;
                    }
                }
                CoordinateTransformation::Scale(_) => {
                    if has_scale {
                        return Err(InvalidCoordinateTransforms::Count(
                            "Multiple scales found".to_owned(),
                        ));
                    } else {
                        has_scale = true;
                    }
                }
            }
        }
        Ok(ndim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    fn str2ct(s: &str) -> CoordinateTransformation {
        serde_json::from_str(s).unwrap()
    }

    #[test]
    fn test_transforms() {
        assert_eq!(
            str2ct(r#"{"type": "identity"}"#),
            CoordinateTransformation::Identity,
        );
        assert_eq!(
            str2ct(r#"{"type": "translation", "path": "path/to/whatever"}"#),
            CoordinateTransformation::Translation(TranslationOrPath::Path(
                "path/to/whatever".to_owned()
            )),
        );
        assert_eq!(
            str2ct(r#"{"type": "scale", "scale": [1,2,3]}"#),
            CoordinateTransformation::Scale(ScaleOrPath::Scale(vec![1.0, 2.0, 3.0])),
        );
    }
}
