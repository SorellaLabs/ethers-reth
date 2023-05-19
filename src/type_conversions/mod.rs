use std::{
    collections::{BTreeMap, HashSet},
    hash::Hash,
};

use ethers::prelude::k256::sha2::digest::typenum::Ord;

// Numerous acts of type terrorism having been commited during the making of this program. Please
// forgive us.
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

// -----------------------------------------------

/// generic HashSet<> -> Vec<> conversion
/// mainly for Vec<> (ethers) -> Hashset<> (reth)
impl<T, F> ToReth<HashSet<T>> for Vec<F>
where
    F: ToReth<T>,
    T: Hash + Eq,
{
    fn into_reth(self) -> HashSet<T> {
        self.into_iter().map(|x| x.into_reth()).collect()
    }
}

impl<F, T> ToEthers<HashSet<F>> for Vec<T>
where
    T: ToEthers<F>,
    F: Hash + Eq,
{
    fn into_ethers(self) -> HashSet<F> {
        self.into_iter().map(|x| x.into_ethers()).collect()
    }
}

// -----------------------------------------------

/// generic BTreeMap<> conversion
impl<T, K, F, U> ToReth<BTreeMap<T, K>> for BTreeMap<F, U>
where
    F: ToReth<T>,
    U: ToReth<K>,
    BTreeMap<T, K>: FromIterator<(T, K)>,
{
    fn into_reth(self) -> BTreeMap<T, K> {
        self.into_iter().map(|x| (x.0.into_reth(), x.1.into_reth())).collect::<BTreeMap<T, K>>()
    }
}

impl<F, U, T, K> ToEthers<BTreeMap<F, U>> for BTreeMap<T, K>
where
    T: ToEthers<F>,
    K: ToEthers<U>,
    BTreeMap<F, U>: FromIterator<(F, U)>,
{
    fn into_ethers(self) -> BTreeMap<F, U> {
        self.into_iter().map(|x| (x.0.into_ethers(), x.1.into_ethers())).collect()
    }
}

// -----------------------------------------------

/// generic Tuple of 2 conversion
impl<T, K, F, U> ToReth<(T, K)> for (F, U)
where
    F: ToReth<T> + Clone,
    U: ToReth<K> + Clone,
{
    fn into_reth(self) -> (T, K) {
        (self.0.into_reth(), self.1.into_reth())
    }
}

impl<F, U, T, K> ToEthers<(F, U)> for (T, K)
where
    T: ToEthers<F> + Clone,
    K: ToEthers<U> + Clone,
{
    fn into_ethers(self) -> (F, U) {
        (self.0.into_ethers(), self.1.into_ethers())
    }
}
