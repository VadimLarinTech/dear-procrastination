use near_units::parse_near;
use workspaces::prelude::*;
use serde_json::json;
use workspaces::{Account, Contract, Worker, network::Sandbox};
use near_sdk::{Timestamp};

const WASM_FILEPATH: &str = "../../out/main.wasm";
const TIMESTAMP_IN_FUTURE_MS: Timestamp = 1693818395000;
const TIMESTAMP_IN_PAST_MS: Timestamp = 1630746395000;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm = std::fs::read(WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    // create accounts
    let owner = worker.root_account();

    let alice = owner
        .create_subaccount(&worker, "alice")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;

    // begin tests
    test_create_task(&alice, &contract, &worker).await?;
    test_getting_tasks(&alice, &contract, &worker).await?;
    test_completing_tasks(&alice, &contract, &worker).await?;
    Ok(())
}


async fn test_create_task(
    alice: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let started_balance = alice
        .view_account(&worker)
        .await?
        .balance;

    alice.call(&worker, &contract.id(), "create_task")
        .deposit(parse_near!("3 N"))
        .args_json(json!({"task": "First task", "deadline_time": 1693818395}))?
        .transact()
        .await?;

    alice.call(&worker, &contract.id(), "create_task")
        .deposit(parse_near!("4 N"))
        .args_json(json!({"task": "Second task", "deadline_time": 1630746395}))?
        .transact()
        .await?;

    let balance_after_create_two_tasks = alice.view_account(&worker)
        .await?
        .balance;

    assert_eq!(started_balance, balance_after_create_two_tasks - parse_near!("7 N"));

    println!("      Passed ✅ create 2 tasks");
    Ok(())
}

async fn test_getting_tasks(
    alice: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let first_task: serde_json::Value = alice.call(&worker, contract.id(), "get_task_by_id")
        .args_json(json!({"record_id": 1, "user_id": alice.id()}))?
        .transact()
        .await?
        .json()?;

    let second_task: serde_json::Value = alice.call(&worker, contract.id(), "get_task_by_id")
        .args_json(json!({"record_id": 2, "user_id": alice.id()}))?
        .transact()
        .await?
        .json()?;

    let first_task_expected = json!(
        {
            "task": "First task",
            "is_complete_status": false,
            "guarantee_of_task_completion": 3e+24,
            "deadline_time": 1693818395,
            "account_balance": 33e+26,
            "deposit_status": "Contributed"
        }
    );

    let second_task_expected = json!(
        {
            "task": "Second task",
            "is_complete_status": false,
            "guarantee_of_task_completion": 3e+24,
            "deadline_time": 1630746395,
            "account_balance": 37e+26,
            "deposit_status": "Contributed"
        }
    );

    assert_eq!(first_task, first_task_expected);
    assert_eq!(second_task, second_task_expected);

    println!("      Passed ✅ getting 2 tasks");
    Ok(())
}

async fn test_completing_tasks(
    alice: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let complete_first_task: serde_json::Value = alice.call(&worker, contract.id(), "make_complete_task_status")
        .args_json(json!({"changed_record_id": 1}))?
        .transact()
        .await?
        .json()?;

    let complete_second_task: serde_json::Value = alice.call(&worker, contract.id(), "make_complete_task_status")
        .args_json(json!({"changed_record_id": 2}))?
        .transact()
        .await?
        .json()?;

    let complete_first_task_expected = json!(
        {
            "task": "First task",
            "is_complete_status": true,
            "guarantee_of_task_completion": 3e+24,
            "deadline_time": 1630746395,
            "account_balance": 33e+26,
            "deposit_status": "Refunded"
        }
    );

    let complete_second_task_expected = json!(
        {
            "task": "Second task",
            "is_complete_status": true,
            "guarantee_of_task_completion": 3e+24,
            "deadline_time": 1693818395,
            "account_balance": 33e+26,
            "deposit_status": "Withheld"
        }
    );

    assert_eq!(complete_first_task, complete_first_task_expected);
    assert_eq!(complete_second_task, complete_second_task_expected);

    println!("      Passed ✅ completing 2 tasks");
    Ok(())
}
