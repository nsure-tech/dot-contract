///capital_converter converts the dot token to ndot which is the token used to stake in Nsure's capital pool. Ndot reperesents your share when deposited into the capital pool. Rewards in Nsure will be distributed based on time weighted manner.
#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod capital_converter {
    use erc20::Erc20;
    use ink_env::call::FromAccountId;
    use ink_prelude::string::String;
    use ink_storage::{collections::HashMap as StorageHashMap, lazy::Lazy};
    use primitive_types::U256;

    #[ink(event)]
    pub struct Mint {
        #[ink(topic)]
        sender: Option<AccountId>,
        #[ink(topic)]
        input: Balance,
        #[ink(topic)]
        amount: Balance,
    }

    #[ink(event)]
    pub struct Burn {
        #[ink(topic)]
        sender: Option<AccountId>,
        #[ink(topic)]
        amount: Balance,
        #[ink(topic)]
        output: Balance,
    }

    #[ink(event)]
    pub struct Payouts {
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        amount: Balance,
    }

    #[ink(event)]
    pub struct SetOperator {
        #[ink(topic)]
        operator: AccountId,
    }

    #[ink(event)]
    pub struct SetMaxConvert {
        #[ink(topic)]
        max: Balance,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    /// The ERC-20 error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        InsufficientSupply,
        /// Returned if not enough allowance to fulfill a request is available.
        InsufficientAllowance,
    }

    /// The ERC-20 result type.
    pub type Result<T> = core::result::Result<T, Error>;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct CapitalConverter {
        // erc20 storage
        /// Total token supply.
        total_supply: Lazy<Balance>,
        /// Mapping from owner to number of owned token.
        balances: StorageHashMap<AccountId, Balance>,
        /// Mapping of the token amount which an account is allowed to withdraw
        /// from another account.
        allowances: StorageHashMap<(AccountId, AccountId), Balance>,
        /// Name of the token
        name: Option<String>,
        /// Symbol of the token
        symbol: Option<String>,
        /// Decimals of the token
        decimals: u8,

        // mock address to indicate DOT, like ETH: 0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE
        dot: AccountId,
        max_convert: Balance,
        token: AccountId,
        token_contract: Lazy<Erc20>,
        operator: AccountId,
        // in case of flashloan attacks
        deposit_at: StorageHashMap<AccountId, BlockNumber>,
        owner: AccountId,
    }

    impl CapitalConverter {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(
            name: Option<String>,
            symbol: Option<String>,
            decimals: u8,
            token: AccountId,
        ) -> Self {
            let caller = Self::env().caller();
            let token_contract: Erc20 = FromAccountId::from_account_id(token);
            let instance = Self {
                total_supply: Lazy::new(0),
                balances: StorageHashMap::new(),
                allowances: StorageHashMap::new(),
                name,
                symbol,
                decimals,
                owner: caller,
                dot: AccountId::from([0xdd; 32]),
                max_convert: 10000 * 10u128.saturating_pow(decimals as u32),
                token,
                token_contract: Lazy::new(token_contract),
                operator: Default::default(),
                deposit_at: StorageHashMap::new(),
            };
            instance
        }

        /// Returns the total token supply.
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            *self.total_supply
        }

        /// Returns the token name.
        #[ink(message)]
        pub fn token_name(&self) -> Option<String> {
            self.name.clone()
        }

        /// Returns the token symbol.
        #[ink(message)]
        pub fn token_symbol(&self) -> Option<String> {
            self.symbol.clone()
        }

        /// Returns the token decimals.
        #[ink(message)]
        pub fn token_decimals(&self) -> u8 {
            self.decimals
        }

        /// Wrapped mock Dot Address
        #[ink(message)]
        pub fn dot_account_id(&self) -> AccountId {
            self.dot
        }

        /// Returns the account balance for the specified `owner`.
        ///
        /// Returns `0` if the account is non-existent.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(&owner).copied().unwrap_or(0)
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        ///
        /// Returns `0` if no allowance has been set `0`.
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get(&(owner, spender)).copied().unwrap_or(0)
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let from = self.env().caller();
            self.transfer_from_to(from, to, value)
        }

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount.
        ///
        /// If this function is called again it overwrites the current allowance with `value`.
        ///
        /// An `Approval` event is emitted.
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), value);
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            Ok(())
        }

        /// Transfers `value` tokens on the behalf of `from` to the account `to`.
        ///
        /// This can be used to allow a contract to transfer tokens on ones behalf and/or
        /// to charge fees in sub-currencies, for example.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientAllowance` error if there are not enough tokens allowed
        /// for the caller to withdraw from `from`.
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the the account balance of `from`.
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance(from, caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance);
            }
            self.transfer_from_to(from, to, value)?;
            self.allowances.insert((from, caller), allowance - value);
            Ok(())
        }

        fn mint(&mut self, user: AccountId, amount: Balance) -> Result<()> {
            assert_ne!(user, Default::default());
            assert!(amount > 0, "invalid amount");

            let user_balance = self.balance_of(user);
            self.balances.insert(user, user_balance + amount);
            *self.total_supply += amount;
            self.env().emit_event(Transfer {
                from: Some(Default::default()),
                to: Some(user),
                value: amount,
            });
            Ok(())
        }

        fn burn(&mut self, user: AccountId, amount: Balance) -> Result<()> {
            if *self.total_supply < amount {
                return Err(Error::InsufficientSupply);
            }
            let user_balance = self.balance_of(user);
            if user_balance < amount {
                return Err(Error::InsufficientBalance);
            }

            self.balances.insert(user, user_balance - amount);
            *self.total_supply -= amount;
            self.env().emit_event(Transfer {
                from: Some(user),
                to: Some(Default::default()),
                value: amount,
            });
            Ok(())
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        fn transfer_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }
            self.balances.insert(from, from_balance - value);
            let to_balance = self.balance_of(to);
            self.balances.insert(to, to_balance + value);
            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn smart_balance(&self) -> Balance {
            if self.token == self.dot {
                return self.env().balance();
            }

            return self.token_contract.balance_of(self.env().account_id());
        }

        fn calculate_mint_amount(&self, deposit_amount: Balance) -> Balance {
            let da: U256 = deposit_amount.into();
            if self.total_supply() == 0 {
                let decimal = 10u128.saturating_pow(self.token_decimals() as u32);
                let dc: U256 = decimal.into();
                let decimals;
                if self.token == self.dot {
                    decimals = 10u128.saturating_pow(10);
                } else {
                    decimals =
                        10u128.saturating_pow(self.token_contract.token_decimals().unwrap() as u32);
                }
                let td: U256 = decimals.into();
                let value: U256 = da * dc / td;
                return value.as_u128();
            }
            let total_supply = self.total_supply();
            let ts: U256 = total_supply.into();
            let initial_balance = self.smart_balance().saturating_sub(deposit_amount);
            let ib: U256 = initial_balance.into();
            let value = da * ts / ib;
            return value.as_u128();
        }

        // convert ETH or USDx to nETH/nUSDx
        #[ink(message, payable)]
        pub fn convert(&mut self, amount: Balance) {
            assert!(amount > 0, "CapitalConverter: Cannot stake 0.");
            assert!(amount <= self.max_convert, "exceeding the maximum limit");
            let caller = self.env().caller();
            let block_number = self.env().block_number();
            let value = self.env().transferred_balance();
            self.deposit_at.insert(caller, block_number);

            if self.token != self.dot {
                assert!(
                    value == 0,
                    "CapitalConverter: Should not allow ETH deposits."
                );
                assert!(
                    self.token_contract
                        .transfer_from(caller, Self::env().account_id(), amount)
                        .is_ok(),
                    "transfer_from operation did not succeed"
                );
            } else {
                assert!(amount == value, "CapitalConverter: Incorrect eth amount.");
            }

            let value = self.calculate_mint_amount(amount);
            assert!(
                self.mint(caller, value).is_ok(),
                "mint operation did not succeed"
            );

            self.env().emit_event(Mint {
                sender: Some(caller),
                input: amount,
                amount: value,
            });
        }

        #[ink(message)]
        pub  fn show_token(&self) ->AccountId {
            self.token
        }

        // withdraw the ETH or USDx
        #[ink(message)]
        pub fn exit(&mut self, amount: Balance) {
            let caller = self.env().caller();

            assert!(
                self.balance_of(caller) >= amount && amount > 0,
                "CapitalConverter: insufficient assets"
            );

            assert!(
                *self.deposit_at.get(&caller).unwrap() > 0,
                "No deposit history"
            );
            assert!(
                *self.deposit_at.get(&caller).unwrap() < self.env().block_number(),
                "Reject flashloan"
            );

            let value = amount * self.smart_balance() / self.total_supply();
            if self.token != self.dot {
                assert!(
                    self.token_contract.transfer(caller, value).is_ok(),
                    "transfer operation did not succeed"
                );
            } else {
                assert!(
                    self.env().transfer(caller, value).is_ok(),
                    "transfer operation did not succeed"
                );
            }

            assert!(
                self.burn(caller, amount).is_ok(),
                "burn operation did not succeed"
            );

            self.env().emit_event(Burn {
                sender: Some(caller),
                amount,
                output: value,
            });
        }

        #[ink(message)]
        pub fn payouts(&mut self, to: AccountId, amount: Balance) {
            self.only_operator();

            assert!(to != Default::default(), "to is zero");
            if self.token != self.dot {
                assert!(
                    self.token_contract.transfer(to, amount).is_ok(),
                    "transfer operation did not succeed"
                );
            } else {
                assert!(
                    self.env().transfer(to, amount).is_ok(),
                    "transfer operation did not succeed"
                );
            }

            self.env().emit_event(Payouts {
                to: Some(to),
                amount,
            });
        }

        #[ink(message)]
        pub fn set_operator(&mut self, operator: AccountId) {
            self.only_owner();
            assert!(operator != Default::default(), "operator is zero");
            self.operator = operator;
            self.env().emit_event(SetOperator { operator });
        }

        #[ink(message)]
        pub fn set_max_convert(&mut self, max: Balance) {
            self.only_owner();
            self.max_convert = max;
            self.env().emit_event(SetMaxConvert { max });
        }

        /// Contract owner.
        #[ink(message)]
        pub fn owner(&self) -> Option<AccountId> {
            Some(self.owner)
        }

        /// transfer contract ownership to new owner.
        #[ink(message)]
        pub fn transfer_ownership(&mut self, new_owner: Option<AccountId>) {
            self.only_owner();
            if let Some(owner) = new_owner {
                self.owner = owner;
            }
        }

        #[ink(message)]
        pub  fn set_dot(&mut self,new_dot:AccountId){
            self.dot = new_dot;
        }
        fn only_owner(&self) {
            assert_eq!(self.env().caller(), self.owner);
        }

        fn only_operator(&self) {
            assert!(self.env().caller() == self.operator, "not operator");
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
  /// module and test functions are marked with a `#[test]` attribute.
  /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;
        use ink_env::AccountId;

        #[ink::test]
        fn mint_test() {
            let mut capital_converter = CapitalConverter::new(
                None,
                None,
                8,
                AccountId::from([0x00; 32]),
            );
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            assert_eq!(capital_converter.mint(accounts.alice, 1000), Ok(()));

            assert_eq!(*capital_converter.total_supply, 1000);

            assert_eq!(capital_converter.balance_of(accounts.alice), 1000);
        }

        #[ink::test]
        fn burn_test() {
            let mut capital_converter = CapitalConverter::new(
                None,
                None,
                8,
                AccountId::from([0x00; 32]),
            );
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            assert_eq!(capital_converter.mint(accounts.alice, 1000), Ok(()));

            assert_eq!(capital_converter.burn(accounts.alice, 200), Ok(()));

            assert_eq!(capital_converter.balance_of(accounts.alice), 800);

            assert_eq!(*capital_converter.total_supply, 800);
        }

        #[ink::test]
        fn transfer_test() {
            let mut capital_converter = CapitalConverter::new(
                None,
                None,
                8,
                AccountId::from([0x00; 32]),
            );
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            assert_eq!(capital_converter.mint(accounts.alice, 1000), Ok(()));

            assert_eq!(capital_converter.balance_of(accounts.bob), 0);

            assert_eq!(capital_converter.transfer(accounts.bob, 100), Ok(()));

            assert_eq!(capital_converter.balance_of(accounts.bob), 100);

            assert_eq!(capital_converter.balance_of(accounts.alice), 900);
        }

        #[ink::test]
        fn allowance_test() {
            let mut capital_converter = CapitalConverter::new(
                None,
                None,
                8,
                AccountId::from([0x00; 32]),
            );
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            assert_eq!(capital_converter.mint(accounts.alice, 1000), Ok(()));

            assert_eq!(capital_converter.approve(accounts.bob, 100), Ok(()));

            assert_eq!(capital_converter.allowance(accounts.alice, accounts.bob), 100);
        }

    }
}
