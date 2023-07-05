use super::{ToEthers, ToReth};

use ethers::types::{
    BlockId as EthersBlockId, BlockNumber as EthersBlockNumber, H256 as EthersH256,
    GethDebugTracingCallOptions as EthersDebugTracingCallOptions,
    GethTrace as EthersGethTrace,
    GethTraceFrame as EthersGethTraceFrame,
    DefaultFrame,
    NoopFrame,
    FourByteFrame,
    CallFrame,
    PreStateFrame,
};
use reth_primitives::{BlockId, BlockNumberOrTag, H256};
use reth_rpc_types::trace::geth::{GethDebugTracingCallOptions, GethDebugTracingOptions, GethTrace};


/// GethDebugTracingCallOptions (ethers) -> (reth)
impl ToReth<GethDebugTracingCallOptions> for EthersDebugTracingCallOptions {
    fn into_reth(self) -> GethDebugTracingCallOptions {
        GethDebugTracingCallOptions {
            tracing_options: GethDebugTracingOptions::default(),
            state_overrides: None,
            block_overrides: None,
        }
    }
}

/// GethTrace (reth) -> (ethers)
impl ToEthers<EthersGethTrace> for GethTrace {
    fn into_ethers(self) -> EthersGethTrace {
        match self {
            GethTrace::Default(default_frame) => EthersGethTrace::Known(Default(DefaultFrame::default())),
            GethTrace::CallTracer(call_frame) => EthersGethTrace::Known(EthersGethTraceFrame::CallTracer(CallFrame::default())),
            GethTrace::FourByteTracer(four_byte_frame) => EthersGethTrace::Known(EthersGethTraceFrame::FourByteTracer(FourByteFrame::default())),
            GethTrace::PreStateTracer(pre_state_frame) => EthersGethTrace::Known(EthersGethTraceFrame::PreStateTracer(PreStateFrame::default())),
            GethTrace::NoopTracer(noop_frame) => EthersGethTrace::Known(EthersGethTraceFrame::NoopTracer(NoopFrame::default())),
            GethTrace::JS(value) => EthersGethTrace::Unknown(EthersGethTraceFrame::Unknown(value)),
        }
    }
}

/// BlockId (ethers) -> (reth)
impl ToReth<BlockId> for EthersBlockId {
    fn into_reth(self) -> BlockId {
        match self {
            EthersBlockId::Hash(hash) => {
                BlockId::Hash(<EthersH256 as ToReth<H256>>::into_reth(hash).into())
            }
            EthersBlockId::Number(number) => {
                BlockId::Number(BlockNumberOrTag::Number(number.as_number().unwrap().as_u64()))
            }
        }
    }
}

/// BlockId (reth) -> (ethers)
impl ToEthers<EthersBlockId> for BlockId {
    fn into_ethers(self) -> EthersBlockId {
        match self {
            BlockId::Hash(hash) => EthersBlockId::Hash(hash.block_hash.into_ethers()),
            BlockId::Number(number) => EthersBlockId::Number(number.into_ethers()),
        }
    }
}

// -----------------------------------------------

/// BlockNumber (ethers) -> (reth)
impl ToReth<BlockNumberOrTag> for EthersBlockNumber {
    fn into_reth(self) -> BlockNumberOrTag {
        match self {
            EthersBlockNumber::Latest => BlockNumberOrTag::Latest,
            EthersBlockNumber::Finalized => BlockNumberOrTag::Finalized,
            EthersBlockNumber::Safe => BlockNumberOrTag::Safe,
            EthersBlockNumber::Earliest => BlockNumberOrTag::Earliest,
            EthersBlockNumber::Pending => BlockNumberOrTag::Pending,
            EthersBlockNumber::Number(n) => BlockNumberOrTag::Number(n.as_u64()),
        }
    }
}

/// BlockNumber (reth) -> (ethers)
impl ToEthers<EthersBlockNumber> for BlockNumberOrTag {
    fn into_ethers(self) -> EthersBlockNumber {
        match self {
            BlockNumberOrTag::Latest => EthersBlockNumber::Latest,
            BlockNumberOrTag::Earliest => EthersBlockNumber::Earliest,
            BlockNumberOrTag::Pending => EthersBlockNumber::Pending,
            BlockNumberOrTag::Finalized => EthersBlockNumber::Finalized,
            BlockNumberOrTag::Safe => EthersBlockNumber::Safe,
            BlockNumberOrTag::Number(n) => EthersBlockNumber::Number(n.into()),
        }
    }
}

// -----------------------------------------------

/// BlockNumber (ethers) -> BlockID (reth)
impl ToReth<BlockId> for EthersBlockNumber {
    fn into_reth(self) -> BlockId {
        BlockId::Number(self.into_reth())
    }
}

/// BlockNumber (reth) -> (ethers)
impl ToEthers<EthersBlockNumber> for BlockId {
    fn into_ethers(self) -> EthersBlockNumber {
        match self {
            BlockId::Number(x) => x.into_ethers(),
            _ => EthersBlockNumber::Latest, // default
        }
    }
}
