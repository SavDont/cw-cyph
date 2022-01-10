use std::str;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{Entry, ExecuteMsg, GetAllResponse, InstantiateMsg, QueryMsg};
use crate::state::DATA;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-cyph";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Add { name, password } => try_add(deps, info, name, password),
        ExecuteMsg::Edit { name, password } => try_edit(deps, info, name, password),
        ExecuteMsg::Delete { name } => try_delete(deps, info, name),
    }
}

pub fn try_add(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    password: String,
) -> Result<Response, ContractError> {
    let exists = DATA.has(deps.storage, (&info.sender, &name));
    if exists {
        return Err(ContractError::KeyAlreadyExists {});
    }
    DATA.save(deps.storage, (&info.sender, &name), &password)?;
    Ok(Response::new().add_attribute("method", "try_add"))
}

pub fn try_edit(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    password: String,
) -> Result<Response, ContractError> {
    DATA.update(
        deps.storage,
        (&info.sender, &name),
        |v| -> Result<String, ContractError> {
            match v {
                None => Err(ContractError::KeyDoesntExist {}),
                Some(_x) => Ok(password),
            }
        },
    )?;
    Ok(Response::new().add_attribute("method", "try_edit"))
}

pub fn try_delete(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
) -> Result<Response, ContractError> {
    let exists = DATA.has(deps.storage, (&info.sender, &name));
    if !exists {
        return Err(ContractError::KeyDoesntExist {});
    }
    DATA.remove(deps.storage, (&info.sender, &name));
    Ok(Response::new().add_attribute("method", "try_delete"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAll { owner } => to_binary(&query_all(deps, owner)?),
    }
}

fn query_all(deps: Deps, owner: String) -> StdResult<GetAllResponse> {
    let owner_checked = deps.api.addr_validate(owner.as_str())?;
    let all: StdResult<Vec<_>> = DATA
        .prefix(&owner_checked)
        .range(deps.storage, None, None, Order::Ascending)
        .collect();
    let mut resp: Vec<Entry> = Vec::new();
    for (name, password) in all? {
        let name_string = str::from_utf8(&name[..])?.to_string();
        resp.push(Entry {
            name: name_string,
            password: password,
        });
    }
    Ok(GetAllResponse { entries: resp })
}
