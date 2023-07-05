use super::{ToEthers, ToReth};

use ethers::types::{
    BlockId as EthersBlockId, BlockNumber as EthersBlockNumber, CallFrame, DefaultFrame,
    FourByteFrame, GethDebugTracingCallOptions as EthersDebugTracingCallOptions,
    GethDebugTracingOptions as EthersDebugTracingOptions, GethTrace as EthersGethTrace,
    GethTraceFrame as EthersGethTraceFrame, NoopFrame, PreStateFrame, PreStateMode,
    H256 as EthersH256,
};
use reth_primitives::{BlockId, BlockNumberOrTag, H256};
use reth_rpc_types::trace::geth::{
    GethDebugTracerConfig, GethDebugTracingCallOptions, GethDebugTracingOptions,
    GethDefaultTracingOptions, GethTrace,
};

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

/// GethDebugTracingOptions (ethers) -> (reth)
impl ToReth<GethDebugTracingOptions> for EthersDebugTracingOptions {
    fn into_reth(self) -> GethDebugTracingOptions {
        GethDebugTracingOptions {
            config: GethDefaultTracingOptions::default(),
            tracer: None,
            tracer_config: GethDebugTracerConfig::default(),
            timeout: None,
        }
    }
}

/// GethTrace (reth) -> (ethers)
impl ToEthers<EthersGethTrace> for GethTrace {
    fn into_ethers(self) -> EthersGethTrace {
        match self {
            GethTrace::Default(_) => {
                EthersGethTrace::Known(EthersGethTraceFrame::Default(DefaultFrame::default()))
            }
            GethTrace::CallTracer(_) => {
                EthersGethTrace::Known(EthersGethTraceFrame::CallTracer(CallFrame::default()))
            }
            GethTrace::FourByteTracer(_) => EthersGethTrace::Known(
                EthersGethTraceFrame::FourByteTracer(FourByteFrame::default()),
            ),
            GethTrace::PreStateTracer(_) => {
                EthersGethTrace::Known(EthersGethTraceFrame::PreStateTracer(
                    PreStateFrame::Default(PreStateMode::default()),
                ))
            }
            GethTrace::NoopTracer(_) => {
                EthersGethTrace::Known(EthersGethTraceFrame::NoopTracer(NoopFrame::default()))
            }
            GethTrace::JS(value) => EthersGethTrace::Unknown(value),
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
