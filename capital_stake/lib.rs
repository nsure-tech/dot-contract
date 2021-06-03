#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod capital_stake {
    use erc20::Erc20;
    use ink_env::call::FromAccountId;
    use ink_prelude::{vec, vec::Vec};
    use ink_storage::{
        collections::HashMap as StorageHashMap,
        lazy::Lazy,
        traits::{PackedLayout, SpreadLayout},
    };

    #[ink(event)]
    pub struct Deposit {
        #[ink(topic)]
        user: AccountId,
        #[ink(topic)]
        pid: u32,
        #[ink(topic)]
        amount: Balance,
    }

    #[ink(event)]
    pub struct SetOperator {
        #[ink(topic)]
        operator: AccountId,
    }

    #[ink(event)]
    pub struct SetSigner {
        #[ink(topic)]
        signer: AccountId,
    }

    #[ink(event)]
    pub struct SwitchDeposit {
        #[ink(topic)]
        swi: bool,
    }

    #[ink(event)]
    pub struct SetUserCapacityMax {
        #[ink(topic)]
        pid: u32,
        #[ink(topic)]
        max: Balance,
    }

    #[ink(event)]
    pub struct SetCapacityMax {
        #[ink(topic)]
        pid: u32,
        #[ink(topic)]
        max: Balance,
    }

    #[ink(event)]
    pub struct UpdateBlockReward {
        #[ink(topic)]
        reward: Balance,
    }

    #[ink(event)]
    pub struct UpdateWithdrawPending {
        #[ink(topic)]
        seconds: u64,
    }

    #[ink(event)]
    pub struct Add {
        #[ink(topic)]
        point: u128,
        #[ink(topic)]
        token: AccountId,
        #[ink(topic)]
        update: bool,
    }

    #[ink(event)]
    pub struct Set {
        #[ink(topic)]
        pid: u32,
        #[ink(topic)]
        point: u128,
        #[ink(topic)]
        update: bool,
    }

    #[ink(event)]
    pub struct EDeposit {
        #[ink(topic)]
        user: AccountId,
        #[ink(topic)]
        pid: u32,
        #[ink(topic)]
        amount: Balance,
    }

    #[ink(event)]
    pub struct Unstake {
        #[ink(topic)]
        user: AccountId,
        #[ink(topic)]
        pid: u32,
        #[ink(topic)]
        amount: Balance,
    }

    #[ink(event)]
    pub struct Withdraw {
        #[ink(topic)]
        user: AccountId,
        #[ink(topic)]
        pid: u32,
        #[ink(topic)]
        amount: Balance,
    }

    #[ink(event)]
    pub struct Claim {
        #[ink(topic)]
        user: AccountId,
        #[ink(topic)]
        pid: u32,
        #[ink(topic)]
        amount: Balance,
    }

    // pub const NAME: String = "CapitalStake".to_string();
    // pub const VERSION: String = "1".to_string();

    // Info of each user.
    #[derive(
        Debug, PartialEq, Eq, scale::Encode, scale::Decode, Clone, Copy, SpreadLayout, PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct UserInfo {
        // How many  tokens the user has provided.
        pub amount: Balance,
        // Reward debt. See explanation below.
        pub reward_debt: u128,
        pub reward: Balance,
        // payments available for withdrawal by an investor
        pub pending_withdrawal: Balance,
        pub pending_at: u64,
    }

    // Info of each pool.
    #[derive(
        Debug, PartialEq, Eq, scale::Encode, scale::Decode, Clone, SpreadLayout, PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct PoolInfo {
        //Total Deposit of token
        pub amount: Balance,
        // Address of token contract.
        pub lp_token: AccountId,
        pub alloc_point: u128,
        pub last_reward_block: BlockNumber,
        pub acc_nsure_per_share: u128,
        pub pending: u128,
    }

    #[ink(storage)]
    pub struct CapitalStake {
        signer: AccountId,
        nsure: Lazy<Erc20>,
        nsure_per_block: Balance,
        // 14 Days
        pending_duration: u64,
        capacity_max: StorageHashMap<u32, Balance>,
        can_deposit: bool,
        operator: AccountId,
        // the max capacity for one user's deposit.
        user_capacity_max: StorageHashMap<u32, Balance>,
        // Info of each pool.
        pool_info: Vec<PoolInfo>,
        /// @notice A record of states for signing / validating signatures
        nonces: StorageHashMap<AccountId, u128>,
        user_info: StorageHashMap<(u32, AccountId), UserInfo>,
        // Total allocation poitns. Must be the sum of all allocation points in all pools.
        total_alloc_point: u128,
        start_block: BlockNumber,
        owner: AccountId,
//user info
         amount: StorageHashMap<AccountId, Balance>,
        // Reward debt. See explanation below.
         reward_debt: StorageHashMap<AccountId, Balance>,
         reward: StorageHashMap<AccountId, Balance>,
        // payments available for withdrawal by an investor
         pending_withdrawal: StorageHashMap<AccountId, Balance>,
         pending_at: StorageHashMap<AccountId, u64>,
    }

    impl CapitalStake {
        #[ink(constructor)]
        pub fn new(signer: AccountId, nsure: AccountId, start_block: BlockNumber) -> Self {
            let nsure: Erc20 = FromAccountId::from_account_id(nsure);
            Self {
                signer,
                nsure: Lazy::new(nsure),
                nsure_per_block: 18 * 10u128.saturating_pow(10),
                capacity_max: StorageHashMap::new(),
                operator: Default::default(),
                user_capacity_max: StorageHashMap::new(),
                nonces: StorageHashMap::new(),
                user_info: StorageHashMap::new(),
                total_alloc_point: 0,
                pending_duration: 10,
                can_deposit: true,
                pool_info: vec![],
                start_block,
                owner: Self::env().caller(),

                amount: StorageHashMap::new(),
                // Reward debt. See explanation below.
                reward_debt: StorageHashMap::new(),
                reward: StorageHashMap::new(),
                // payments available for withdrawal by an investor
                pending_withdrawal: StorageHashMap::new(),
                pending_at: StorageHashMap::new(),
            }
        }

        #[ink(message)]
        pub fn set_default(&mut self){
            self.only_owner();
            self.user_capacity_max.insert(0, 99999999999999);
            self.capacity_max.insert(0, 99999999999999);
        }

        #[ink(message)]
        pub fn get_user_info(&self,user:AccountId) ->(Balance,Balance,Balance,Balance,u64) {
            let  amount = self.amount.get(&user).map(|i|*i).unwrap_or(0u128);
            let  reward_debt = self.reward_debt.get(&user).map(|i|*i).unwrap_or(0u128);
            let  pending_withdrawal_info = self.pending_withdrawal.get(&user).map(|i|*i).unwrap_or(0u128);
            let  reward = self.reward.get(&user).map(|i|*i).unwrap_or(0u128);

            let  pending_at_info = self.pending_at.get(&user).map(|i|*i).unwrap_or(0u64);
            return (amount,reward_debt,pending_withdrawal_info,reward,pending_at_info)
        }

        #[ink(message)]
        pub fn get_pool_info(&self,pid:u32) -> (Balance,AccountId,u128,u32,u128,u128) {
            let pool: &PoolInfo = self.pool_info.get(pid as usize).unwrap();
            return (pool.amount,pool.lp_token,pool.alloc_point,pool.last_reward_block,pool.acc_nsure_per_share,pool.pending)
        }

        #[ink(message)]
        pub fn set_operator(&mut self, operator: AccountId) {
            self.only_owner();
            assert!(operator != Default::default(), "operator is zero");
            self.operator = operator;
            self.env().emit_event(SetOperator { operator });
        }

        #[ink(message)]
        pub fn set_signer(&mut self, signer: AccountId) {
            self.only_owner();
            assert!(signer != Default::default(), "signer is zero");
            self.signer = signer;
            self.env().emit_event(SetSigner { signer });
        }

        #[ink(message)]
        pub fn switch_deposit(&mut self) {
            self.only_owner();
            self.can_deposit = !self.can_deposit;
            self.env().emit_event(SwitchDeposit {
                swi: self.can_deposit,
            });
        }

        #[ink(message)]
        pub fn set_user_capacity_max(&mut self, pid: u32, max: Balance) {
            self.only_owner();
            self.user_capacity_max.insert(pid, max);
            self.env().emit_event(SetUserCapacityMax { pid, max });
        }

        #[ink(message)]
        pub fn set_capacity_max(&mut self, pid: u32, max: Balance) {
            self.only_owner();
            self.capacity_max.insert(pid, max);
            self.env().emit_event(SetCapacityMax { pid, max });
        }

        #[ink(message)]
        pub fn update_block_reward(&mut self, reward: Balance) {
            self.only_owner();
            self.nsure_per_block = reward;
            self.env().emit_event(UpdateBlockReward { reward });
        }

        #[ink(message)]
        pub fn update_withdraw_pending(&mut self, seconds: u64) {
            self.only_owner();
            self.pending_duration = seconds;
            self.env().emit_event(UpdateWithdrawPending { seconds });
        }

        #[ink(message)]
        pub fn pool_length(&self) -> u32 {
            self.pool_info.len() as u32
        }

        // Add a new lp to the pool. Can only be called by the owner.
        #[ink(message)]
        pub fn add(
            &mut self,
            alloc_point: u128,
            lp_token: AccountId,
            with_update: bool,
            max_capacity: Balance,
        ) {
            self.only_owner();
            assert_ne!(lp_token, Default::default(), "lp_token is zero");

            for i in 0..self.pool_length() as usize {
                assert_ne!(
                    lp_token,
                    self.pool_info.get(i).unwrap().lp_token,
                    "Duplicate Token!"
                );
            }

            if with_update {
                self.mass_update_pools();
            }

            self.capacity_max
                .insert(self.pool_info.len() as u32, max_capacity);
            let block_number = self.env().block_number();
            let last_reward_block;
            if block_number > self.start_block {
                last_reward_block = block_number;
            } else {
                last_reward_block = self.start_block;
            }

            self.total_alloc_point = self.total_alloc_point + alloc_point;

            self.pool_info.push(PoolInfo {
                amount: 0,
                lp_token,
                alloc_point,
                last_reward_block,
                acc_nsure_per_share: 0,
                pending: 0,
            });

            self.env().emit_event(Add {
                point: alloc_point,
                token: lp_token,
                update: with_update,
            });
        }

        #[ink(message)]
        pub fn set(&mut self, pid: u32, alloc_point: u128, with_update: bool) {
            self.only_owner();
            assert!(pid < self.pool_info.len() as u32, "invalid _pid");
            if with_update {
                self.mass_update_pools();
            }

            let pool: &mut PoolInfo = self.pool_info.get_mut(pid as usize).unwrap();
            self.total_alloc_point = self.total_alloc_point - pool.alloc_point + alloc_point;
            pool.alloc_point = alloc_point;

            self.env().emit_event(Set {
                pid,
                point: alloc_point,
                update: with_update,
            });
        }

        #[ink(message)]
        pub fn show_accoutn_id(&self) -> AccountId {
            let self_account = self.env().account_id();
            self_account
        }

        #[ink(message)]
        pub fn get_Height(&self) -> u32 {
            let block_number = self.env().block_number();
            return block_number;
        }

        #[ink(message)]
        pub fn pending_nsure(& self, pid: u32, user: AccountId) -> u128 {
            let pool: &PoolInfo = self.pool_info.get(pid as usize).unwrap();
            // let user = self.user_info.get(&(pid, user)).unwrap();
            // let amount = self.amount.get(&user).unwrap();
            let  amount = self.amount.get(&user).map(|i|*i).unwrap_or(0u128);
            // let reward_debt = self.reward_debt.get(&user).unwrap();
            let  reward_debt = self.reward_debt.get(&user).map(|i|*i).unwrap_or(0u128);

            let self_account = self.env().account_id();
            let block_number = self.env().block_number();

            let mut acc_nsure_per_share = pool.acc_nsure_per_share;

            let lp_token: Erc20 = FromAccountId::from_account_id(pool.lp_token);
            let lp_supply = lp_token.balance_of(self_account);
            if block_number > pool.last_reward_block && lp_supply != 0 {
                let multiplier = Self::get_multiplier(pool.last_reward_block, block_number);
                let nsure_reward =
                    multiplier * self.nsure_per_block * pool.alloc_point / self.total_alloc_point;
                acc_nsure_per_share = acc_nsure_per_share + nsure_reward * 10u128.saturating_pow(12) / lp_supply;
            }

            amount * acc_nsure_per_share / 10u128.saturating_pow(12) - reward_debt
        }

        #[ink(message)]
        pub fn show_pooinfo(&self,pid: u32) -> u32 {
            let pool = self.pool_info.get(pid as usize).unwrap();
            pool.last_reward_block
        }


        #[ink(message)]
        pub fn show_user_info_amount(&self) -> Balance {
            let caller = self.env().caller();
            *self.amount.get(&caller).unwrap()
        }

        #[ink(message)]
        pub fn deposit(&mut self, pid: u32, amount: Balance) {
            assert!(self.can_deposit, "can not");
            assert!(pid < self.pool_info.len() as u32, "invalid _pid");
            let caller = self.env().caller();
            let self_account = self.env().account_id();

            let pool = self.pool_info.get(pid as usize).unwrap();

                        // let user = self.user_info.get(&(pid, caller)).unwrap();

                        // assert!(
                        //     user.amount + amount <= *self.user_capacity_max.get(&pid).unwrap(),
                        //     "exceed user limit"
                        // );
                        assert!(
                            pool.amount + amount <= *self.capacity_max.get(&pid).unwrap(),
                            "exceed the total limit"
                        );
            self.update_pool(pid);

            let pool = self.pool_info.get_mut(pid as usize).unwrap();
                         ///// let user = self.user_info.get_mut(&(pid, caller)).unwrap();

            let mut amount_info = self.amount.get_mut(&caller).map(|i|*i).unwrap_or(0u128);
            let mut reward_debt = self.reward_debt.get_mut(&caller).map(|i|*i).unwrap_or(0u128);

///todo let amount_info = self.amount.get_mut(&caller).map(|i|*i).unwrap_or(0u128);
            let mut lp_token: Erc20 = FromAccountId::from_account_id(pool.lp_token);
            assert!(lp_token.transfer_from(caller, self_account, amount).is_ok());

            let pending = amount_info * (pool.acc_nsure_per_share) /  10u128.saturating_pow(12)   - reward_debt;

            amount_info = amount_info + amount;
            reward_debt = amount_info * pool.acc_nsure_per_share / 10u128.saturating_pow(12);
            pool.amount = pool.amount + amount;
            
            self.amount.insert(caller, amount_info);

            if pending > 0 {
                self.safe_nsure_transfer(caller, pending);
            }

            self.env().emit_event(EDeposit {
                user: caller,
                pid,
                amount,
            });
        }

        // unstake, need pending sometime
        #[ink(message)]
        pub fn unstake(&mut self, pid: u32, amount: Balance) {
            // TODO verify signer

            assert!(pid < self.pool_info.len() as u32, "invalid _pid");
            let caller = self.env().caller();
            // let user = self.user_info.get(&(pid, caller)).unwrap();
            // assert!(user.amount >= amount, "unstake: insufficient assets");


            // let mut amount_info = self.amount.get_mut(&caller).map(|i|*i).unwrap_or(0u128);
            // let mut reward_debt = self.reward_debt.get_mut(&caller).map(|i|*i).unwrap_or(0u128);
            // let pending_at_info = self.pending_at.get_mut(&caller).map(|i|*i).unwrap_or(0u64);
            // let mut pending_withdrawal_info = self.pending_withdrawal.get_mut(&caller).map(|i|*i).unwrap_or(0u128);


            self.update_pool(pid);
            let pool = self.pool_info.get_mut(pid as usize).unwrap();
            // let user = self.user_info.get_mut(&(pid, caller)).unwrap();

            let mut amount_info = self.amount.get_mut(&caller).map(|i|*i).unwrap_or(0u128);
            let mut reward_debt = self.reward_debt.get_mut(&caller).map(|i|*i).unwrap_or(0u128);
            let mut pending_at_info = self.pending_at.get_mut(&caller).map(|i|*i).unwrap_or(0u64);
            let mut pending_withdrawal_info = self.pending_withdrawal.get_mut(&caller).map(|i|*i).unwrap_or(0u128);


            let pending = amount_info * pool.acc_nsure_per_share / 10u128.saturating_pow(12) - reward_debt;

            amount_info = amount_info - amount;
            reward_debt = amount_info * pool.acc_nsure_per_share / 10u128.saturating_pow(12);

            pending_at_info = Self::env().block_timestamp();
            pending_withdrawal_info = pending_withdrawal_info + amount;

            pool.pending = pool.pending + amount;

            self.safe_nsure_transfer(caller, pending);

            self.env().emit_event(Unstake {
                user: caller,
                pid,
                amount,
            });
        }

        // when it's pending while a claim occurs, the value of the withdrawal will decrease as usual
        // so we keep the claim function by this tool.
        #[ink(message)]
        pub fn withdraw(&mut self, pid: u32) {
            assert!(pid < self.pool_info.len() as u32, "invalid _pid");
            let caller = self.env().caller();
            let timestamp = self.env().block_timestamp();
            let pool: &mut PoolInfo = self.pool_info.get_mut(pid as usize).unwrap();
            // let user = self.user_info.get_mut(&(pid, caller)).unwrap();

            let mut amount_info = self.amount.get_mut(&caller).map(|i|*i).unwrap_or(0u128);
            let mut reward_debt = self.reward_debt.get_mut(&caller).map(|i|*i).unwrap_or(0u128);
            let pending_at_info = self.pending_at.get_mut(&caller).map(|i|*i).unwrap_or(0u64);
            let mut pending_withdrawal_info = self.pending_withdrawal.get_mut(&caller).map(|i|*i).unwrap_or(0u128);

            // assert!(
            //     timestamp >= pending_at_info + self.pending_duration,
            //     "still pending"
            // );

            // let amount = pending_withdrawal_info;
            let amount = amount_info;
            pool.amount = pool.amount - amount;
            pool.pending = pool.pending - amount;

            pending_withdrawal_info = 0;

            let mut lp_token: Erc20 = FromAccountId::from_account_id(pool.lp_token);
            assert!(lp_token.transfer(caller, amount).is_ok());
            self.amount.insert(caller, 0);
            self.env().emit_event(Withdraw {
                user: caller,
                pid,
                amount,
            });
        }

        //claim reward
        #[ink(message)]
        pub fn claim(&mut self, pid: u32) {
            assert!(pid < self.pool_info.len() as u32, "invalid _pid");
            let caller = self.env().caller();

            self.update_pool(pid);

            let pool = self.pool_info.get(pid as usize).unwrap();
            // let user = self.user_info.get_mut(&(pid, caller)).unwrap();

            let mut amount_info = self.amount.get_mut(&caller).map(|i|*i).unwrap_or(0u128);
            let mut reward_debt = self.reward_debt.get_mut(&caller).map(|i|*i).unwrap_or(0u128);
            let pending_at_info = self.pending_at.get_mut(&caller).map(|i|*i).unwrap_or(0u64);
            let mut pending_withdrawal_info = self.pending_withdrawal.get_mut(&caller).map(|i|*i).unwrap_or(0u128);


            reward_debt = amount_info * pool.acc_nsure_per_share / 10u128.saturating_pow(12);
            let pending = amount_info * pool.acc_nsure_per_share / 10u128.saturating_pow(12) - reward_debt;
            self.safe_nsure_transfer(caller, pending);

            self.env().emit_event(Claim {
                user: caller,
                pid,
                amount: pending,
            });
        }

        pub fn is_pending(&self, pid: u32) -> (bool, u64) {
            let caller = self.env().caller();
            // let user = self.user_info.get(&(pid, caller)).unwrap();
            let  amount_info = self.amount.get(&caller).map(|i|*i).unwrap_or(0u128);
            let  reward_debt = self.reward_debt.get(&caller).map(|i|*i).unwrap_or(0u128);
            let pending_at_info = self.pending_at.get(&caller).map(|i|*i).unwrap_or(0u64);
            let  pending_withdrawal_info = self.pending_withdrawal.get(&caller).map(|i|*i).unwrap_or(0u128);

            let timestamp = self.env().block_timestamp();
            if timestamp >= pending_at_info + self.pending_duration {
                return (false, 0);
            }

            return (true, pending_at_info + self.pending_duration - timestamp);
        }

        fn mass_update_pools(&mut self) {
            let length = self.pool_info.len() as u32;
            for pid in 0..length {
                self.update_pool(pid);
            }
        }

        #[ink(message)]
        pub fn show_block_num(&self) -> u32 {
            self.env().block_number()
        }


        fn update_pool(&mut self, pid: u32) {
            assert!(pid < self.pool_info.len() as u32, "invalid _pid");
            let self_account = self.env().account_id();
            let block_number = self.env().block_number();
            let pool: &mut PoolInfo = self.pool_info.get_mut(pid as usize).unwrap();
            if block_number <= pool.last_reward_block {
                return;
            }

            let lp_token: Erc20 = FromAccountId::from_account_id(pool.lp_token);
            let lp_supply = lp_token.balance_of(self_account);
            if lp_supply == 0 {
                pool.last_reward_block = block_number;
                return;
            }

            let multiplier = Self::get_multiplier(pool.last_reward_block, block_number);
            let nsure_reward = multiplier as u128 * self.nsure_per_block * pool.alloc_point
                / self.total_alloc_point;

                //test
                    let rewardtest :Balance = 40000000000;
                //test
            // if self.nsure.mint(self_account, nsure_reward).is_ok() {
                if self.nsure.mint(self_account, rewardtest).is_ok() {
                pool.acc_nsure_per_share =
                    pool.acc_nsure_per_share + (nsure_reward * 10u128.saturating_pow(12) / lp_supply);
                pool.last_reward_block = block_number;
            }
        }

        #[ink(message)]
        pub fn  show_mint(&mut self,user:AccountId ) {
            self.nsure.mint(user,3);
        }

        // #[ink(message)]
        // pub fn show_nsure(&self) -> Lazy<Erc20>{
        //     self.nsure
        // }
        fn get_multiplier(from: BlockNumber, to: BlockNumber) -> u128 {
            (to - from) as u128
        }

        fn safe_nsure_transfer(&mut self, to: AccountId, amount: Balance) {
            assert_ne!(to, Default::default(), "to is zero");
            let self_account = self.env().account_id();
            let nsure_bal = self.nsure.balance_of(self_account);
            if amount > nsure_bal {
                assert!(self.nsure.transfer(to, nsure_bal).is_ok());
            } else {
                assert!(self.nsure.transfer(to, amount).is_ok());
            }
        }

        #[ink(message)]
        pub fn owner(&self) -> Option<AccountId> {
            Some(self.owner)
        }

        #[ink(message)]
        pub fn transfer_ownership(&mut self, new_owner: Option<AccountId>) {
            self.only_owner();
            if let Some(owner) = new_owner {
                self.owner = owner;
            }
        }

        fn only_owner(&self) {
            assert_eq!(self.env().caller(), self.owner);
        }

        fn only_operator(&self) {
            assert!(self.env().caller() == self.operator, "not operator");
        }
    }
}
