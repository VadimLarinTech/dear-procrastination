mod utils;
mod web4;

use crate::utils::unordered_map_pagination;
use core::option::Option;
use std::borrow::Borrow;
use near_sdk::Balance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::Promise;
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, Timestamp};
use num_traits::cast::ToPrimitive;

const MIN_DEPOSIT: u128 = 3000000000000000000000000;

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Contract {
    /// A list of users
    pub common_records: LookupMap<AccountId, UserRecords>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct UserRecords {
    /// A list of task records
    pub user_records: UnorderedMap<i64, Record>,
    /// Uniq id of record, increases by increment
    pub record_id: i64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Record {
    /// Description of task
    pub task: String,
    /// Status of completing task, false - not complete, true - complete
    pub is_complete_status: bool,
    /// A deposit that is paid when creating a task and is returned to the user if the task is not
    /// completed within the specified time.
    pub guarantee_of_task_completion: u128,
    /// When this time is reached, the deposit for the failed task is not returned
    /// and is considered a payment for procrastination.
    pub deadline_time: Timestamp,
    /// User balance at the time of task creation, in Near
    pub account_balance: Balance,
    /// User deposit status, can be "Contributed", "Refunded", "Withheld"
    pub deposit_status: DepositStatus,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    CommonRecords,
    UserRecords,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum DepositStatus {
    Contributed,
    Refunded,
    Withheld,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            common_records: LookupMap::new(StorageKey::CommonRecords),
        }
    }
}

impl Default for UserRecords {
    fn default() -> Self {
        Self {
            user_records: UnorderedMap::new(StorageKey::UserRecords),
            record_id: 1,
        }
    }
}

#[near_bindgen]
impl Contract {
    /// The method creates a task
    /// to create a task it is necessary to make a deposit of at least 3 Near
    /// it is also necessary to specify the deadline for the task in Timestamp
    #[payable]
    pub fn create_task(
        &mut self,
        task: String,
        deadline_time: Timestamp,
    ) {
        assert!(
            env::attached_deposit().borrow().to_u128().unwrap() >= *&MIN_DEPOSIT,
            "For creation task you need pay minimum 3 Near"
        );

        let account_id = env::predecessor_account_id();
        let account_balance: Balance = env::account_balance();

        let record = Record {
            task,
            is_complete_status: false,
            deadline_time,
            guarantee_of_task_completion: env::attached_deposit(),
            account_balance,
            deposit_status: DepositStatus::Contributed,
        };

        if self.common_records.get(&account_id).is_some() {
            let mut user_record = self.common_records.get(&account_id).unwrap();

            user_record
                .user_records
                .insert(&user_record.record_id, &record);
            user_record.record_id += 1;

            self.common_records.insert(&account_id, &user_record);
        }
        if self.common_records.get(&account_id).is_none() {
            let mut user_record = UserRecords::default();

            user_record
                .user_records
                .insert(&user_record.record_id, &record);
            user_record.record_id += 1;

            self.common_records.insert(&account_id, &user_record);
        }
    }

    /// The method allows to get the task by its order number
    pub fn get_task_by_id(&self, record_id: i64, user_id: AccountId) -> Record {
        assert!(
            &self.common_records.get(&user_id).is_some(),
            "User not found"
        );
        return self
            .common_records
            .get(&user_id)
            .unwrap()
            .user_records
            .get(&record_id)
            .unwrap();
    }

    /// The method allows to get all user tasks
    pub fn get_all_user_tasks(&self, user_id: AccountId) -> Vec<(i64, Record)> {
        assert!(
            &self.common_records.get(&user_id).is_some(),
            "User not found"
        );
        assert!(
            &self
                .common_records
                .get(&user_id)
                .unwrap()
                .user_records
                .get(&1)
                .is_some(),
            "You have not added tasks yet"
        );
        unordered_map_pagination(
            &self.common_records.get(&user_id).unwrap().user_records,
            None,
            None,
        )
    }

    /// The method allows you to complete scheduled tasks
    /// if the deadline for the task has not expired, the method will return the deposit to the user
    pub fn make_complete_task_status(&mut self, changed_record_id: i64) -> String {
        if self
            .common_records
            .get(&env::predecessor_account_id())
            .is_some()
        {
            let mut changed_user_records = self
                .common_records
                .get(&env::predecessor_account_id())
                .unwrap();

            if changed_user_records
                .user_records
                .get(&changed_record_id)
                .is_some()
            {
                let mut record = changed_user_records
                    .user_records
                    .get(&changed_record_id)
                    .unwrap();

                assert_eq!(record.is_complete_status, false, "Task already completed");

                record.is_complete_status = true;

                changed_user_records
                    .user_records
                    .insert(&changed_record_id, &record);

                self.common_records
                    .insert(&env::predecessor_account_id(), &changed_user_records);

                if record.deadline_time > env::block_timestamp() {
                    Promise::new(env::predecessor_account_id())
                        .transfer(record.guarantee_of_task_completion);
                    record.account_balance = env::account_balance();
                    record.deposit_status = DepositStatus::Refunded;
                    changed_user_records.user_records.insert(&changed_record_id, &record);

                    return String::from("Deposit refunded ".to_owned() + &*record.guarantee_of_task_completion.to_string());
                }
                record.deposit_status = DepositStatus::Withheld;
                changed_user_records.user_records.insert(&changed_record_id, &record);
            }

        }
        return String::from("Deadline was ended, deposit stayed in service");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::json_types::ValidAccountId;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env};
    use std::convert::TryFrom;

    const FIRST_TASK: i64 = 1;
    const COMPLETE_STATUS: bool = true;

    fn to_valid_account(account: &str) -> ValidAccountId {
        ValidAccountId::try_from(account.to_string()).expect("Invalid account")
    }

    fn get_context(predecessor: ValidAccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    fn create_record(
        &deposit: &u128,
        &status: &bool,
        &deadline: &Timestamp,
    ) -> Record {
        let record = Record {
            task: "default task".to_string(),
            is_complete_status: status,
            deadline_time: deadline,
            guarantee_of_task_completion: deposit,
            account_balance: env::account_balance(),
            deposit_status: DepositStatus::Contributed,
        };
        return record;
    }

    #[test]
    fn check_creation_of_task() {
        let context = get_context(to_valid_account("lrn.testnet"));
        testing_env!(context.build());
        let account = context.build().predecessor_account_id;
        let mut received_contract = Contract {
            common_records: LookupMap::new(StorageKey::CommonRecords),
        };

        received_contract.create_task("default task".to_string(),  1658179621);

        let first_record = create_record(&3000000000000000000000000, &false, &1658179621);

        let received_task = received_contract
            .common_records
            .get(&account)
            .unwrap()
            .user_records.get(&1).unwrap().task;

        let received_deposit = received_contract
            .common_records
            .get(&account)
            .unwrap()
            .user_records.get(&1).unwrap().guarantee_of_task_completion;

        let received_status = received_contract
            .common_records
            .get(&account)
            .unwrap()
            .user_records.get(&1).unwrap().is_complete_status;

        let received_deadline_time = received_contract
            .common_records
            .get(&account)
            .unwrap()
            .user_records.get(&1).unwrap().deadline_time;

        assert_eq!(first_record.task, received_task);
        assert_eq!(first_record.guarantee_of_task_completion, received_deposit);
        assert_eq!(first_record.is_complete_status, received_status);
        assert_eq!(first_record.deadline_time, received_deadline_time);
    }

    #[test]
    #[should_panic]
    fn check_min_deposit_for_creation_task() {
        let context = get_context(to_valid_account("lrn.testnet"));
        testing_env!(context.build());
        let mut received_contract = Contract {
            common_records: LookupMap::new(StorageKey::CommonRecords),
        };

        received_contract.create_task("default task".to_string(), 1658179621);
    }

    #[test]
    fn check_getting_of_all_records() {
        let context = get_context(to_valid_account("lrn.testnet"));
        testing_env!(context.build());
        let account = context.build().predecessor_account_id;
        let mut received_contract = Contract {
            common_records: LookupMap::new(StorageKey::CommonRecords),
        };

        received_contract.create_task("default task".to_string(), 1658179621);
        received_contract.create_task("default task".to_string(), 1658179622);

        let vec = received_contract.get_all_user_tasks(account);

        assert_eq!(vec.len(), 2);
    }

    #[test]
    #[should_panic]
    fn check_getting_all_task_without_created_tasks() {
        let context = get_context(to_valid_account("lrn.testnet"));
        testing_env!(context.build());
        let account = context.build().predecessor_account_id;
        let received_contract = Contract {
            common_records: LookupMap::new(StorageKey::CommonRecords),
        };
        received_contract.get_all_user_tasks(account);
    }

    #[test]
    fn check_changing_status_of_task() {
        let mut context = get_context(to_valid_account("lrn.testnet"));
        testing_env!(context.build());
        let account = context.build().predecessor_account_id;
        let mut received_contract = Contract {
            common_records: LookupMap::new(StorageKey::CommonRecords),
        };
        let attached_dep: Balance = 3000000000000000000000000;
        context.build().attached_deposit = attached_dep;

        received_contract.create_task("default task".to_string(), 1658179621);
        received_contract.make_complete_task_status(FIRST_TASK);
        let received_status = received_contract.common_records.get(&account).unwrap().user_records.get(&FIRST_TASK).unwrap().is_complete_status;
        assert_eq!(received_status, COMPLETE_STATUS);
    }

    #[test]
    #[should_panic]
    fn check_panic_when_trying_complete_completed_task() {
        let context = get_context(to_valid_account("lrn.testnet"));
        testing_env!(context.build());
        let mut received_contract = Contract {
            common_records: LookupMap::new(StorageKey::CommonRecords),
        };

        received_contract.create_task("default task".to_string(), 1658179621);
        received_contract.make_complete_task_status(FIRST_TASK);
        received_contract.make_complete_task_status(FIRST_TASK);
    }
}
