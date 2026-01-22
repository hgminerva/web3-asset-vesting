#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod vesting {

    use ink::prelude::vec::Vec;

    /// Error Messages
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Bad origin error, e.g., wrong caller
        BadOrigin,
        /// There is already an existing vested balance for that address
        VestedBalanceExist,
    }

    /// Success Messages
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Success {
        /// Vesting setup successful
        VestingSetupSuccess,
        /// Success adding vested balance
        VestedBalanceAdded,
    }

    /// Vesting Status
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum VestingStatus {
        EmitSuccess(Success),
        EmitError(Error),
    }

    /// Vesting Event
    #[ink(event)]
    pub struct VestingEvent {
        #[ink(topic)]
        operator: AccountId,
        status: VestingStatus,
    } 

    /// Vested balance schedules
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct VestedBalanceSchedule {
        /// Schedule number 1-100
        pub schedule_number: u8,
        /// Schedule balance
        pub schedule_balance: u128,
        /// Status (0-Frozen, 1-Liquid, 2-Requested, 3-Transferred)
        pub status: u8,
        /// Transfer recipient
        pub recipient_address: AccountId,
        /// Particulars
        pub particulars: Vec<u8>,
    }    

    /// Vested balances
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct VestedBalance {
        /// The address that holds the vested balance
        pub address: AccountId,
        /// Vested schedules
        pub vested_balance_schedules: Vec<VestedBalanceSchedule>,
        /// The original balance
        pub original_balance: u128,
        /// The total frozen balance
        pub frozen_balance: u128,
        /// The total requested balance
        pub requested_balance: u128,
        /// The total transferred balance
        pub transferred_balance: u128,   
    }

    /// Contract Storage
    #[ink(storage)]
    pub struct Vesting {
        /// The asset that is vested.
        pub asset_id: u128,
        /// Total number of scheduled vested balances
        pub total_vested_schedule: u8,
        /// Vested balances
        pub vested_balances: Vec<VestedBalance>,
        /// Vesting owner
        pub vesting_owner: AccountId,
    }

    impl Vesting {
        /// Constructor 
        #[ink(constructor)]
        pub fn new(asset_id: u128, total_vested_schedule: u8) -> Self {

            let caller = Self::env().caller();

            Self { 
                asset_id: asset_id, 
                total_vested_schedule: total_vested_schedule,
                vested_balances: Vec::new(),
                vesting_owner: caller,
            }

        }

        /// Default
        #[ink(constructor)]
        pub fn default() -> Self {

            Self::new(0u128, 0u8)

        }

        /// Setup vesting
        #[ink(message)]
        pub fn setup_vesting(&mut self,
            asset_id: u128,
            total_vested_schedule: u8,) -> Result<(), Error> {
            
            let caller = self.env().caller();
            if self.env().caller() != self.vesting_owner {
                self.env().emit_event(VestingEvent {
                    operator: caller,
                    status: VestingStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // The setup will erase the existing vested balances
            self.asset_id = asset_id;
            self.total_vested_schedule = total_vested_schedule;
            self.vested_balances =  Vec::new();
            
            self.env().emit_event(VestingEvent {
                operator: caller,
                status: VestingStatus::EmitSuccess(Success::VestingSetupSuccess),
            });

            Ok(())
        }

        #[ink(message)]
        pub fn get_vesting_info(&self,) -> (u128, u8, AccountId) {
            (
                self.asset_id,
                self.total_vested_schedule,
                self.vesting_owner,
            )
        }

    }

    /// Unit tests
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let vesting = Vesting::default();
        }

    }


    /// End-to-end (E2E) or integration tests for ink! contracts.
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = VestingRef::default();

            // When
            let contract_account_id = client
                .instantiate("vesting", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<VestingRef>(contract_account_id.clone())
                .call(|web3_asset_vesting| web3_asset_vesting.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = VestingRef::new(false);
            let contract_account_id = client
                .instantiate("vesting", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<VestingRef>(contract_account_id.clone())
                .call(|web3_asset_vesting| web3_asset_vesting.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<VestingRef>(contract_account_id.clone())
                .call(|web3_asset_vesting| web3_asset_vesting.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<VestingRef>(contract_account_id.clone())
                .call(|web3_asset_vesting| web3_asset_vesting.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
