#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod vesting {

    /// Error Messages
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// There is already an existing vested balance for that address
        VestedBalanceExist,
    }

    /// Success Messages
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Success {
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

    /// Vested balances
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct VestedBalance {
        /// The address that holds the vested balance
        pub address: AccountId,
        /// The original balance
        pub original_balance: u128,
        /// This balance is available for transfer request
        pub free_balance: u128,
        /// This balance is still frozen
        pub frozen_balance: u128,
        /// This is the total balance that is already requested and transferred to a recipient
        pub total_requested_balance: u128,        
    }

    /// List of requested transfers
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct RequestedBalance {
        /// The address that holds the vested balance
        pub address: AccountId,
        /// The recipient address of the frozen balance
        pub recipient_address: AccountId,
        /// Requested balance
        pub requested_balance: u128, 
        /// Transfer transaction hash
        pub tx_hash: Vec<u8>,
    }

    /// Contract Storage
    #[ink(storage)]
    pub struct Vesting {
        /// The asset that is vested.
        pub asset_id: u128,
        /// Vested balances
        pub vested_balances: Vec<VestedBalance>,
        /// Requested balances
        pub requested_balances: Vec<RequestedBalance>,
    }

    impl Vesting {
        /// Constructor 
        #[ink(constructor)]
        pub fn new(asset_id: u128) -> Self {

            Self { 
                asset_id: asset_id, 
                vested_balances: Vec::new(),
                requested_balances: Vec::new(),
            }

        }

        /// Default
        #[ink(constructor)]
        pub fn default() -> Self {

            Self::new(0u128)

        }

        /// Add a vested balance
        #[ink(message)]
        pub fn add_vested_balance(&mut self,
            address: AccountId,
            original_balance: u128,) -> Result<(), Error> {
            
            let caller = self.env().caller();

            /// The initial vested balance are frozen
            let new_vested_balance = VestedBalance {
                address: address,
                original_balance: original_balance,
                free_balance: 0,
                frozen_balance: original_balance,
                total_requested_balance: 0,
            };

            self.vested_balances.push(new_vested_balance);

            self.env().emit_event(VestingEvent {
                operator: caller,
                status: VestingStatus::EmitSuccess(Success::VestedBalanceAdded),
            });

            Ok(())
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
            assert_eq!(vesting.get(), false);
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
