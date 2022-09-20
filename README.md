# Split Coins

## Commands
- `cargo wasm` to build the smart contract
- `cargo test` to run smart contract unit tests
- `cargo fmt` to fmt rust code

## Coding Challenge
We want you to build a simple Sei smart contract that allows 1-to-2 transfer of the usei token. 

### Requirements

Please create a Sei smart contract (using Cosmwasm 1.0) with the following requirements:
- **you should be able to instantiate the contract and set the owner.** [Code](src/contract.rs#20:52). Set owner in `InstantiateMsg`. If `owner=None`, then assume `owner=info.sender`.

- **you should support a read query to get the owner of the smart contract.** [Code](src/contract.rs#180:185). Load `CONFIG` and get the `config.owner` value. One design choice I made here was to return `OwnerResponse` singleton struct instead of just `Addr`. The reason was because other example contracts typically define a Response struct type so I wanted consistency across queries.

- **you should support an execute message where an account can send coins to the contract and specify two accounts that can withdraw the coins (for simplicity, split coins evenly across the two destination accounts).** [Code](src/contract.rs#70:129). Validate that the caller only sent usei token to `execute(...)`. Then split the usei tokens sent between addr1 and addr2. If the amount is odd, give the extra token to addr1 (this edge case could be resolved many other ways - Eg. burn token or give the extra token to contract owner etc).

- **you should store the withdrawable coins for every account who has non-zero coins in the contract.** [Code](src/state.rs#16).
- **you should support an execute message where an account can withdraw some or all of its withdrawable coins.** [Code](src/contract.rs#131:169). If the caller sets `amount=?`, try to withdraw `amount`. Otherwise, if the caller sets `amount=None`, assume the caller is trying to withdraw the entire balance. During the execute, check `WALLETS` and update `key=info.sender` based on how much the caller is trying to withdraw and error check accordingly.
- **you should support a read query to get the withdrawable coins of any specified account.** [Code](src/contract.rs#187:193).
- **you should write unit tests for all of these scenarios (we should be able to run cargo test and all of the unit tests should pass)**. [Code](src/tests.rs). The test naming and comments should indicate what each test case is testing for. The tests should cover both valid and invalid cases for each individual functionality.


Bonus
- **Implement a fee structure for the transfer contract, where each send incurs fees that are collectable by the contract owner**. [Code](src/contract.rs#30:32). Allow the contract owner to define the `fee_percent` to be collected on initialization. Due to clunkiness of `cosmowasm_std::Decimal` and since `f64` is unserializable by `deps.storage`, assumed that fee_percent \[0%-100%\] granularity would suffice. Set default `fee_percent=0` if contract owner put `fee_percent=None`. Store the fees collected in `WALLETS` at `config.cw20_address` - defined on Instantiation. This allows us to leverage `ExecuteMsg::WithdrawCoins` to easily withdraw fees collected.


Resources
- https://docs.cosmwasm.com/docs/1.0/

