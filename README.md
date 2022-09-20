# Split Coins

# SQUASH THE COMMITS INTO 1

### Commands to Run
- `cargo wasm` to build the smart contract
- `cargo test` to run smart contract unit tests
- `cargo fmt` to fmt rust code

### Coding Challenge
We want you to build a simple Sei smart contract that allows 1-to-2 transfer of the usei token. 

Please create a Sei smart contract (using Cosmwasm 1.0) with the following requirements:
- you should be able to instantiate the contract and set the owner
If the contract is instantiated with `owner`, then set the value. Otherwise, utilize `info.sender` and set that value as the owner. Ideally, the author could default the owner value to be None - however this would require two different struct `InstantiateMsg` and `InstantiateMsgNoOwner` and if we got `InstantiateMsgNoOwner`, then we would define `owner` as None.

- you should support a read query to get the owner of the smart contract
Added QueryMsg::GetOwner to allow users to query the owner. One important design concern is whether or not to use `OwnerResponse` singleton wrapper or just return an `Addr`. It's more semantics and since the other queries return a Response object, I feel more inclined to follow the model and return a Response object.

- you should support an execute message where an account can send coins to the contract and specify two accounts that can withdraw the coins (for simplicity, split coins evenly across the two destination accounts)

Only the owner should be able to start the contract
Validate if target address are correct - the owner of the accounts should be able to withdraw
Validate if original address has enough funds (may be taken care of by cw20 library)
Deposit 1/2 the amount into the addresses
What do you do if the amount is odd? - need a test case for this. Ideally, you should be able to split infinitely but in the case you cannot, you should just give addr1 the extra unit.

@TODO: change POT to equal wallet (naming doesn't really make sense imo)
@TODO: the coins are written into the contract, they don't actually get deposited into a chain address, only smart contract address
@TODO: to check against USEI, it should be against the denom, not the sender...

@ TODO: should review against each github and see the issues with each one


- you should store the withdrawable coins for every account who has non-zero coins in the contract
- you should support an execute message where an account can withdraw some or all of its withdrawable coins
- you should support a read query to get the withdrawable coins of any specified account
- you should write unit tests for all of these scenarios (we should be able to run cargo test and all of the unit tests should pass)

Bonus
- Implement a fee structure for the transfer contract, where each send incurs fees that are collectable by the contract owner

Deliverable
- Please share a public github repo with the specified smart contract

Resources
- https://docs.cosmwasm.com/docs/1.0/

### Testing