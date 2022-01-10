#[cfg(test)]
mod tests {

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    use crate::contract::{execute, instantiate, query};
    use crate::error::ContractError;
    use crate::msg::{Entry, ExecuteMsg, GetAllResponse, InstantiateMsg, QueryMsg};

    #[test]
    fn check_initialize() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Query should start off with no values
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetAll {
                owner: "someone".to_string(),
            },
        )
        .unwrap();
        let value: GetAllResponse = from_binary(&res).unwrap();
        let empty_vec: Vec<Entry> = Vec::new();
        assert_eq!(empty_vec, value.entries);
    }

    #[test]
    fn add() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add a new password
        let info = mock_info("someone", &coins(1, "token"));
        let msg = ExecuteMsg::Add {
            name: "google".to_string(),
            password: "abcd1234!".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Check that new password can be queried
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetAll {
                owner: "someone".to_string(),
            },
        )
        .unwrap();
        let value: GetAllResponse = from_binary(&res).unwrap();
        assert_eq!(
            vec![Entry {
                name: "google".to_string(),
                password: "abcd1234!".to_string(),
            }],
            value.entries
        );

        // Check that we can't add to existing password
        let info = mock_info("someone", &coins(1, "token"));
        let msg = ExecuteMsg::Add {
            name: "google".to_string(),
            password: "abc".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::KeyAlreadyExists {}) => {}
            _ => panic!("Must return unauthorized error"),
        }
    }

    #[test]
    fn edit() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Attempt to edit a non-existent password
        let info = mock_info("someone", &coins(1, "token"));
        let msg = ExecuteMsg::Edit {
            name: "google".to_string(),
            password: "abcd1234!".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::KeyDoesntExist {}) => {}
            _ => panic!("Must return key doesnt exist error"),
        }

        // Add a new password
        let info = mock_info("someone", &coins(1, "token"));
        let msg = ExecuteMsg::Add {
            name: "google".to_string(),
            password: "abcd1234!".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Edit the password
        let info = mock_info("someone", &coins(1, "token"));
        let msg = ExecuteMsg::Edit {
            name: "google".to_string(),
            password: "abc".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Check that edited password can be queried
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetAll {
                owner: "someone".to_string(),
            },
        )
        .unwrap();
        let value: GetAllResponse = from_binary(&res).unwrap();
        assert_eq!(
            vec![Entry {
                name: "google".to_string(),
                password: "abc".to_string(),
            }],
            value.entries
        );
    }

    #[test]
    fn delete() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Attempt to delete a non-existent password
        let info = mock_info("someone", &coins(1, "token"));
        let msg = ExecuteMsg::Delete {
            name: "google".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::KeyDoesntExist {}) => {}
            _ => panic!("Must return key doesnt exist error"),
        }

        // Add a new password
        let info = mock_info("someone", &coins(1, "token"));
        let msg = ExecuteMsg::Add {
            name: "google".to_string(),
            password: "abcd1234!".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Delete the password
        let info = mock_info("someone", &coins(1, "token"));
        let msg = ExecuteMsg::Delete {
            name: "google".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Check that deleted password is no longer present
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetAll {
                owner: "someone".to_string(),
            },
        )
        .unwrap();
        let value: GetAllResponse = from_binary(&res).unwrap();
        let empty_vec: Vec<Entry> = Vec::new();
        assert_eq!(empty_vec, value.entries);
    }
}
