#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::error::ContractError;
    use crate::msg::{ExecuteMsg, InstantiateMsg, OwnerResponse, QueryMsg, WalletResponse};
    use crate::state::{CONFIG, WALLETS};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{coin, coins, BankMsg, Uint128};
    use cosmwasm_std::{from_binary, Addr, CosmosMsg};

    #[test]
    fn set_contract_owner_default_none() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: None,
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None,
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(MOCK_CONTRACT_ADDR, config.owner.to_string());
    }

    #[test]
    fn set_contract_owner() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None,
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(String::from("eric"), config.owner.to_string());
    }

    #[test]
    fn query_contract_owner() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: None,
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None,
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOwner {}).unwrap();

        let owner: OwnerResponse = from_binary(&res).unwrap();
        assert_eq!(String::from(MOCK_CONTRACT_ADDR), owner.owner.to_string())
    }

    #[test]
    fn execute_split_coins_invalid_token_funds() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None,
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::SplitCoins {
            target_addr1: String::from("test1"),
            target_addr2: String::from("test2"),
        };
        let info_missing_usei = mock_info(MOCK_CONTRACT_ADDR, &[coin(100, "abc")]);
        let info_multiple_coins =
            mock_info(MOCK_CONTRACT_ADDR, &[coin(100, "usei"), coin(10, "abc")]);

        let res = execute(deps.as_mut(), mock_env(), info_missing_usei, msg.clone());
        assert!(res.is_err());
        match res.unwrap_err() {
            ContractError::InvalidTokenTransfer {} => {}
            e => panic!("unexpected error: {:?}", e),
        }

        let res = execute(deps.as_mut(), mock_env(), info_multiple_coins, msg.clone());
        assert!(res.is_err());
        match res.unwrap_err() {
            ContractError::InvalidTokenTransfer {} => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[test]
    fn execute_split_coins_even_amount() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None,
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("sender", &[coin(100, "usei")]);
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
            fee_percent: None,
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("sender", &[coin(101, "usei")]);
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
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None,
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("sender", &[coin(101, "usei")]);
        let msg = ExecuteMsg::SplitCoins {
            target_addr1: String::from("test1"),
            target_addr2: String::from("test2"),
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("sender", &[coin(50, "usei")]);
        let msg = ExecuteMsg::SplitCoins {
            target_addr1: String::from("test1"),
            target_addr2: String::from("test3"),
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("sender", &[coin(5, "usei")]);
        let msg = ExecuteMsg::SplitCoins {
            target_addr1: String::from("test2"),
            target_addr2: String::from("test1"),
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let balance1 = WALLETS
            .load(&deps.storage, Addr::unchecked("test1"))
            .unwrap();
        let balance2 = WALLETS
            .load(&deps.storage, Addr::unchecked("test2"))
            .unwrap();
        let balance3 = WALLETS
            .load(&deps.storage, Addr::unchecked("test3"))
            .unwrap();

        assert_eq!(Uint128::new(51 + 25 + 2), balance1);
        assert_eq!(Uint128::new(50 + 3), balance2);
        assert_eq!(Uint128::new(25), balance3);
    }

    #[test]
    fn execute_split_coins_wallet_exceed_uint128() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None,
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert!(WALLETS
            .save(
                deps.as_mut().storage,
                Addr::unchecked("test1"),
                &Uint128::MAX
            )
            .is_ok());
        let info = mock_info("sender", &[coin(u128::MAX, "usei")]);
        let msg = ExecuteMsg::SplitCoins {
            target_addr1: String::from("test1"),
            target_addr2: String::from("test2"),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);

        assert!(res.is_err());
        match res.unwrap_err() {
            ContractError::Std(_) => {}
            e => panic!("unexpected error: {:?}", e),
        }

        let balance1 = WALLETS
            .load(&deps.storage, Addr::unchecked("test1"))
            .unwrap();
        assert_eq!(Uint128::MAX, balance1);
    }

    #[test]
    fn execute_split_coins_invalid_address() {
        // test with an invalid address - should throw error (not even sure what an invalid address is)
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None,
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        let info = mock_info("sender", &[coin(u128::MAX, "usei")]);
        let msg = ExecuteMsg::SplitCoins {
            target_addr1: String::from(" "),
            target_addr2: String::from("test2"),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);

        assert!(res.is_err());
        match res.unwrap_err() {
            ContractError::Std(_) => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[test]
    fn execute_split_coins_with_int_fee() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: Some(Uint128::new(2)),
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info(MOCK_CONTRACT_ADDR, &[coin(100, "usei")]);
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
        let balance_owner = WALLETS
            .load(&deps.storage, Addr::unchecked(MOCK_CONTRACT_ADDR))
            .unwrap();

        assert_eq!(0, res.messages.len());
        assert_eq!(Uint128::new(49), balance1);
        assert_eq!(Uint128::new(49), balance2);
        assert_eq!(Uint128::new(2), balance_owner);
    }

    #[test]
    fn execute_split_coins_with_fractional_fee() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: Some(Uint128::new(11)),
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info(MOCK_CONTRACT_ADDR, &[coin(10, "usei")]);
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
        let balance_owner = WALLETS
            .load(&deps.storage, Addr::unchecked(MOCK_CONTRACT_ADDR))
            .unwrap();

        assert_eq!(0, res.messages.len());
        assert_eq!(Uint128::new(5), balance1);
        assert_eq!(Uint128::new(4), balance2);
        assert_eq!(Uint128::new(1), balance_owner);
    }

    #[test]
    fn execute_withdraw_coins_default() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from("eric"),
            fee_percent: None,
        };
        let info = mock_info("eric", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

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
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None,
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert!(WALLETS
            .save(
                deps.as_mut().storage,
                Addr::unchecked("test1"),
                &Uint128::new(49)
            )
            .is_ok());

        let msg = ExecuteMsg::WithdrawCoins {
            amount: Some(Uint128::new(50)),
        };
        let info = mock_info("test1", &[]);

        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(res.is_err());

        match res.unwrap_err() {
            ContractError::InsufficientFunds {} => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[test]
    fn execute_withdraw_coins_wallet_dne() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None,
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::WithdrawCoins {
            amount: Some(Uint128::new(50)),
        };
        let info = mock_info("test1", &[]);

        let res = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(res.is_err());

        match res.unwrap_err() {
            ContractError::InsufficientFunds {} => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[test]
    fn execute_withdraw_coins_multiple_times_same_wallet() {
        // try to withdraw multiple times from same wallet - ensure it updates on every withdrawl
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from("eric"),
            fee_percent: None,
        };
        let info = mock_info("eric", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert!(WALLETS
            .save(
                deps.as_mut().storage,
                Addr::unchecked("test1"),
                &Uint128::new(50)
            )
            .is_ok());

        let msg = ExecuteMsg::WithdrawCoins {
            amount: Some(Uint128::new(20)),
        };
        let info = mock_info("test1", &[]);

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        let msg = res.messages[0].clone().msg;
        assert_eq!(
            msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: String::from("test1"),
                amount: coins(20u128, "usei")
            })
        );

        let balance1 = WALLETS
            .load(&deps.storage, Addr::unchecked("test1"))
            .unwrap();
        assert_eq!(balance1, Uint128::new(30));

        let msg = ExecuteMsg::WithdrawCoins {
            amount: Some(Uint128::new(25)),
        };
        let info = mock_info("test1", &[]);

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        let msg = res.messages[0].clone().msg;
        assert_eq!(
            msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: String::from("test1"),
                amount: coins(25u128, "usei")
            })
        );

        let balance1 = WALLETS
            .load(&deps.storage, Addr::unchecked("test1"))
            .unwrap();
        assert_eq!(balance1, Uint128::new(5));
    }

    #[test]
    fn query_non_existing_wallet_balance() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None,
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[coin(100, "abc"), coin(99, "usei")]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

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
        assert_eq!(Uint128::new(0), wallet.amount);
    }

    #[test]
    fn query_wallet_zero_balance() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None,
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[coin(100, "abc"), coin(99, "usei")]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        assert!(WALLETS
            .save(
                deps.as_mut().storage,
                Addr::unchecked("test1"),
                &Uint128::new(0)
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
        assert_eq!(Uint128::new(0), wallet.amount);
    }

    #[test]
    fn query_existing_wallet_balance() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            owner: Some(String::from("eric")),
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
            fee_percent: None,
        };
        let info = mock_info(MOCK_CONTRACT_ADDR, &[coin(100, "abc"), coin(99, "usei")]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

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
