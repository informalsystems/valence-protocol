# Astroport LPer library

The **Valence Astroport LPer library** library allows to **provide liquidity** into an **Astroport Liquidity Pool** from an **input account** and deposit the **LP tokens** into an **output account**.

## High-level flow

```mermaid
---
title: Astroport Liquidity Provider
---
graph LR
  IA((Input
      Account))
  OA((Output
		  Account))
  P[Processor]
  S[Astroport
      Liquidity
      Provider]
  AP[Astroport
     Pool]
  P -- 1/Provide Liquidity --> S
  S -- 2/Query balances --> IA
  S -- 3/Compute amounts --> S
  S -- 4/Do Provide Liquidity --> IA
  IA -- 5/Provide Liquidity
				  [Tokens] --> AP
  AP -- 5'/Transfer LP Tokens --> OA

```

## Functions

| Function    | Parameters | Description |
|-------------|------------|-------------|
| **ProvideDoubleSidedLiquidity** | `expected_pool_ratio_range: Option<DecimalRange>` | Provide double-sided liquidity to the pre-configured **Astroport Pool** from the **input account**, and deposit the **LP tokens** into the **output account**. Abort it the pool ratio is not within the `expected_pool_ratio` range (if specified). |
| **ProvideSingleSidedLiquidity** | `asset: String`<br>`limit: Option<Uint128>`<br>`expected_pool_ratio_range: Option<DecimalRange>` | Provide single-sided liquidity for the specified `asset` to the pre-configured **Astroport Pool** from the **input account**, and deposit the **LP tokens** into the **output account**. Abort it the pool ratio is not within the `expected_pool_ratio` range (if specified). |

## Configuration

The library is configured on instantiation via the `LibraryConfig` type.

```rust
pub struct LibraryConfig {
    // Account from which the funds are LPed
    pub input_addr: LibraryAccountType,
    // Account to which the LP tokens are forwarded
    pub output_addr: LibraryAccountType,
    // Pool address
    pub pool_addr: String,
    // LP configuration
    pub lp_config: LiquidityProviderConfig,
}

pub struct LiquidityProviderConfig {
    // Pool type, old Astroport pools use Cw20 lp tokens and new pools use native tokens, so we specify here what kind of token we are going to get.
    // We also provide the PairType structure of the right Astroport version that we are going to use for each scenario
    pub pool_type: PoolType,
    // Denoms of both native assets we are going to provide liquidity for
    pub asset_data: AssetData,
    /// Max spread used when swapping assets to provide single sided liquidity
    pub max_spread: Option<Decimal>,
}

#[cw_serde]
pub enum PoolType {
    NativeLpToken(valence_astroport_utils::astroport_native_lp_token::PairType),
    Cw20LpToken(valence_astroport_utils::astroport_cw20_lp_token::PairType),
}


pub struct AssetData {
    pub asset1: String,
    pub asset2: String,
}
```
