use crate::type_conversions::{ToEthers, ToReth};

use ethers::types::{
    AccountDiff as EthersAccountDiff, Action as EthersAction, ActionType as EthersActionType,
    BlockTrace as EthersBlockTrace, Call as EthersCall, CallResult as EthersCallResult,
    CallType as EthersCallType, ChangedType as EthersChangedType, Create as EthersCreate,
    CreateResult as EthersCreateResult, Diff as EthersDiff, MemoryDiff as EthersMemoryDiff,
    Res as EthersRes, Reward as EthersReward, RewardType as EthersRewardType,
    StateDiff as EthersStateDiff, StorageDiff as EthersStorageDiff, Suicide as EthersSuicide,
    Trace as EthersTrace, TraceType as EthersTraceType, TransactionTrace as EthersTransactionTrace,
    VMExecutedOperation as EthersVMExecutedOperation, VMOperation as EthersVMOperation,
    VMTrace as EthersVMTrace,
};
use reth_revm::primitives::bitvec::macros::internal::funty::Fundamental;
use reth_rpc_types::trace::parity::{
    AccountDiff, Action, CallAction, CallOutput, CallType, ChangedType, CreateAction, CreateOutput,
    Delta, LocalizedTransactionTrace, MemoryDelta, RewardAction, RewardType, SelfdestructAction,
    StateDiff, StorageDelta, TraceOutput, TraceResult, TraceResults,
    TraceResultsWithTransactionHash, TraceType, TransactionTrace, VmExecutedOperation,
    VmInstruction, VmTrace,
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

// -----------------------------------------------

/// TraceType (ethers) -> (reth)
impl ToReth<TraceType> for EthersTraceType {
    fn into_reth(self) -> TraceType {
        match self {
            EthersTraceType::Trace => TraceType::Trace,
            EthersTraceType::VmTrace => TraceType::VmTrace,
            EthersTraceType::StateDiff => TraceType::StateDiff,
        }
    }
}

/// TraceType (reth) -> (ethers)
impl ToEthers<EthersTraceType> for TraceType {
    fn into_ethers(self) -> EthersTraceType {
        match self {
            TraceType::Trace => EthersTraceType::Trace,
            TraceType::VmTrace => EthersTraceType::VmTrace,
            TraceType::StateDiff => EthersTraceType::StateDiff,
        }
    }
}

// -----------------------------------------------

/// EthersBlockTrace (ethers) -> TraceResults (reth)
impl ToReth<TraceResults> for EthersBlockTrace {
    fn into_reth(self) -> TraceResults {
        TraceResults {
            output: self.output.into_reth(),
            trace: self.trace.into_reth(),
            vm_trace: self.vm_trace.into_reth(),
            state_diff: self.state_diff.into_reth(),
        }
    }
}

/// TraceResults (reth) -> EthersBlockTrace (ethers)
impl ToEthers<EthersBlockTrace> for TraceResults {
    fn into_ethers(self) -> EthersBlockTrace {
        EthersBlockTrace {
            output: self.output.into_ethers(),
            trace: self.trace.into_ethers(),
            vm_trace: self.vm_trace.into_ethers(),
            state_diff: self.state_diff.into_ethers(),
            transaction_hash: None,
        }
    }
}

// -----------------------------------------------

/// EthersBlockTrace (ethers) -> TraceResultsWithTransactionHash (reth)
impl ToReth<TraceResultsWithTransactionHash> for EthersBlockTrace {
    fn into_reth(self) -> TraceResultsWithTransactionHash {
        TraceResultsWithTransactionHash {
            full_trace: self.clone().into_reth(),
            transaction_hash: self.transaction_hash.into_reth().unwrap(),
        }
    }
}

/// TraceResultsWithTransactionHash (reth) -> EthersBlockTrace (ethers)
impl ToEthers<EthersBlockTrace> for TraceResultsWithTransactionHash {
    fn into_ethers(self) -> EthersBlockTrace {
        EthersBlockTrace {
            output: self.full_trace.output.into_ethers(),
            trace: self.full_trace.trace.into_ethers(),
            vm_trace: self.full_trace.vm_trace.into_ethers(),
            state_diff: self.full_trace.state_diff.into_ethers(),
            transaction_hash: Some(self.transaction_hash.into_ethers()),
        }
    }
}

// -----------------------------------------------

/// TransactionTrace (ethers) -> (reth)
impl ToReth<TransactionTrace> for EthersTransactionTrace {
    fn into_reth(self) -> TransactionTrace {
        TransactionTrace {
            trace_address: self.trace_address,
            subtraces: self.subtraces,
            action: self.action.into_reth(),
            result: self.result.into_reth(),
        }
    }
}

// -----------------------------------------------

/// VMTrace (ethers) -> (reth)
impl ToReth<VmTrace> for EthersVMTrace {
    fn into_reth(self) -> VmTrace {
        VmTrace { code: self.code.into_reth(), ops: self.ops.into_reth() }
    }
}

// -----------------------------------------------

/// EthersVMOperation (ethers) -> VmInstruction (reth)
impl ToReth<VmInstruction> for EthersVMOperation {
    fn into_reth(self) -> VmInstruction {
        VmInstruction {
            pc: self.pc,
            cost: self.cost,
            ex: self.ex.into_reth(),
            sub: self.sub.into_reth(),
        }
    }
}


// -----------------------------------------------

/// VmExecutedOperation (ethers) -> (reth)
impl ToReth<VmExecutedOperation> for EthersVMExecutedOperation {
    fn into_reth(self) -> VmExecutedOperation {
        VmExecutedOperation {
            used: self.used,
            push: Some(self.push[0]).into_reth(), // Check this
            mem: self.mem.into_reth(),
            store: self.store.into_reth(),
        }
    }
}


// -----------------------------------------------

/// EthersMemoryDiff (ethers) -> MemoryDelta (reth)
impl ToReth<MemoryDelta> for EthersMemoryDiff {
    fn into_reth(self) -> MemoryDelta {
        MemoryDelta { off: self.off, data: self.data.into_reth() }
    }
}

// -----------------------------------------------

/// EthersStorageDiff (ethers) -> StorageDelta '(reth)
impl ToReth<StorageDelta> for EthersStorageDiff {
    fn into_reth(self) -> StorageDelta {
        StorageDelta { key: self.key.into_reth(), val: self.val.into_reth() }
    }
}


// -----------------------------------------------

/// AccountDiff (ethers) -> (reth)
impl ToReth<AccountDiff> for EthersAccountDiff {
    fn into_reth(self) -> AccountDiff {
        AccountDiff {
            balance: self.balance.into_reth(),
            nonce: self.nonce.into_reth(),
            code: self.code.into_reth(),
            storage: self.storage.into_reth(),
        }
    }
}

// -----------------------------------------------

/// StateDiff (ethers) -> (reth)
impl ToReth<StateDiff> for EthersStateDiff {
    fn into_reth(self) -> StateDiff {
        StateDiff(self.0.into_reth())
    }
}

// -----------------------------------------------

/// Diff (ethers) -> Delta (reth)
impl<T, F> ToReth<Delta<T>> for EthersDiff<F>
where
    F: ToReth<T>,
    T: Clone,
{
    fn into_reth(self) -> Delta<T> {
        match self {
            EthersDiff::Same => Delta::Unchanged,
            EthersDiff::Born(x) => Delta::Added(x.into_reth()),
            EthersDiff::Died(x) => Delta::Removed(x.into_reth()),
            EthersDiff::Changed(x) => Delta::Changed(x.into_reth()),
        }
    }
}

// -----------------------------------------------

/// ChangedType (ethers) -> (reth)
impl<T, F> ToReth<ChangedType<T>> for EthersChangedType<F>
where
    F: ToReth<T>,
    T: Clone,
{
    fn into_reth(self) -> ChangedType<T> {
        ChangedType { from: self.from.into_reth(), to: self.to.into_reth() }
    }
}

