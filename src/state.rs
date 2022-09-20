use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub cw20_addr: Addr,
    pub fee_percent: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const WALLETS: Map<Addr, Uint128> = Map::new("wallets");
