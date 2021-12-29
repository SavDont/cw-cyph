#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{GetAllResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{DATA};

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

pub fn try_add(deps: DepsMut, info: MessageInfo, name: String, password: String) -> Result<Response, ContractError> {
    let exists = DATA.has(deps.storage, (&info.sender, &name));
    if exists {
        return Err(ContractError::KeyAlreadyExists {});
    }
    DATA.save(deps.storage, (&info.sender, &name), &password)?;
    Ok(Response::new().add_attribute("method", "try_add"))
}

pub fn try_edit(deps: DepsMut, info: MessageInfo, name: String, password: String) -> Result<Response, ContractError> {
    DATA.update(deps.storage, (&info.sender, &name), |v| -> Result<String, ContractError> {
        match v {
            None => Err(ContractError::KeyDoesntExist {}),
            Some (_x) => Ok(password)
        }
    })?;
    Ok(Response::new().add_attribute("method", "try_edit"))
}

pub fn try_delete(deps: DepsMut, info: MessageInfo, name: String) -> Result<Response, ContractError> {
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
    Ok(GetAllResponse { passwords: all?})
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn check_initialize() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Query should start off with no values
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAll { owner: String::from("someone") }).unwrap();
        let value: GetAllResponse = from_binary(&res).unwrap();
        let empty_vec: Vec<(Vec<u8>, String)> = Vec::new();
        assert_eq!(empty_vec, value.passwords);
    }

    #[test]
    fn add() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add a new password
        let info = mock_info("someone", &coins( 1, "token"));
        let msg = ExecuteMsg::Add { name: String::from("google"), password: String::from("abcd1234!")};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Check that new password can be queried
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAll { owner: String::from("someone") }).unwrap();
        let value: GetAllResponse = from_binary(&res).unwrap();
        assert_eq!(vec![("google".as_bytes().to_vec(), String::from("abcd1234!"))], value.passwords);

        // Check that we can't add to existing password
        let info = mock_info("someone", &coins(1, "token"));
        let msg = ExecuteMsg::Add { name: String::from("google"), password: String::from("abc")};
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::KeyAlreadyExists {}) => {}
            _ => panic!("Must return unauthorized error"),
        }
    }

    #[test]
    fn edit() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Attempt to edit a non-existent password
        let info = mock_info("someone", &coins( 1, "token"));
        let msg = ExecuteMsg::Edit { name: String::from("google"), password: String::from("abcd1234!")};
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::KeyDoesntExist {}) => {},
            _ => panic!("Must return key doesnt exist error")
        }

        // Add a new password
        let info = mock_info("someone", &coins( 1, "token"));
        let msg = ExecuteMsg::Add { name: String::from("google"), password: String::from("abcd1234!")};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Edit the password
        let info = mock_info("someone", &coins( 1, "token"));
        let msg = ExecuteMsg::Edit { name: String::from("google"), password: String::from("abc")};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Check that edited password can be queried
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAll { owner: String::from("someone") }).unwrap();
        let value: GetAllResponse = from_binary(&res).unwrap();
        assert_eq!(vec![("google".as_bytes().to_vec(), String::from("abc"))], value.passwords);
    }

    #[test]
    fn delete() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Attempt to delete a non-existent password
        let info = mock_info("someone", &coins( 1, "token"));
        let msg = ExecuteMsg::Delete { name: String::from("google") };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::KeyDoesntExist {}) => {},
            _ => panic!("Must return key doesnt exist error")
        }

        // Add a new password
        let info = mock_info("someone", &coins( 1, "token"));
        let msg = ExecuteMsg::Add { name: String::from("google"), password: String::from("abcd1234!")};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Delete the password
        let info = mock_info("someone", &coins( 1, "token"));
        let msg = ExecuteMsg::Delete { name: String::from("google")};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Check that deleted password is no longer present
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAll { owner: String::from("someone") }).unwrap();
        let value: GetAllResponse = from_binary(&res).unwrap();
        let empty_vec: Vec<(Vec<u8>, String)> = Vec::new();
        assert_eq!(empty_vec, value.passwords);
    }
}
