use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, BlockInfo, Uint128};
use cw_utils::Expiration;

use crate::{
    function::{AtomicFunction, Function, NonAtomicFunction, RetryLogic},
    msg::ProcessorMessage,
};

#[cw_serde]
// What an owner or subowner can pass to the contract to create an authorization
pub struct AuthorizationInfo {
    // Unique ID for the authorization, will be used as denom of the TokenFactory token if needed
    pub label: String,
    pub mode: AuthorizationModeInfo,
    pub not_before: Expiration,
    pub duration: AuthorizationDuration,
    // Default will be 1, defines how many times a specific authorization can be executed concurrently
    pub max_concurrent_executions: Option<u64>,
    pub subroutine: Subroutine,
    // If not passed, we will set the priority to Medium
    pub priority: Option<Priority>,
}

#[cw_serde]
pub enum AuthorizationDuration {
    Forever,
    Seconds(u64),
    Blocks(u64),
}

#[cw_serde]
// What we will save in the state of the Authorization contract for each label
pub struct Authorization {
    pub label: String,
    pub mode: AuthorizationMode,
    pub not_before: Expiration,
    pub expiration: Expiration,
    pub max_concurrent_executions: u64,
    pub subroutine: Subroutine,
    pub priority: Priority,
    pub state: AuthorizationState,
}

impl AuthorizationInfo {
    pub fn into_authorization(self, block_info: &BlockInfo, api: &dyn Api) -> Authorization {
        let expiration = match self.duration {
            AuthorizationDuration::Forever => Expiration::Never {},
            AuthorizationDuration::Seconds(seconds) => {
                Expiration::AtTime(block_info.time.plus_seconds(seconds))
            }
            AuthorizationDuration::Blocks(blocks) => {
                Expiration::AtHeight(block_info.height + blocks)
            }
        };
        Authorization {
            label: self.label,
            mode: self.mode.into_mode_validated(api),
            not_before: self.not_before,
            expiration,
            max_concurrent_executions: self.max_concurrent_executions.unwrap_or(1),
            subroutine: self.subroutine,
            priority: self.priority.unwrap_or_default(),
            state: AuthorizationState::Enabled,
        }
    }
}

#[cw_serde]

pub enum AuthorizationModeInfo {
    Permissioned(PermissionTypeInfo),
    Permissionless,
}

#[cw_serde]
pub enum PermissionTypeInfo {
    // With call limit, we will mint certain amount of tokens per address. Each time they execute successfully we'll burn the token they send
    WithCallLimit(Vec<(String, Uint128)>),
    // Without call limit we will mint 1 token per address and we will query the sender if he has the token to verify if he can execute the functions
    WithoutCallLimit(Vec<String>),
}

impl AuthorizationModeInfo {
    pub fn into_mode_validated(&self, api: &dyn Api) -> AuthorizationMode {
        match self {
            Self::Permissioned(permission_type) => {
                AuthorizationMode::Permissioned(permission_type.into_type_validated(api))
            }
            Self::Permissionless => AuthorizationMode::Permissionless,
        }
    }
}

impl PermissionTypeInfo {
    pub fn into_type_validated(&self, api: &dyn Api) -> PermissionType {
        match self {
            Self::WithCallLimit(permissions) => PermissionType::WithCallLimit(
                permissions
                    .iter()
                    .map(|(addr, amount)| (api.addr_validate(addr).unwrap(), *amount))
                    .collect(),
            ),
            Self::WithoutCallLimit(permissions) => PermissionType::WithoutCallLimit(
                permissions
                    .iter()
                    .map(|addr| api.addr_validate(addr).unwrap())
                    .collect(),
            ),
        }
    }
}

#[cw_serde]
pub enum AuthorizationMode {
    Permissioned(PermissionType),
    Permissionless,
}

#[cw_serde]
pub enum PermissionType {
    // With call limit, we will mint certain amount of tokens per address. Each time they execute successfully we'll burn the token they send
    WithCallLimit(Vec<(Addr, Uint128)>),
    // Without call limit we will mint 1 token per address and we will query the sender if he has the token to verify if he can execute the functions
    WithoutCallLimit(Vec<Addr>),
}

#[cw_serde]
pub enum Subroutine {
    Atomic(AtomicSubroutine),
    NonAtomic(NonAtomicSubroutine),
}

impl Subroutine {
    pub fn get_contract_address_by_function_index(&self, index: usize) -> String {
        self.get_function_by_index(index)
            .map(|function| function.get_contract_address())
            .unwrap_or_default()
    }

    pub fn get_function_by_index(&self, index: usize) -> Option<&dyn Function> {
        match self {
            Subroutine::Atomic(config) => config.functions.get(index).map(|a| a as &dyn Function),
            Subroutine::NonAtomic(config) => {
                config.functions.get(index).map(|a| a as &dyn Function)
            }
        }
    }

    pub fn get_expiration_time(&self) -> Option<u64> {
        match self {
            Subroutine::Atomic(config) => config.expiration_time,
            Subroutine::NonAtomic(config) => config.expiration_time,
        }
    }
}

#[cw_serde]
pub struct AtomicSubroutine {
    pub functions: Vec<AtomicFunction>,
    // Used for Atomic batches, if we don't specify retry logic then the functions won't be retried.
    pub retry_logic: Option<RetryLogic>,
    // How long that a batch for this subroutine will be valid once it's sent from the authorization contract, in seconds.
    pub expiration_time: Option<u64>,
}

#[cw_serde]
pub struct NonAtomicSubroutine {
    pub functions: Vec<NonAtomicFunction>,
    // How long that a batch for this subroutine will be valid once it's sent from the authorization contract, in seconds.
    pub expiration_time: Option<u64>,
}

#[cw_serde]
#[derive(Default)]
pub enum Priority {
    #[default]
    Medium,
    High,
}

#[cw_serde]
pub enum AuthorizationState {
    Enabled,
    Disabled,
}

#[cw_serde]
pub enum AuthorizationMsg {
    EnqueueMsgs {
        // Used for the callback or to remove the messages
        id: u64,
        msgs: Vec<ProcessorMessage>,
        subroutine: Subroutine,
        priority: Priority,
        expiration_time: Option<u64>,
    },
    EvictMsgs {
        queue_position: u64,
        priority: Priority,
    },
    InsertMsgs {
        queue_position: u64,
        id: u64,
        msgs: Vec<ProcessorMessage>,
        subroutine: Subroutine,
        priority: Priority,
        expiration_time: Option<u64>,
    },
    Pause {},
    Resume {},
}
