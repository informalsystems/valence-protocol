// Since Elys is using an old CosmWasm version, to make it compatible with our packages, we are going to redefine the messages here using Cosmwasm 2.x that we need
// for our library
// The content here is from elys-std 0.1.0
use cosmwasm_std::{Coin, CustomQuery, DepsMut, StdError, StdResult};

#[derive(
    ::serde::Serialize, ::serde::Deserialize, Clone, Debug, PartialEq, Eq, ::schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ElysQuery {
    QueryCommittedTokensLocked { address: String },
    QueryGetPool { pool_id: u64 },
}

impl CustomQuery for ElysQuery {}

#[derive(
    ::serde::Serialize, ::serde::Deserialize, Clone, Debug, PartialEq, Eq, ::schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub struct QueryGetPoolResponse {
    pub pool: ::core::option::Option<PoolResponse>,
}
#[derive(
    ::serde::Serialize, ::serde::Deserialize, Clone, Debug, PartialEq, Eq, ::schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub struct PoolResponse {
    pub deposit_denom: String,
    pub redemption_rate: String,
    pub interest_rate: String,
    pub interest_rate_max: String,
    pub interest_rate_min: String,
    pub interest_rate_increase: String,
    pub interest_rate_decrease: String,
    pub health_gain_factor: String,
    pub total_value: String,
    pub max_leverage_ratio: String,
    pub pool_id: u64,
    pub total_deposit: String,
    pub total_borrow: String,
    pub borrow_ratio: String,
    pub max_withdraw_ratio: String,
}
#[derive(
    ::serde::Serialize, ::serde::Deserialize, Clone, Debug, PartialEq, Eq, ::schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub struct QueryCommittedTokensLockedResponse {
    pub address: String,
    pub locked_committed: Vec<Coin>,
    pub total_committed: Vec<Coin>,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
)]
pub struct MsgBond {
    #[prost(string, tag = "1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub amount: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    #[serde(alias = "poolID")]
    pub pool_id: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
)]
pub struct MsgUnbond {
    #[prost(string, tag = "1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub amount: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    #[serde(alias = "poolID")]
    pub pool_id: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
)]
pub struct MsgClaimRewards {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, repeated, tag = "2")]
    #[serde(alias = "poolIDs")]
    pub pool_ids: ::prost::alloc::vec::Vec<u64>,
}

pub fn query_pool(deps: &DepsMut<ElysQuery>, pool_id: u64) -> StdResult<PoolResponse> {
    let query = ElysQuery::QueryGetPool { pool_id };
    let query_pool_response: Option<QueryGetPoolResponse> = deps.querier.query(&query.into()).ok();

    // Check if pool exists, otherwise return an error
    let pool_response =
        query_pool_response.ok_or_else(|| StdError::generic_err("failed to query pool"))?;
    let pool = pool_response
        .pool
        .ok_or_else(|| StdError::generic_err("pool not found"))?;

    Ok(pool)
}
