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
        VestedBalanceAlreadyExist,
        /// Vested balance not found
        VestedBalanceNotFound,
        /// Vested balance schedule not found
        VestedBalanceScheduleNotFound,
        /// Vested balance schedule not liquid
        VestedBalanceScheduleNotLiquid,
        /// Vested balance schedule not requested
        VestedBalanceScheduleNotRequested,
    }

    /// Success Messages
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Success {
        /// Vesting setup successful
        VestingSetupSuccess,
        /// Success adding vested balance
        VestedBalanceAdded,
        /// Success removing vested balance
        VestedBalanceRemoved,
        /// Success adding vested balance scheduled thawed
        VestedBalanceScheduleThawed,        
        /// Request for transfer successful
        VestedBalanceScheduleRequested,
        /// Request for transfer successful
        VestedBalanceScheduleApproved,
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
        pub recipient_address: Option<AccountId>,
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
        /// The total liquid balance
        pub liquid_balance: u128,
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

        /// Get vesting information
        #[ink(message)]
        pub fn get_vesting_info(&self,) -> (u128, u8, AccountId) {
            (
                self.asset_id,
                self.total_vested_schedule,
                self.vesting_owner,
            )
        }

        /// Add vested balances
        #[ink(message)]
        pub fn add_vested_balance(&mut self,
            address: AccountId,
            original_balance: u128,) -> Result<(), Error> {
            
            // Check the caller, it must be the owner
            let caller = self.env().caller();
            if self.env().caller() != self.vesting_owner {
                self.env().emit_event(VestingEvent {
                    operator: caller,
                    status: VestingStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if the address already exist
            if self.vested_balances.iter().any(|v| v.address == address)
            {
                self.env().emit_event(VestingEvent {
                    operator: caller,
                    status: VestingStatus::EmitError(Error::VestedBalanceAlreadyExist),
                });
                return Ok(());
            }

            // Compute for the vested balance schedules
            let mut schedules: Vec<VestedBalanceSchedule> =
                Vec::with_capacity(self.total_vested_schedule as usize);

            let schedule_balance = original_balance / self.total_vested_schedule as u128;

            for i in 1..=self.total_vested_schedule {
                schedules.push(VestedBalanceSchedule {
                    schedule_number: i,
                    schedule_balance: schedule_balance,
                    status: 0,                      // 0 = Frozen - Default status
                    recipient_address: None,     // the address is the default recipient
                    particulars: Vec::new(),
                });
            }

            // Save the vested balance
            self.vested_balances.push(VestedBalance {
                address: address,
                vested_balance_schedules: schedules,
                original_balance: original_balance,
                frozen_balance: original_balance,
                liquid_balance: 0,
                requested_balance: 0,
                transferred_balance: 0,   
            });

            self.env().emit_event(VestingEvent {
                operator: caller,
                status: VestingStatus::EmitSuccess(Success::VestedBalanceAdded),
            });

            Ok(())
        }

        /// Get a vested balance per address
        #[ink(message)]
        pub fn get_vested_balance(
            &self,
            address: AccountId,
        ) -> Option<VestedBalance> {
            self.vested_balances
                .iter()
                .find(|v| v.address == address)
                .cloned()
        }

        /// Get all vested balances
        #[ink(message)]
        pub fn get_all_vested_balance(&self,) -> Vec<VestedBalance> {
            self.vested_balances.clone()
        }
    
        /// Thaw frozen balances
        #[ink(message)]
        pub fn thaw_vested_balances(&mut self,
            schedule_number: u8,) -> Result<(), Error> {
            
            // Check the caller, it must be the owner
            let caller = self.env().caller();
            if self.env().caller() != self.vesting_owner {
                self.env().emit_event(VestingEvent {
                    operator: caller,
                    status: VestingStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Iterate all vested frozen balances on a given schedule number and thaw 
            for vested_balance in self.vested_balances.iter_mut() {

                // Change the status
                for schedule in vested_balance.vested_balance_schedules.iter_mut() {
                    if schedule.schedule_number == schedule_number && schedule.status == 0 {
                        schedule.status = 1; // 1 = Liquid (thawed)
                    }
                }

                // Calculate balances of the vested address
                Self::calculate_balances(vested_balance);
            }  

            self.env().emit_event(VestingEvent {
                operator: caller,
                status: VestingStatus::EmitSuccess(Success::VestedBalanceScheduleThawed),
            });

            Ok(())
        }

        /// Request for transfer
        #[ink(message)]
        pub fn request_transfer(&mut self,
            schedule_number: u8,
            recipient_address: AccountId) -> Result<(), Error> {

            let caller = self.env().caller();

            // 1️. Find the caller's vested balance
            if let Some(vested_balance) = self.vested_balances.iter_mut().find(|v| v.address == caller) {

                // 2️. Find the schedule in the caller's vested_balance
                if let Some(schedule) = vested_balance.vested_balance_schedules.iter_mut()
                    .find(|s| s.schedule_number == schedule_number) {

                    // 3️. Ensure the schedule is liquid
                    if schedule.status == 1 {

                        // Update the schedule
                        schedule.status = 2; // Requested
                        schedule.recipient_address = Some(recipient_address);

                        // Recalculate balances
                        Self::calculate_balances(vested_balance);

                        // Emit success event
                        self.env().emit_event(VestingEvent {
                            operator: caller,
                            status: VestingStatus::EmitSuccess(Success::VestedBalanceScheduleRequested),
                        });

                    } else {

                        // Schedule not liquid
                        self.env().emit_event(VestingEvent {
                            operator: caller,
                            status: VestingStatus::EmitError(Error::VestedBalanceScheduleNotLiquid),
                        });

                    }

                } else {

                    // Schedule not found
                    self.env().emit_event(VestingEvent {
                        operator: caller,
                        status: VestingStatus::EmitError(Error::VestedBalanceScheduleNotFound),
                    });

                }

            } else {

                // Caller has no vested balance
                self.env().emit_event(VestingEvent {
                    operator: caller,
                    status: VestingStatus::EmitError(Error::VestedBalanceNotFound),
                });

            }

            Ok(())
        }

        /// Approve transfer
        #[ink(message)]
        pub fn approve_transfer(&mut self,
            requesting_address: AccountId,
            schedule_number: u8,
            tx_hash: Vec<u8>) -> Result<(), Error> {
            
            // Check the caller, it must be the owner
            let caller = self.env().caller();
            if self.env().caller() != self.vesting_owner {
                self.env().emit_event(VestingEvent {
                    operator: caller,
                    status: VestingStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            }

            if let Some(vested_balance) = self.vested_balances.iter_mut().find(|v| v.address == requesting_address) {

                // 2️. Find the schedule in the caller's vested_balance
                if let Some(schedule) = vested_balance.vested_balance_schedules.iter_mut()
                    .find(|s| s.schedule_number == schedule_number) {

                    // 3️. Ensure the schedule is requested
                    if schedule.status == 2 {

                        // Update the schedule
                        schedule.status = 3;                    // Requested
                        schedule.particulars = tx_hash;         // Tx-hash

                        // Recalculate balances
                        Self::calculate_balances(vested_balance);

                        // Emit success event
                        self.env().emit_event(VestingEvent {
                            operator: caller,
                            status: VestingStatus::EmitSuccess(Success::VestedBalanceScheduleApproved),
                        });

                    } else {

                        // Schedule not liquid
                        self.env().emit_event(VestingEvent {
                            operator: caller,
                            status: VestingStatus::EmitError(Error::VestedBalanceScheduleNotRequested),
                        });

                    }

                } else {

                    // Schedule not found
                    self.env().emit_event(VestingEvent {
                        operator: caller,
                        status: VestingStatus::EmitError(Error::VestedBalanceScheduleNotFound),
                    });

                }

            } else {

                // Caller has no vested balance
                self.env().emit_event(VestingEvent {
                    operator: caller,
                    status: VestingStatus::EmitError(Error::VestedBalanceNotFound),
                });

            }            

            Ok(())
        }

        /// Removes the balance and its schedules regardless of the status
        #[ink(message)]
        pub fn remove_vested_balance(&mut self,
            address: AccountId,) -> Result<(), Error> {

            // Check the caller, it must be the owner
            let caller = self.env().caller();
            if self.env().caller() != self.vesting_owner {
                self.env().emit_event(VestingEvent {
                    operator: caller,
                    status: VestingStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            let index = match self
                .vested_balances
                .iter()
                .position(|v| v.address == address)
            {
                Some(i) => i,
                None => {
                    self.env().emit_event(VestingEvent {
                        operator: caller,
                        status: VestingStatus::EmitError(Error::VestedBalanceNotFound),
                    });
                    return Ok(());
                }
            };

            self.vested_balances.swap_remove(index);

            self.env().emit_event(VestingEvent {
                operator: caller,
                status: VestingStatus::EmitSuccess(Success::VestedBalanceRemoved),
            });

            Ok(())
        }
        
        /// Helper function to calculate balances
        fn calculate_balances(vested_balance: &mut VestedBalance) {
            vested_balance.frozen_balance = 0;
            vested_balance.liquid_balance = 0;
            vested_balance.requested_balance = 0;
            vested_balance.transferred_balance = 0;

            for schedule in vested_balance.vested_balance_schedules.iter() {
                match schedule.status {
                    0 => vested_balance.frozen_balance += schedule.schedule_balance,
                    1 => vested_balance.liquid_balance += schedule.schedule_balance,
                    2 => vested_balance.requested_balance += schedule.schedule_balance,
                    3 => vested_balance.transferred_balance += schedule.schedule_balance,
                    _ => {}, // status 1 = Liquid, ignored
                }
            }
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
