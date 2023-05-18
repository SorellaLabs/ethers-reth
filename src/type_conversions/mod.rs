pub mod access_list;
pub mod block;
pub mod primitives;
pub mod rpc;
pub mod withdraw;

// -----------------------------------------------
/// conversion traits
pub trait ToReth<T> {
    /// Ethers -> Reth
    fn into_reth(self) -> T;
}

pub trait ToEthers<T> {
    /// Reth -> Ethers
    fn into_ethers(self) -> T;
}

// -----------------------------------------------
/// generic as_ref conversion
impl<T, F> ToReth<F> for &T
where
    T: ToReth<F> + ?Sized + Clone,
{
    fn into_reth(self) -> F {
        (*self).clone().into_reth()
    }
}

impl<F, T> ToEthers<T> for &F
where
    F: ToEthers<T> + ?Sized + Clone,
{
    fn into_ethers(self) -> T {
        (*self).clone().into_ethers()
    }
}

// -----------------------------------------------
/// generic Vec<> conversion
impl<T, F> ToReth<Vec<T>> for Vec<F>
where
    F: ToReth<T>,
{
    fn into_reth(self) -> Vec<T> {
        self.into_iter().map(|x| x.into_reth()).collect()
    }
}

impl<F, T> ToEthers<Vec<F>> for Vec<T>
where
    T: ToEthers<F>,
{
    fn into_ethers(self) -> Vec<F> {
        self.into_iter().map(|x| x.into_ethers()).collect()
    }
}

// -----------------------------------------------
/// generic Option<> conversion
impl<T, F> ToReth<Option<T>> for Option<F>
where
    F: ToReth<T>,
{
    fn into_reth(self) -> Option<T> {
        self.map(|x| x.into_reth())
    }
}

impl<F, T> ToEthers<Option<F>> for Option<T>
where
    T: ToEthers<F>,
{
    fn into_ethers(self) -> Option<F> {
        self.map(|x| x.into_ethers())
    }
}
