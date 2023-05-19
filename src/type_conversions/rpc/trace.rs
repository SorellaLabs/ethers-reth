use crate::type_conversions::{ToEthers, ToReth};

use ethers::types::{
    Action as EthersAction, ActionType as EthersActionType, Call as EthersCall,
    CallResult as EthersCallResult, CallType as EthersCallType, Create as EthersCreate,
    CreateResult as EthersCreateResult, Res as EthersRes, Reward as EthersReward,
    RewardType as EthersRewardType, Suicide as EthersSuicide, Trace as EthersTrace,
};
use reth_revm::primitives::bitvec::macros::internal::funty::Fundamental;
use reth_rpc_types::trace::parity::{
    Action, CallAction, CallOutput, CallType, CreateAction, CreateOutput,
    LocalizedTransactionTrace, RewardAction, RewardType, SelfdestructAction, TraceOutput,
    TraceResult, TransactionTrace,
};

/// Action (ethers) -> (reth)
impl ToReth<Action> for EthersAction {
    fn into_reth(self) -> Action {
        match self {
            EthersAction::Call(a) => Action::Call(CallAction {
                from: a.from.into_reth(),
                to: a.to.into_reth(),
                value: a.value.into_reth(),
                gas: a.gas.into_reth(),
                input: a.input.into_reth(),
                call_type: a.call_type.into_reth(),
            }),
            EthersAction::Create(a) => Action::Create(CreateAction {
                from: a.from.into_reth(),
                value: a.value.into_reth(),
                gas: a.gas.into_reth(),
                init: a.init.into_reth(),
            }),
            EthersAction::Suicide(a) => Action::Selfdestruct(SelfdestructAction {
                address: a.address.into_reth(),
                refund_address: a.refund_address.into_reth(),
                balance: a.balance.into_reth(),
            }),
            EthersAction::Reward(a) => Action::Reward(RewardAction {
                author: a.author.into_reth(),
                value: a.value.into_reth(),
                reward_type: match a.reward_type {
                    EthersRewardType::Uncle => RewardType::Uncle,
                    _ => RewardType::Block,
                },
            }),
        }
    }
}

/// Action (reth) -> (ethers)
impl ToEthers<EthersAction> for Action {
    fn into_ethers(self) -> EthersAction {
        match self {
            Action::Call(a) => EthersAction::Call(EthersCall {
                from: a.from.into_ethers(),
                to: a.to.into_ethers(),
                value: a.value.into_ethers(),
                gas: a.gas.into_ethers(),
                input: a.input.into_ethers(),
                call_type: a.call_type.into_ethers(),
            }),
            Action::Create(a) => EthersAction::Create(EthersCreate {
                from: a.from.into_ethers(),
                value: a.value.into_ethers(),
                gas: a.gas.into_ethers(),
                init: a.init.into_ethers(),
            }),
            Action::Selfdestruct(a) => EthersAction::Suicide(EthersSuicide {
                address: a.address.into_ethers(),
                refund_address: a.refund_address.into_ethers(),
                balance: a.balance.into_ethers(),
            }),
            Action::Reward(a) => EthersAction::Reward(EthersReward {
                author: a.author.into_ethers(),
                value: a.value.into_ethers(),
                reward_type: match a.reward_type {
                    RewardType::Block => EthersRewardType::Block,
                    RewardType::Uncle => EthersRewardType::Uncle,
                },
            }),
        }
    }
}

// -----------------------------------------------

/// CallType (ethers) + (reth)
impl ToReth<CallType> for EthersCallType {
    fn into_reth(self) -> CallType {
        match self {
            EthersCallType::None => CallType::None,
            EthersCallType::Call => CallType::Call,
            EthersCallType::CallCode => CallType::CallCode,
            EthersCallType::DelegateCall => CallType::DelegateCall,
            EthersCallType::StaticCall => CallType::StaticCall,
        }
    }
}

/// CallType (reth) -> (ethers)
impl ToEthers<EthersCallType> for CallType {
    fn into_ethers(self) -> EthersCallType {
        match self {
            CallType::None => EthersCallType::None,
            CallType::Call => EthersCallType::Call,
            CallType::CallCode => EthersCallType::CallCode,
            CallType::DelegateCall => EthersCallType::DelegateCall,
            CallType::StaticCall => EthersCallType::StaticCall,
        }
    }
}

// -----------------------------------------------

/// EthersTrace (ethers) -> TransactionTrace (reth)
impl ToReth<TransactionTrace> for EthersTrace {
    fn into_reth(self) -> TransactionTrace {
        TransactionTrace {
            trace_address: self.trace_address,
            subtraces: self.subtraces,
            action: self.action.into_reth(),
            result: self.result.into_reth(),
        }
    }
}

/// EthersTrace (ethers) + LocalizedTransactionTrace (reth)
impl ToReth<LocalizedTransactionTrace> for EthersTrace {
    fn into_reth(self) -> LocalizedTransactionTrace {
        LocalizedTransactionTrace {
            trace: self.clone().into_reth(),
            transaction_position: self.transaction_position.map(|x| x.as_u64()),
            transaction_hash: self.transaction_hash.into_reth(),
            block_number: Some(self.block_number),
            block_hash: Some(self.block_hash.into_reth()),
        }
    }
}

/// LocalizedTransactionTrace (reth) -> EthersTrace (ethers)
impl ToEthers<EthersTrace> for LocalizedTransactionTrace {
    fn into_ethers(self) -> EthersTrace {
        let action = self.trace.action.into_ethers();
        EthersTrace {
            action: action.clone(),
            result: self.trace.result.clone().into_ethers(),
            trace_address: self.trace.trace_address,
            subtraces: self.trace.subtraces,
            transaction_position: self.transaction_position.map(|x| x as usize),
            transaction_hash: self.transaction_hash.into_ethers(),
            block_number: self.block_number.unwrap(),
            block_hash: self.block_hash.into_ethers().unwrap(),
            action_type: match action {
                EthersAction::Call(_) => EthersActionType::Call,
                EthersAction::Create(_) => EthersActionType::Create,
                EthersAction::Suicide(_) => EthersActionType::Suicide,
                EthersAction::Reward(_) => EthersActionType::Reward,
            },
            error: match self.trace.result {
                Some(TraceResult::Error { error }) => Some(error),
                _ => None,
            },
        }
    }
}

// -----------------------------------------------

/// EthersTrace (ethers) -> LocalizedTransactionTrace (reth)
impl ToReth<TraceResult> for EthersRes {
    fn into_reth(self) -> TraceResult {
        match self {
            EthersRes::Call(EthersCallResult { gas_used, output }) => {
                TraceResult::parity_success(TraceOutput::Call(CallOutput {
                    gas_used: gas_used.into_reth(),
                    output: output.into_reth(),
                }))
            }
            EthersRes::Create(EthersCreateResult { gas_used, code, address }) => {
                TraceResult::parity_success(TraceOutput::Create(CreateOutput {
                    gas_used: gas_used.into_reth(),
                    code: code.into_reth(),
                    address: address.into_reth(),
                }))
            }
            EthersRes::None => TraceResult::Error { error: "Error".to_string() },
        }
    }
}

/// LocalizedTransactionTrace (reth) -> EthersTrace (ethers)
impl ToEthers<EthersRes> for TraceResult {
    fn into_ethers(self) -> EthersRes {
        match self {
            TraceResult::Success { result } => match result {
                TraceOutput::Call(CallOutput { gas_used, output }) => {
                    EthersRes::Call(EthersCallResult {
                        gas_used: gas_used.into_ethers(),
                        output: output.into_ethers(),
                    })
                }
                TraceOutput::Create(CreateOutput { gas_used, code, address }) => {
                    EthersRes::Create(EthersCreateResult {
                        gas_used: gas_used.into_ethers(),
                        code: code.into_ethers(),
                        address: address.into_ethers(),
                    })
                }
            },
            TraceResult::Error { error: _ } => EthersRes::None,
        }
    }
}
