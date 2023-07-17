mod axes;
mod coordinate_transformations;
mod image_label;
mod multiscale;
mod plate;
mod well;

pub use axes::{Axis, CoreAxis, InvalidAxes, SpaceUnit, TimeUnit};
pub use coordinate_transformations::{
    CoordinateTransformation, InvalidCoordinateTransforms, ScaleOrPath, Transform,
    TranslationOrPath,
};
pub use image_label::{Color, ImageLabel, InvalidImageLabel, Properties, Source};
pub use multiscale::{InvalidMultiscale, Multiscale, MultiscaleDataset};
pub use plate::{Acquisition, AcquisitionId, Index, InvalidPlate, Plate, PlateWell};
use serde::{Deserialize, Serialize};
pub use well::{FieldOfView, InvalidWell, Well};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NgffMetadata {
    multiscales: Option<Vec<Multiscale>>,
    labels: Option<Vec<String>>,
    #[serde(rename = "image-label")]
    image_label: Option<ImageLabel>,
    plate: Option<Plate>,
    well: Option<Well>,
}
