use thiserror::Error;

pub type ZPath = String;

/// variant_from_data!(EnumType, VariantName, DataType)
///
/// adds `From<D>` for an enum with a variant containing D
///
/// N.B. this is also handled by enum_delegate::implement
// #[macro_export]
macro_rules! variant_from_data {
    ($enum:ty, $variant:ident, $data_type:ty) => {
        impl std::convert::From<$data_type> for $enum {
            fn from(c: $data_type) -> Self {
                <$enum>::$variant(c)
            }
        }
    };
}

pub(crate) use variant_from_data;

// macro_rules! transitive_into {
//     ($target:ty, $intermediate:ty) => {
//         impl<T> std::convert::From<T> for $target where T: std::convert::Into<$intermediate> {
//             fn from(t: T) -> Self {
//                 Self::from(T.into::<$intermediate>)
//             }
//         }
//     };
// }

// pub(crate) use transitive_into;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Error)]
#[error("Inconsistent dimensionalities: {0}, {1}")]
pub struct InconsistentDimensionality(usize, usize);

impl InconsistentDimensionality {
    pub fn check_dims(dim1: usize, dim2: usize) -> Result<usize, Self> {
        if dim1 == dim2 {
            Ok(dim1)
        } else {
            Err(Self(dim1, dim2))
        }
    }

    pub fn check_dim_opts(dim1: Option<usize>, dim2: Option<usize>) -> Result<Option<usize>, Self> {
        let Some(n1) = dim1 else {
            return Ok(dim2);
        };
        let Some(n2) = dim2 else {
            return Ok(dim1);
        };
        Self::check_dims(n1, n2).map(Some)
    }
}

pub trait Ndim {
    fn ndim(&self) -> usize;

    fn same_ndim<T: Ndim>(&self, other: &T) -> Result<usize, InconsistentDimensionality> {
        InconsistentDimensionality::check_dims(self.ndim(), other.ndim())
    }
}

pub trait MaybeNdim {
    fn maybe_ndim(&self) -> Option<usize>;

    fn union_ndim<T: MaybeNdim>(
        &self,
        other: &T,
    ) -> Result<Option<usize>, InconsistentDimensionality> {
        let Some(n1) = self.maybe_ndim() else {
            return Ok(other.maybe_ndim());
        };
        let Some(n2) = other.maybe_ndim() else {
            return Ok(Some(n1));
        };
        InconsistentDimensionality::check_dims(n1, n2).map(Some)
    }

    fn validate_ndim(&self) -> Result<(), InconsistentDimensionality> {
        Ok(())
    }
}

impl<T: Ndim> MaybeNdim for T {
    fn maybe_ndim(&self) -> Option<usize> {
        Some(self.ndim())
    }
}
