use anyhow::Ok;

#[allow(unused_imports)]
use near_sdk::{json_types::U128, AccountId, Balance};
use workspaces::{Contract, prelude::DevAccountDeployer, Worker, network::Sandbox, Account};
use near_units::parse_near;
#[allow(unused_imports)]
use serde_json::{json, Value};
use near_sdk::serde::{Deserialize, Serialize};

pub type CommandId = String;
pub type NameProduct = String;

const CONTRACT_PATH: &str = "./wasm/contract.wasm";

#[derive( Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]

pub struct Quality {
    pub certificate: Vec<AccountId>,
    pub stage: Vec<AccountId>,
}

#[derive( Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CommandDetail {
    pub command_id: CommandId,
    pub name_product: NameProduct,
    pub is_sell: bool,
    pub amount_product: u128,
    pub price_per_product: Balance,
    pub quality: Option<Quality>,
    pub command_owner_id: AccountId,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract_wasm = std::fs::read(CONTRACT_PATH)?;
    let contract: Contract = worker.dev_deploy(&contract_wasm).await?;

    let owner = worker.root_account().unwrap();

    let commander = owner.create_subaccount(&worker, "commander")
                                            .initial_balance(parse_near!("1000 N"))
                                            .transact()
                                            .await?
                                            .into_result()?;

    // Init contract
    contract
        .call(&worker, "new")
        .args_json(serde_json::json!({
            "owner_id": owner.id(),
        }))?
        .transact()
        .await?;

    // Begin test
    test_add_buy_command(&commander, &contract, &worker).await?;
    Ok(())
}

async fn test_add_buy_command(user: &Account, contract: &Contract, worker: &Worker<Sandbox>) -> anyhow::Result<()> {
    let deposit = parse_near!("70 N");
    // println!("deposit: {}",deposit);
    user.
        call(&worker, contract.id(), "add_command")
        .args_json(json!({
            "command_id": "command_1",
            "name_product": "Iphone_14",
            "is_sell": false,
            "amount_product": "2",
            "price_per_product": "30000000000000000000000000",
            "quality": null,
        }))?
        .deposit(deposit)
        .transact()
        .await?;

    println!("      Passed ✅  add_buy_command");

    user.
        call(&worker, contract.id(), "add_command")
        .args_json(json!({
            "command_id": "command_2",
            "name_product": "Iphone_14",
            "is_sell": false,
            "amount_product": "1",
            "price_per_product": "31000000000000000000000000",
            "quality": null,
        }))?
        .deposit(deposit)
        .transact()
        .await?;

    let res_command: CommandDetail = user.call(&worker, contract.id(), "get_command")
                                        .args_json(json!({
                                            "command_id": "command_1"
                                        }))?
                                        .transact()
                                        .await?
                                        .json()?;

    assert_eq!(res_command.command_id, "command_1", "WRONG_COMMAND_ID_1");
    assert_eq!(res_command.name_product.to_string(), "Iphone_14", "WRONG_NAME_PRODUCT_1");
    assert_eq!(res_command.is_sell, false, "WRONG_IS_SELL_1");
    assert_eq!(res_command.amount_product, 2, "WRONG_AMOUNT_PRODUCT_1");
    assert_eq!(res_command.price_per_product, 30000000000000000000000000, "WRONG_PRICE_PER_PRODUCT_1");

    println!("      Passed ✅  get_command");

    let res_vec: Vec<CommandDetail> = user.call(&worker, contract.id(), "get_product_order_way")
                    .args_json(json!({
                        "name_product": "Iphone_14",
                        "is_sell": false,
                    }))?
                    .transact()
                    .await?
                    .json()?;

    assert_eq!(res_vec.get(0).unwrap().command_id, "command_2", "WRONG_COMMAND_ID_2");
    assert_eq!(res_vec.get(0).unwrap().amount_product, 1, "WRONG_AMOUNT_PRODUCT_2");
    assert_eq!(res_vec.get(0).unwrap().price_per_product, 31000000000000000000000000, "WRONG_PRICE_PER_PRODUCT_2");

    assert_eq!(res_vec.get(1).unwrap().command_id, "command_1", "WRONG_COMMAND_ID_3");
    assert_eq!(res_vec.get(1).unwrap().amount_product, 2, "WRONG_AMOUNT_PRODUCT_3");
    assert_eq!(res_vec.get(1).unwrap().price_per_product, 30000000000000000000000000, "WRONG_PRICE_PER_PRODUCT_3");
    
    println!("      Passed ✅  get_product_order_way");

    Ok(())
}
