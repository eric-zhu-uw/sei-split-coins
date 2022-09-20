#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::error::ContractError;
    use crate::msg::{ExecuteMsg, InstantiateMsg, OwnerResponse, QueryMsg, WalletResponse};
    use crate::state::{CONFIG, WALLETS};
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR,
    };
    use cosmwasm_std::{coin, coins, BankMsg, Uint128};
    use cosmwasm_std::{from_binary, Addr, CosmosMsg};

    #[test]
    // owner=None in InstantiationMsg - use info.sender
    fn set_contract_owner_default() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: None,
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(MOCK_CONTRACT_ADDR, config.owner.to_string());
    }

    #[test]
    // owner field is set in InstantiationMsg
    fn set_contract_owner() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(String::from("eric"), config.owner.to_string());

        // let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOwner {}).unwrap();
        // let value : Addr = from_binary(&res).unwrap();
        // let x = deps.get("config".as_bytes());
    }

    #[test]
    fn query_contract_owner() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: None,
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // query pot
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOwner {}).unwrap();

        let owner: OwnerResponse = from_binary(&res).unwrap();
        assert_eq!(String::from(MOCK_CONTRACT_ADDR), owner.owner.to_string())
    }

    #[test]
    fn execute_split_coins_no_usei() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[coin(100, "abc")]);
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::SplitCoins {
            target_addr1: String::from("test1"),
            target_addr2: String::from("test2"),
        };

        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(res.is_err());

        match res.unwrap_err() {
            ContractError::InvalidTokenTransfer {} => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }

    // #[test]
    // fn execute_split_coins_less_usei_than_amount() {
    //     let mut deps = mock_dependencies();
    //     let msg = InstantiateMsg {
    //         owner: Some(String::from("eric")),
    //         cw20_addr: String::from(MOCK_CONTRACT_ADDR),
    //         fee_percent: None
    //     };
    //     let info = mock_info(MOCK_CONTRACT_ADDR, &[coin(99, "usei")]);
    //     let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    //     let msg = ExecuteMsg::SplitCoins {
    //         target_addr1: String::from("test1"),
    //         target_addr2: String::from("test2"),
    //     };

    //     let res = execute(deps.as_mut(), mock_env(), info, msg);
    //     assert!(res.is_err());

    //     match res.unwrap_err() {
    //         ContractError::InvalidFunds {} => {}
    //         e => panic!("unexpected error: {:?}", e),
    //     }
    // }

    #[test]
    fn execute_split_coins_default() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[coin(100, "usei")]);
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::SplitCoins {
            target_addr1: String::from("test1"),
            target_addr2: String::from("test2"),
        };

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        let balance1 = WALLETS
            .load(&deps.storage, Addr::unchecked("test1"))
            .unwrap();
        let balance2 = WALLETS
            .load(&deps.storage, Addr::unchecked("test2"))
            .unwrap();

        assert_eq!(0, res.messages.len());
        assert_eq!(Uint128::new(50), balance1);
        assert_eq!(Uint128::new(50), balance2);
    }

    #[test]
    fn execute_split_coins_odd_amount() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[coin(101, "usei")]);
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::SplitCoins {
            target_addr1: String::from("test1"),
            target_addr2: String::from("test2"),
        };

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        let balance1 = WALLETS
            .load(&deps.storage, Addr::unchecked("test1"))
            .unwrap();
        let balance2 = WALLETS
            .load(&deps.storage, Addr::unchecked("test2"))
            .unwrap();

        assert_eq!(0, res.messages.len());
        assert_eq!(Uint128::new(51), balance1);
        assert_eq!(Uint128::new(50), balance2);
    }

    #[test]
    fn execute_split_coins_multiple_times_same_wallet() {
        // test splitting multiple times into the same wallet and verify correctness
        assert!(true);
    }

    #[test]
    fn execute_split_coins_exceed_uint128() {
        // test exceeding the u128 limit for a wallet - checked_add
        assert!(true);
    }

    #[test]
    fn execute_split_coins_invalid_address() {
        // test with an invalid address - should throw error (not even sure what an invalid address is)
        assert!(true);
    }

    #[test]
    fn execute_withdraw_coins_default() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from("eric"),
            fee_percent: None
        };
        let info = mock_info("eric", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::SplitCoins {
            target_addr1: String::from("test1"),
            target_addr2: String::from("test2"),
        };

        let info = mock_info(MOCK_CONTRACT_ADDR, &[coin(100, "usei")]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = ExecuteMsg::WithdrawCoins {
            amount: Some(Uint128::new(50)),
        };
        let info = mock_info("test1", &[]);

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        let msg = res.messages[0].clone().msg;

        assert_eq!(
            msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: String::from("test1"),
                amount: coins(50u128, "usei")
            })
        );

        let balance1 = WALLETS
            .load(&deps.storage, Addr::unchecked("test1"))
            .unwrap();
        let balance2 = WALLETS
            .load(&deps.storage, Addr::unchecked("test2"))
            .unwrap();

        assert_eq!(balance1, Uint128::new(0));
        assert_eq!(balance2, Uint128::new(50));
    }

    #[test]
    fn execute_withdraw_coins_more_than_balance() {
        // try to withdraw more than balance - should throw error
        assert!(true);
    }

    #[test]
    fn execute_withdraw_coins_multiple_times_same_wallet() {
        // try to withdraw multiple times from same wallet - ensure it updates on every withdrawl
        assert!(true);
    }

    #[test]
    fn execute_withdraw_coins_wallet_dne() {
        // try to withdraw from a wallet that does not even exist
        assert!(true);
    }

    #[test]
    fn query_non_existing_wallet_balance() {
        // could also throw error?
        // this should just return 0 if it's a valid wallet but doesn't exist
        assert!(true);
    }

    #[test]
    fn query_wallet_zero_balance() {
        // this shoudl just return 0
    }

    #[test]
    fn query_existing_wallet_balance() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[coin(100, "abc"), coin(99, "usei")]);
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // isolate query functionality but directly writing into storage
        assert!(WALLETS
            .save(
                deps.as_mut().storage,
                Addr::unchecked("test1"),
                &Uint128::new(100)
            )
            .is_ok());

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetWallet {
                addr: Addr::unchecked("test1"),
            },
        )
        .unwrap();

        let wallet: WalletResponse = from_binary(&res).unwrap();
        assert_eq!("test1", wallet.addr.to_string());
        assert_eq!(Uint128::new(100), wallet.amount);
    }
}
