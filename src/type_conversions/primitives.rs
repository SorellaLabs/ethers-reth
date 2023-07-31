use crate::type_conversions::{ToEthers, ToReth};
use std::{fmt::Debug, mem};

use ethers::types::{
    Bloom as EthersBloom, Bytes as EthersBytes, H160 as EthersH160, H256 as EthersH256,
    H64 as EthersH64, U256 as EthersU256, U64 as EthersU64,
};
use reth_primitives::{
    serde_helper::{num::U64HexOrNumber, JsonStorageKey},
    Bloom, Bytes, H160, H256, H64, U128, U256, U64, U8,
};

/// non-Uint numerical conversions
#[macro_export]
macro_rules! impl_ToEthers {
    ($t:ty, ($($u:ty),+)) => {
        $(impl ToEthers<$t> for $u {
            fn into_ethers(self) -> $t {
                const SIZE_IN: usize = mem::size_of::<$u>();
                const SIZE_OUT: usize = mem::size_of::<$t>();
                let mut buf = [0u8; 2048];

                buf[(2048-SIZE_IN)..].copy_from_slice(&Into::<[u8; SIZE_IN]>::into(self));
                let nw = TryInto::<[u8; SIZE_OUT]>::try_into(&buf[(2048-SIZE_OUT)..]).unwrap();
                <$t>::from(nw)
            }
        }
        impl_ToReth!($u, $t);
    )*
    }
}

/// non-Uint numerical conversions
#[macro_export]
macro_rules! impl_ToReth {
    ($($t:ty, $u:ty),+) => {
        $(impl ToReth<$t> for $u {
            fn into_reth(self) -> $t {
                const SIZE_IN: usize = mem::size_of::<$u>();
                const SIZE_OUT: usize = mem::size_of::<$t>();
                let mut buf = [0u8; 2048];

                buf[(2048-SIZE_IN)..].copy_from_slice(&Into::<[u8; SIZE_IN]>::into(self));
                let nw = TryInto::<[u8; SIZE_OUT]>::try_into(&buf[(2048-SIZE_OUT)..]).unwrap();
                <$t>::from(nw)
            }
        })*
    }
}

/// Uint<bits, limbs> numerical conversions
#[macro_export]
macro_rules! impl_ToEthers_Uint {
    ($t:ty, ($($u:ty),+)) => {
        $(impl ToEthers<$t> for $u {
            fn into_ethers(self) -> $t {
                const SIZE_IN: usize = mem::size_of::<$u>();
                const SIZE_OUT: usize = mem::size_of::<$t>();
                let mut buf = [0u8; 64];

                buf[(64-SIZE_IN)..].copy_from_slice(&self.to_be_bytes::<SIZE_IN>());
                <$t>::from_big_endian(&buf[(64-SIZE_OUT)..])
            }
        }
        impl_ToReth_Uint!($u, $t);
    )*
    }
}

/// Uint<bits, limbs> numerical conversions
#[macro_export]
macro_rules! impl_ToReth_Uint {
    ($($t:ty, $u:ty),+) => {
        $(impl ToReth<$t> for $u {
            fn into_reth(self) -> $t {
                const SIZE_IN: usize = mem::size_of::<$u>();
                const SIZE_OUT: usize = mem::size_of::<$t>();
                let mut buf = [0u8; 64];

                self.to_big_endian(&mut buf[(64-SIZE_IN)..]);
                <$t>::try_from_be_slice(&buf[(64-SIZE_OUT)..]).unwrap()
            }
        }
    )*
    }
}

/// type conversions for fixed size arrays
#[macro_export]
macro_rules! array_impls {
    ($($N:expr), +) => {
        $(
            impl<T, F> ToReth<[T; $N]> for [F; $N]
            where
                F: ToReth<T> + Clone,
                T: Default + Debug,
            {
                fn into_reth(self) -> [T; $N] {
                    let mut result: [T; $N] = (0..$N).map(|i| self[i].clone().into_reth()).collect::<Vec<T>>().try_into().unwrap();
                    for (i, item) in self.iter().enumerate() {
                        result[i] = item.clone().into_reth();
                    }
                    result
                }
            }

            impl<F, T> ToEthers<[F; $N]> for [T; $N]
            where
                T: ToEthers<F> + Clone,
                F: Default + Debug,
            {
                fn into_ethers(self) -> [F; $N] {
                    let mut result: [F; $N] = (0..$N).map(|i| self[i].clone().into_ethers()).collect::<Vec<F>>().try_into().unwrap();
                    for (i, item) in self.iter().enumerate() {
                        result[i] = item.clone().into_ethers();
                    }
                    result
                }
            }
        )+
    };
}

array_impls!(4, 32);

impl_ToEthers_Uint!(EthersU256, (U256, U128));
impl_ToEthers!(EthersU256, (U64, H256));

impl_ToEthers_Uint!(EthersU64, (U256));
impl_ToEthers!(EthersU64, (U64));

impl_ToEthers!(EthersH256, (H256));
impl_ToEthers!(EthersH160, (H160));
impl_ToEthers!(EthersH64, (H64));

impl_ToEthers!(EthersBloom, (Bloom));

/// U8 (reth) -> U64 (ethers)
impl ToReth<U8> for EthersU64 {
    fn into_reth(self) -> U8 {
        const SIZE_IN: usize = mem::size_of::<EthersU64>();
        const SIZE_OUT: usize = 1;
        let mut buf = [0u8; 64];

        self.to_big_endian(&mut buf[(64 - SIZE_IN)..]);
        <U8>::try_from_be_slice(&buf[(64 - SIZE_OUT)..]).unwrap()
    }
}

impl ToEthers<EthersU64> for U8 {
    fn into_ethers(self) -> U64 {
        const SIZE_IN: usize = 1;
        const SIZE_OUT: usize = mem::size_of::<EthersU64>();
        let mut buf = [0u8; 64];

        buf[(64 - SIZE_IN)..].copy_from_slice(&self.to_be_bytes::<SIZE_IN>());
        <EthersU64>::from_big_endian(&buf[(64 - SIZE_OUT)..])
    }
}

/// Bytes conversion
impl ToReth<Bytes> for EthersBytes {
    fn into_reth(self) -> Bytes {
        self.to_vec().into()
    }
}

impl ToEthers<EthersBytes> for Bytes {
    fn into_ethers(self) -> EthersBytes {
        self.to_vec().into()
    }
}

/// JsonStorageKey (reth) -> H256 (ethers)
impl ToReth<JsonStorageKey> for EthersH256 {
    fn into_reth(self) -> JsonStorageKey {
        JsonStorageKey(self.into())
    }
}

/// JsonStorageKey (ethers) -> H256 (reth)
impl ToEthers<H256> for JsonStorageKey {
    fn into_ethers(self) -> H256 {
        self.0
    }
}

/// JsonStorageKey (reth) -> H256 (ethers)
impl ToReth<U64HexOrNumber> for EthersU256 {
    fn into_reth(self) -> U64HexOrNumber {
        let u: U64 = self.into_reth();
        U64HexOrNumber::from(u)
    }
}

/// JsonStorageKey (ethers) -> H256 (reth)
impl ToEthers<EthersU256> for U64HexOrNumber {
    fn into_ethers(self) -> EthersU256 {
        U64::from(self).into_ethers()
    }
}

#[cfg(test)]
mod tests {
    use crate::type_conversions::*;
    use ethers::types::{Bloom as EthersBloom, U256 as EthersU256, U64 as EthersU64};
    use reth_primitives::{hex_literal::hex, Bloom, U256, U64, U8};

    #[test]
    fn bloom() {
        let hex = hex!(
            "000000000000000000810000000000000000000000000000000000020000000000000000000000000000008000"
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
            "000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000"
            "000000000000000000000000000000000000000000000000000000280000000000400000800000004000000000"
            "000000000000000000000000000000000000000000000000000000000000100000100000000000000000000000"
            "00000000001400000000000000008000000000000000000000000000000000"
        );
        let r: Bloom = Bloom::from(hex);
        let e: EthersBloom = EthersBloom::from(hex);
        assert_eq!(r, e.into_reth());
        assert_eq!(e, r.into_ethers());
    }

    #[test]
    fn u256() {
        let r: U256 = U256::from(1);
        let e: EthersU256 = EthersU256::from(1);
        assert_eq!(r, e.into_reth());
        assert_eq!(e, r.into_ethers());
    }

    #[test]
    fn u64() {
        let r: U64 = U64::from(1);
        let e: EthersU64 = EthersU64::from(1);
        assert_eq!(r, e.into_reth());
        assert_eq!(e, r.into_ethers());
    }

    #[test]
    fn ethers_u64_u8() {
        let r: U8 = U8::from(1);
        let e: EthersU64 = EthersU64::from(1);
        assert_eq!(r, e.into_reth());
        assert_eq!(e, r.into_ethers());
    }

    #[test]
    fn ethers_u64_u256() {
        let r: U256 = U256::from(1);
        let e: EthersU64 = EthersU64::from(1);
        assert_eq!(r, e.into_reth());
        assert_eq!(e, r.into_ethers());
    }
}
