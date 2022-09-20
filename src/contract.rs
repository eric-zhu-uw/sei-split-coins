#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128,
};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, OwnerResponse, QueryMsg, WalletResponse, FeeResponse};
use crate::state::{Config, CONFIG, WALLETS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sei-split-coins";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const PCT_DENOM: Uint128 = Uint128::new(100);

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let fee_percent = msg.fee_percent.unwrap_or(Uint128::new(0));
    if fee_percent > Uint128::new(100)  {
        return Err(ContractError::InvalidParams {});
    }

    let owner = msg
        .owner
        .and_then(|s| deps.api.addr_validate(s.as_str()).ok())
        .unwrap_or(info.sender);

    let config = Config {
        owner: owner.clone(),
        cw20_addr: deps.api.addr_validate(msg.cw20_addr.as_str())?,
        fee_percent: fee_percent
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("cw20_addr", msg.cw20_addr)
        .add_attribute("fee_percent", fee_percent.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SplitCoins {
            target_addr1,
            target_addr2,
        } => execute_split_coins(deps, _env, info, target_addr1, target_addr2),
        ExecuteMsg::WithdrawCoins { amount } => execute_withdraw_coins(deps, _env, info, amount),
    }
}

pub fn execute_split_coins(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    target_addr1: String,
    target_addr2: String,
) -> Result<Response, ContractError> {
    // mark as invalid if there are any tokens besides usei in the execute function
    if info.funds.len() != 1 || info.funds[0].denom != "usei" {
        return Err(ContractError::InvalidTokenTransfer {});
    }
    let config: Config = CONFIG.load(deps.storage)?;
    let amount = info.funds[0].amount;
    let fees_collected = amount
        .checked_mul(config.fee_percent)
        .or_else(|_| Err(ContractError::InvalidParams {}))?
        .checked_div(PCT_DENOM)
        .or_else(|_| Err(ContractError::InvalidParams {}))?;

    let amount = amount - fees_collected;
    let half_amount = amount / Uint128::new(2);

    let target_addr1 = deps.api.addr_validate(&target_addr1)?;
    let target_addr2 = deps.api.addr_validate(&target_addr2)?;

    WALLETS.update(
        deps.storage,
        target_addr1.clone(),
        |balance| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_add(half_amount + (amount % Uint128::new(2)))?)
        },
    )?;
    WALLETS.update(
        deps.storage,
        target_addr2.clone(),
        |balance| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_add(half_amount)?)
        },
    )?;
    WALLETS.update(
        deps.storage,
        config.cw20_addr.clone(),
        |balance| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_add(fees_collected)?)
        }
    )?;

    let res = Response::new()
        .add_attribute("action", "SplitCoins")
        .add_attribute("from", info.sender)
        .add_attribute("amount", amount)
        .add_attribute("target_addr1", target_addr1)
        .add_attribute("target_addr1_amount", half_amount + (amount % Uint128::new(2)))
        .add_attribute("target_addr2", target_addr2)
        .add_attribute("target_addr2_amount", half_amount)
        .add_attribute("fees_collected", fees_collected);

    Ok(res)
}

pub fn execute_withdraw_coins(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    if amount.is_some() && amount.unwrap() == Uint128::new(0) {
        return Err(ContractError::InvalidParams {});
    }

    let mut withdraw_amount: Uint128 = Uint128::new(0);
    WALLETS.update(
        deps.storage,
        info.sender.clone(),
        |balance| -> Result<Uint128, ContractError> {
            match balance {
                Some(_) => {
                    withdraw_amount = amount.unwrap_or_else(|| balance.unwrap());
                    balance
                    .unwrap()
                    .checked_sub(withdraw_amount)
                    .or_else(|_| Err(ContractError::InsufficientFunds {}))
                },
                None => Err(ContractError::InsufficientFunds {}),
            }
        },
    )?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: coins(withdraw_amount.u128(), "usei"),
        })
        .add_attribute("action", "WithdrawCoins")
        .add_attribute("addr", info.sender)
        .add_attribute("amount", withdraw_amount))
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOwner {} => to_binary(&query_owner(deps)?),
        QueryMsg::GetWallet { addr } => to_binary(&query_wallet(deps, addr)?),
        QueryMsg::GetFee {} => to_binary(&query_fee(deps)?)
    }
}

fn query_owner(deps: Deps) -> StdResult<OwnerResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    Ok(OwnerResponse {
        owner: config.owner,
    })
}

fn query_wallet(deps: Deps, addr: Addr) -> StdResult<WalletResponse> {
    let amount = WALLETS.load(deps.storage, addr.clone()).unwrap_or_default();
    Ok(WalletResponse {
        addr: addr,
        amount: amount,
    })
}

fn query_fee(deps: Deps) -> StdResult<FeeResponse> {
    let config: Config = CONFIG.load(deps.storage)?;

    Ok(FeeResponse {
        addr: config.cw20_addr,
        fee_percent: config.fee_percent
    })
}