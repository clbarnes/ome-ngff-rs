use crate::util::{InconsistentDimensionality, Ndim, ZPath};
use std::collections::HashMap;
use thiserror::Error;

use super::{
    axes::{Axis, InvalidAxes},
    coordinate_transformations::{
        CoordinateTransformation, InvalidCoordinateTransforms, Transform,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiscaleDataset {
    path: ZPath,
    coordinate_transformations: Vec<CoordinateTransformation>,
}

impl MultiscaleDataset {
    pub fn validate(
        &self,
        ndim: Option<usize>,
    ) -> Result<Option<usize>, InvalidCoordinateTransforms> {
        InvalidCoordinateTransforms::validate(
            self.coordinate_transformations.as_slice(),
            true,
            ndim,
        )
    }
}

#[derive(Debug, Clone, Error)]
pub enum InvalidMultiscale {
    #[error(transparent)]
    Axes(#[from] InvalidAxes),
    #[error(transparent)]
    Transforms(#[from] InvalidCoordinateTransforms),
    #[error(transparent)]
    Dimensions(#[from] InconsistentDimensionality),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Multiscale {
    axes: Vec<Axis>,
    datasets: Vec<MultiscaleDataset>,
    coordinate_transformations: Option<Vec<CoordinateTransformation>>,
    name: Option<Value>,
    version: Option<Value>,
    #[serde(rename = "type")]
    multiscale_type: Option<Value>,
    metadata: Option<HashMap<String, Value>>,
}

impl Ndim for Multiscale {
    fn ndim(&self) -> usize {
        self.axes.len()
    }
}

impl Multiscale {
    pub fn validate(&self) -> Result<(), InvalidMultiscale> {
        InvalidAxes::validate(self.axes.as_slice())?;
        let ndim = self.ndim();
        for ds in self.datasets.iter() {
            ds.validate(Some(ndim))?;
        }
        if let Some(cs) = &self.coordinate_transformations {
            InvalidCoordinateTransforms::validate(cs.as_slice(), false, Some(ndim))?;
        }
        Ok(())
    }
}

impl Transform for (&Multiscale, usize) {
    fn transform(&self, coord: &mut [f64]) -> Result<(), InconsistentDimensionality> {
        let ds = &self.0.datasets[self.1];
        ds.coordinate_transformations.as_slice().transform(coord)?;
        if let Some(cs) = &self.0.coordinate_transformations {
            cs.as_slice().transform(coord)?;
        }
        Ok(())
    }

    fn rev_transform(&self, coord: &mut [f64]) -> Result<(), InconsistentDimensionality> {
        if let Some(cs) = &self.0.coordinate_transformations {
            cs.as_slice().rev_transform(coord)?;
        }
        let ds = &self.0.datasets[self.1];
        ds.coordinate_transformations
            .as_slice()
            .rev_transform(coord)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    const EXAMPLE: &'static str = r#"
        {
            "version": "0.4",
            "name": "example",
            "axes": [
                {"name": "t", "type": "time", "unit": "millisecond"},
                {"name": "c", "type": "channel"},
                {"name": "z", "type": "space", "unit": "micrometer"},
                {"name": "y", "type": "space", "unit": "micrometer"},
                {"name": "x", "type": "space", "unit": "micrometer"}
            ],
            "datasets": [
                {
                    "path": "0",
                    "coordinateTransformations": [{
                        "type": "scale",
                        "scale": [1.0, 1.0, 0.5, 0.5, 0.5]
                    }]
                },
                {
                    "path": "1",
                    "coordinateTransformations": [{
                        "type": "scale",
                        "scale": [1.0, 1.0, 1.0, 1.0, 1.0]
                    }]
                },
                {
                    "path": "2",
                    "coordinateTransformations": [{
                        "type": "scale",
                        "scale": [1.0, 1.0, 2.0, 2.0, 2.0]
                    }]
                }
            ],
            "coordinateTransformations": [{
                "type": "scale",
                "scale": [0.1, 1.0, 1.0, 1.0, 1.0]
            }],
            "type": "gaussian",
            "metadata": {
                "description": "the fields in metadata depend on the downscaling implementation. Here, the parameters passed to the skimage function are given",
                "method": "skimage.transform.pyramid_gaussian",
                "version": "0.16.1",
                "args": "[true]",
                "kwargs": {"multichannel": true}
            }
        }
    "#;

    #[test]
    fn deser_example() {
        let ms: Multiscale = serde_json::from_str(EXAMPLE).unwrap();
        ms.validate().unwrap();
    }
}
