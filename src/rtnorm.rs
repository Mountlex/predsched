use num_traits::Float;
use rand::distributions::Distribution;
use rand::Rng;
use std::fmt;

pub struct TruncatedNormal<F>
where
    F: Float,
{
    lower: Option<F>,
    upper: Option<F>,
    mean: F,
    std_dev: F,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// The lower bound is not less than the upper bound.
    BadBounds,
    /// The standard deviation or other dispersion parameter is not finite.
    BadVariance,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Error::BadBounds => "lower >= upper or mean is not between lower and upper",
            Error::BadVariance => "variation parameter is non-finite in (log)normal distribution",
        })
    }
}

impl std::error::Error for Error {}

impl<F> TruncatedNormal<F>
where
    F: Float,
{
    #[inline]
    pub fn new(lower: F, upper: F, mean: F, std_dev: F) -> Result<TruncatedNormal<F>, Error> {
        if lower >= upper || mean <= lower || mean >= upper {
            return Err(Error::BadBounds);
        }
        Ok(TruncatedNormal {
            lower: Some(lower),
            upper: Some(upper),
            mean,
            std_dev,
        })
    }

    #[inline]
    pub fn upper_truncation(upper: F, mean: F, std_dev: F) -> Result<TruncatedNormal<F>, Error> {
        if  mean >= upper {
            return Err(Error::BadBounds);
        }
        Ok(TruncatedNormal {
            lower: None,
            upper: Some(upper),
            mean,
            std_dev,
        })
    }

    #[inline]
    pub fn lower_truncation(lower: F, mean: F, std_dev: F) -> Result<TruncatedNormal<F>, Error> {
        if  mean <= lower {
            return Err(Error::BadBounds);
        }
        Ok(TruncatedNormal {
            lower: Some(lower),
            upper: None,
            mean,
            std_dev,
        })
    }
}

impl<F> Distribution<F> for TruncatedNormal<F>
where
    F: Float,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> F {
        
    }
}
