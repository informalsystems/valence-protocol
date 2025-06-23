use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GetConfigResponse {
    pub pair_data: PairData,
    pub lp_denom: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PairData {
    pub token_0: TokenData,
    pub token_1: TokenData,
    pub pair_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenData {
    pub denom: String,
}

pub fn parse_get_config_response(json_str: &str) -> Result<GetConfigResponse, serde_json::Error> {
    let config: GetConfigResponse = serde_json::from_str(json_str)?;
    Ok(config)
}
