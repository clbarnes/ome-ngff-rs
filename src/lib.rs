use arrayvec::ArrayVec;

const MAX_DIMS: usize = 5;
pub type Coord<T> = ArrayVec<T, MAX_DIMS>;
pub type RealCoord = Coord<f64>;

#[cfg(any(feature = "v0_4", feature = "v0_5"))]
mod util;

#[cfg(feature = "v0_4")]
pub mod v0_4;

#[cfg(feature = "v0_5")]
pub mod v0_5;
