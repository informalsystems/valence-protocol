use crate::msg::{
    Config, FunctionMsgs, IbcTransferAmount, LibraryConfig, LibraryConfigUpdate, QueryMsg,
    RemoteChainInfo,
};
use cosmwasm_std::{coin, Addr, Empty, Uint128, Uint64};
use cw_multi_test::{error::AnyResult, App, AppResponse, ContractWrapper, Executor};
use cw_ownable::Ownership;
use getset::{Getters, Setters};
use valence_ibc_utils::types::EurekaFee;
use valence_library_utils::{
    denoms::CheckedDenom,
    msg::{ExecuteMsg, InstantiateMsg, LibraryConfigValidation},
    testing::{LibraryTestSuite, LibraryTestSuiteBase},
    LibraryAccountType,
};

const NTRN: &str = "untrn";
const ONE_MILLION: u128 = 1_000_000_000_000_u128;

#[derive(Getters, Setters)]
struct IbcTransferTestSuite {
    #[getset(get)]
    inner: LibraryTestSuiteBase,
    #[getset(get)]
    ibc_transfer_code_id: u64,
    #[getset(get)]
    input_addr: Addr,
    #[getset(get)]
    output_addr: String,
    #[getset(get)]
    input_balance: Option<(u128, String)>,
}

impl Default for IbcTransferTestSuite {
    fn default() -> Self {
        Self::new(None)
    }
}

#[allow(dead_code)]
impl IbcTransferTestSuite {
    pub fn new(input_balance: Option<(u128, String)>) -> Self {
        let mut inner = LibraryTestSuiteBase::new();

        let input_addr = inner.get_contract_addr(inner.account_code_id(), "input_account");
        let output_addr = inner.api().addr_make("output_account").to_string();

        // Template contract
        let ibc_transfer_code = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );

        let ibc_transfer_code_id = inner.app_mut().store_code(Box::new(ibc_transfer_code));

        Self {
            inner,
            ibc_transfer_code_id,
            input_addr,
            output_addr,
            input_balance,
        }
    }

    pub fn ibc_transfer_init(&mut self, cfg: &LibraryConfig) -> Addr {
        let init_msg = InstantiateMsg {
            owner: self.owner().to_string(),
            processor: self.processor().to_string(),
            config: cfg.clone(),
        };
        let addr = self.contract_init(
            self.ibc_transfer_code_id,
            "ibc_transfer_library",
            &init_msg,
            &[],
        );

        let input_addr = self.input_addr().clone();
        if self.app_mut().contract_data(&input_addr).is_err() {
            let account_addr = self.account_init("input_account", vec![addr.to_string()]);
            assert_eq!(account_addr, input_addr);

            if let Some((amount, denom)) = self.input_balance.as_ref().cloned() {
                self.init_balance(&input_addr, vec![coin(amount, denom.to_string())]);
            }
        }

        addr
    }

    fn ibc_transfer_config(
        &self,
        denom: String,
        amount: IbcTransferAmount,
        memo: String,
        remote_chain_info: RemoteChainInfo,
    ) -> LibraryConfig {
        LibraryConfig::new(
            valence_library_utils::LibraryAccountType::Addr(self.input_addr().to_string()),
            valence_library_utils::LibraryAccountType::Addr(self.output_addr().to_string()),
            valence_library_utils::denoms::UncheckedDenom::Native(denom),
            amount,
            memo,
            remote_chain_info,
        )
    }

    fn execute_ibc_transfer(&mut self, addr: Addr) -> AnyResult<AppResponse> {
        self.contract_execute(
            addr,
            &ExecuteMsg::<_, LibraryConfig>::ProcessFunction(FunctionMsgs::IbcTransfer {}),
        )
    }

    fn execute_eureka_ibc_transfer(&mut self, addr: Addr) -> AnyResult<AppResponse> {
        self.contract_execute(
            addr,
            &ExecuteMsg::<_, LibraryConfig>::ProcessFunction(FunctionMsgs::EurekaTransfer {
                eureka_fee: EurekaFee {
                    coin: coin(1, NTRN),
                    receiver: self.output_addr().to_string(),
                    timeout_timestamp: 15000000000,
                },
            }),
        )
    }

    fn update_config(&mut self, addr: Addr, new_config: LibraryConfig) -> AnyResult<AppResponse> {
        let owner = self.owner().clone();
        let updated_config = LibraryConfigUpdate {
            input_addr: Some(new_config.input_addr),
            output_addr: Some(new_config.output_addr),
            amount: Some(new_config.amount),
            denom: Some(new_config.denom),
            memo: Some(new_config.memo),
            remote_chain_info: Some(new_config.remote_chain_info),
            denom_to_pfm_map: Some(new_config.denom_to_pfm_map),
            eureka_config: valence_library_utils::OptionUpdate::Set(new_config.eureka_config),
        };
        self.app_mut().execute_contract(
            owner,
            addr,
            &ExecuteMsg::<FunctionMsgs, LibraryConfigUpdate>::UpdateConfig {
                new_config: updated_config,
            },
            &[],
        )
    }
}

impl LibraryTestSuite<Empty, Empty> for IbcTransferTestSuite {
    fn app(&self) -> &App {
        self.inner.app()
    }

    fn app_mut(&mut self) -> &mut App {
        self.inner.app_mut()
    }

    fn owner(&self) -> &Addr {
        self.inner.owner()
    }

    fn processor(&self) -> &Addr {
        self.inner.processor()
    }

    fn account_code_id(&self) -> u64 {
        self.inner.account_code_id()
    }

    fn cw20_code_id(&self) -> u64 {
        self.inner.cw20_code_id()
    }
}

// Note: all tests below are replicated to the Neutron IBC transfer library
// Any change in the tests below should be reflected in the Neutron IBC transfer library.

#[test]
fn instantiate_with_valid_config() {
    let mut suite = IbcTransferTestSuite::default();

    let cfg = suite.ibc_transfer_config(
        NTRN.to_string(),
        IbcTransferAmount::FullAmount,
        "".to_string(),
        RemoteChainInfo::new("channel-1".to_string(), Some(600u64.into())),
    );

    // Instantiate IBC transfer contract
    let lib = suite.ibc_transfer_init(&cfg);

    // Verify owner
    let owner_res: Ownership<Addr> = suite.query_wasm(&lib, &QueryMsg::Ownership {});
    assert_eq!(owner_res.owner, Some(suite.owner().clone()));

    // Verify processor
    let processor_addr: Addr = suite.query_wasm(&lib, &QueryMsg::GetProcessor {});
    assert_eq!(processor_addr, suite.processor().clone());

    // Verify library config
    let lib_cfg: Config = suite.query_wasm(&lib, &QueryMsg::GetLibraryConfig {});
    assert_eq!(
        lib_cfg,
        Config::new(
            suite.input_addr().clone(),
            suite.output_addr().clone(),
            CheckedDenom::Native(NTRN.into()),
            IbcTransferAmount::FullAmount,
            "".to_string(),
            cfg.remote_chain_info.clone()
        )
    );
}

#[test]
fn pre_validate_config_works() {
    let suite = IbcTransferTestSuite::default();

    let cfg = suite.ibc_transfer_config(
        NTRN.to_string(),
        IbcTransferAmount::FullAmount,
        "".to_string(),
        RemoteChainInfo::new("channel-1".to_string(), Some(600u64.into())),
    );

    // Pre-validate config
    cfg.pre_validate(suite.api()).unwrap();
}

#[test]
#[should_panic(expected = "Invalid IBC transfer config: amount cannot be zero.")]
fn instantiate_fails_for_zero_amount() {
    let mut suite = IbcTransferTestSuite::default();

    let cfg = suite.ibc_transfer_config(
        NTRN.to_string(),
        IbcTransferAmount::FixedAmount(Uint128::zero()),
        "".to_string(),
        RemoteChainInfo::new("channel-1".to_string(), Some(600u64.into())),
    );

    // Instantiate IBC transfer contract
    suite.ibc_transfer_init(&cfg);
}

#[test]
#[should_panic(
    expected = "Invalid IBC transfer config: remote_chain_info's channel_id cannot be empty."
)]
fn instantiate_fails_for_invalid_channel_id() {
    let mut suite = IbcTransferTestSuite::default();

    let cfg = suite.ibc_transfer_config(
        NTRN.to_string(),
        IbcTransferAmount::FixedAmount(Uint128::one()),
        "".to_string(),
        RemoteChainInfo::new("".to_string(), Some(600u64.into())),
    );

    // Instantiate IBC transfer contract
    suite.ibc_transfer_init(&cfg);
}

#[test]
#[should_panic(
    expected = "Invalid IBC transfer config: remote_chain_info's ibc_transfer_timeout cannot be zero."
)]
fn instantiate_fails_for_invalid_ibc_transfer_timeout() {
    let mut suite = IbcTransferTestSuite::default();

    let cfg = suite.ibc_transfer_config(
        NTRN.to_string(),
        IbcTransferAmount::FixedAmount(Uint128::one()),
        "".to_string(),
        RemoteChainInfo::new("channel-1".to_string(), Some(Uint64::zero())),
    );

    // Instantiate IBC transfer contract
    suite.ibc_transfer_init(&cfg);
}

// Config update tests

#[test]
#[should_panic(expected = "Invalid IBC transfer config: amount cannot be zero.")]
fn update_config_validates_amount() {
    let mut suite = IbcTransferTestSuite::default();

    let mut cfg = suite.ibc_transfer_config(
        NTRN.to_string(),
        IbcTransferAmount::FullAmount,
        "".to_string(),
        RemoteChainInfo::new("channel-1".to_string(), Some(600u64.into())),
    );

    // Instantiate IBC transfer contract
    let lib = suite.ibc_transfer_init(&cfg);

    // Update config and set amount to zero
    cfg.amount = IbcTransferAmount::FixedAmount(Uint128::zero());

    // Execute update config action
    suite.update_config(lib.clone(), cfg).unwrap();
}

#[test]
#[should_panic(
    expected = "Invalid IBC transfer config: remote_chain_info's channel_id cannot be empty."
)]
fn update_config_validates_channel_id() {
    let mut suite = IbcTransferTestSuite::default();

    let mut cfg = suite.ibc_transfer_config(
        NTRN.to_string(),
        IbcTransferAmount::FullAmount,
        "".to_string(),
        RemoteChainInfo::new("channel-1".to_string(), Some(600u64.into())),
    );

    // Instantiate IBC transfer contract
    let lib = suite.ibc_transfer_init(&cfg);

    // Update config and set channel_id to empty
    cfg.remote_chain_info.channel_id = "".to_string();

    // Execute update config action
    suite.update_config(lib.clone(), cfg).unwrap();
}

#[test]
#[should_panic(
    expected = "Invalid IBC transfer config: remote_chain_info's ibc_transfer_timeout cannot be zero."
)]
fn update_config_validates_ibc_timeout() {
    let mut suite = IbcTransferTestSuite::default();

    let mut cfg = suite.ibc_transfer_config(
        NTRN.to_string(),
        IbcTransferAmount::FullAmount,
        "".to_string(),
        RemoteChainInfo::new("channel-1".to_string(), Some(600u64.into())),
    );

    // Instantiate IBC transfer contract
    let lib = suite.ibc_transfer_init(&cfg);

    // Update config and set ibc timeout to zero
    cfg.remote_chain_info.ibc_transfer_timeout = Some(Uint64::zero());

    // Execute update config action
    suite.update_config(lib.clone(), cfg).unwrap();
}

#[test]
fn update_config_with_valid_config() {
    let mut suite = IbcTransferTestSuite::default();

    let mut cfg = suite.ibc_transfer_config(
        NTRN.to_string(),
        IbcTransferAmount::FullAmount,
        "".to_string(),
        RemoteChainInfo::new("channel-1".to_string(), Some(600u64.into())),
    );

    // Instantiate IBC transfer contract
    let lib = suite.ibc_transfer_init(&cfg);

    // Update config: swap input and output addresses
    cfg.input_addr = LibraryAccountType::Addr(suite.output_addr().to_string());
    cfg.output_addr = LibraryAccountType::Addr(suite.input_addr().to_string());
    cfg.amount = IbcTransferAmount::FixedAmount(ONE_MILLION.into());
    cfg.memo = "Chancellor on brink of second bailout for banks.".to_string();

    // Execute update config action
    suite.update_config(lib.clone(), cfg).unwrap();

    // Verify library config
    let lib_cfg: Config = suite.query_wasm(&lib, &QueryMsg::GetLibraryConfig {});
    assert_eq!(
        lib_cfg,
        Config::new(
            Addr::unchecked(suite.output_addr().clone()),
            suite.input_addr().clone().to_string(),
            CheckedDenom::Native(NTRN.into()),
            IbcTransferAmount::FixedAmount(ONE_MILLION.into()),
            "Chancellor on brink of second bailout for banks.".to_string(),
            RemoteChainInfo::new("channel-1".to_string(), Some(600u64.into())),
        )
    );
}

// Insufficient balance tests

#[test]
#[should_panic(
    expected = "Execution error: Insufficient balance for denom 'untrn' in config (required: 1000000000000, available: 0)."
)]
fn ibc_transfer_fails_for_insufficient_balance() {
    let mut suite = IbcTransferTestSuite::default();

    let cfg = suite.ibc_transfer_config(
        NTRN.to_string(),
        IbcTransferAmount::FixedAmount(ONE_MILLION.into()),
        "".to_string(),
        RemoteChainInfo::new("channel-1".to_string(), Some(600u64.into())),
    );

    // Instantiate  contract
    let lib = suite.ibc_transfer_init(&cfg);

    // Execute IBC transfer
    suite.execute_ibc_transfer(lib).unwrap();
}

// Eureka config test

#[test]
#[should_panic(expected = "No Eureka config provided.")]
fn eureka_ibc_transfer_fails_no_eureka_config() {
    let mut suite = IbcTransferTestSuite::new(Some((ONE_MILLION, NTRN.to_string())));

    let cfg = suite.ibc_transfer_config(
        NTRN.to_string(),
        IbcTransferAmount::FixedAmount(ONE_MILLION.into()),
        "".to_string(),
        RemoteChainInfo::new("channel-1".to_string(), Some(600u64.into())),
    );

    // Instantiate contract
    let lib = suite.ibc_transfer_init(&cfg);

    // Execute IBC transfer
    suite.execute_eureka_ibc_transfer(lib).unwrap();
}
