#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod vesting {

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
        pub vested_balances: Vec<VestedBalance>,
    }

    impl Web3AssetVesting {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let web3_asset_vesting = Web3AssetVesting::default();
            assert_eq!(web3_asset_vesting.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut web3_asset_vesting = Web3AssetVesting::new(false);
            assert_eq!(web3_asset_vesting.get(), false);
            web3_asset_vesting.flip();
            assert_eq!(web3_asset_vesting.get(), true);
        }
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
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
            let constructor = Web3AssetVestingRef::default();

            // When
            let contract_account_id = client
                .instantiate("web3_asset_vesting", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<Web3AssetVestingRef>(contract_account_id.clone())
                .call(|web3_asset_vesting| web3_asset_vesting.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = Web3AssetVestingRef::new(false);
            let contract_account_id = client
                .instantiate("web3_asset_vesting", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<Web3AssetVestingRef>(contract_account_id.clone())
                .call(|web3_asset_vesting| web3_asset_vesting.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<Web3AssetVestingRef>(contract_account_id.clone())
                .call(|web3_asset_vesting| web3_asset_vesting.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<Web3AssetVestingRef>(contract_account_id.clone())
                .call(|web3_asset_vesting| web3_asset_vesting.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
