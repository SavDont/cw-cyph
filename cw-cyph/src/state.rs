use cosmwasm_std::Addr;
use cw_storage_plus::Map;

pub const DATA: Map<(&Addr, &str), String> = Map::new("data");
