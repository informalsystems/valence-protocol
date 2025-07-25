use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Deps, DepsMut, Uint128};
use cw_ownable::cw_ownable_query;

use valence_library_utils::{
    error::LibraryError, msg::LibraryConfigValidation, LibraryAccountType,
};
use valence_macros::{valence_library_query, ValenceLibraryInterface};
#[cw_serde]
pub enum FunctionMsgs {
    /// Message to withdraw liquidity.
    WithdrawLiquidity {
        // default to 0 `token_min_amount` if not provided
        token_min_amount_0: Option<Uint128>,
        token_min_amount_1: Option<Uint128>,
    },
}

#[valence_library_query]
#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}

#[cw_serde]
#[derive(ValenceLibraryInterface)]
pub struct LibraryConfig {
    /// Address of the input account
    pub input_addr: LibraryAccountType,
    /// Address of the output account
    pub output_addr: LibraryAccountType,
    // Address of the vault we are going to withdraw liquidity from
    pub vault_addr: String,
}

impl LibraryConfig {
    pub fn new(
        input_addr: impl Into<LibraryAccountType>,
        output_addr: impl Into<LibraryAccountType>,
        vault_addr: String,
    ) -> Self {
        LibraryConfig {
            input_addr: input_addr.into(),
            output_addr: output_addr.into(),
            vault_addr,
        }
    }

    fn do_validate(&self, api: &dyn cosmwasm_std::Api) -> Result<(Addr, Addr, Addr), LibraryError> {
        let input_addr = self.input_addr.to_addr(api)?;
        let output_addr = self.output_addr.to_addr(api)?;
        let vault_addr = api.addr_validate(&self.vault_addr)?;

        Ok((input_addr, output_addr, vault_addr))
    }
}

#[cw_serde]
/// Validated library configuration
pub struct Config {
    pub input_addr: Addr,
    pub output_addr: Addr,
    pub vault_addr: Addr,
}

impl LibraryConfigValidation<Config> for LibraryConfig {
    #[cfg(not(target_arch = "wasm32"))]
    fn pre_validate(&self, api: &dyn cosmwasm_std::Api) -> Result<(), LibraryError> {
        self.do_validate(api)?;
        Ok(())
    }

    fn validate(&self, deps: Deps) -> Result<Config, LibraryError> {
        let (input_addr, output_addr, vault_addr) = self.do_validate(deps.api)?;

        Ok(Config {
            input_addr,
            output_addr,
            vault_addr,
        })
    }
}

impl LibraryConfigUpdate {
    pub fn update_config(self, deps: DepsMut) -> Result<(), LibraryError> {
        let mut config: Config = valence_library_base::load_config(deps.storage)?;

        if let Some(input_addr) = self.input_addr {
            config.input_addr = input_addr.to_addr(deps.api)?;
        }

        if let Some(output_addr) = self.output_addr {
            config.output_addr = output_addr.to_addr(deps.api)?;
        }

        if let Some(vault_addr) = self.vault_addr {
            config.vault_addr = deps.api.addr_validate(&vault_addr)?;
        }

        valence_library_base::save_config(deps.storage, &config)?;
        Ok(())
    }
}
