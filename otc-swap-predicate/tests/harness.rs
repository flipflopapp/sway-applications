mod utils;

use fuel_core::service::Config;
use fuel_gql_client::fuel_vm::{consts::REG_ONE, prelude::Opcode};
use fuels::contract::script::Script;
use fuels::prelude::*;
use fuels::test_helpers::WalletsConfig;
use fuels::tx::{AssetId, Contract, Input, Output, Transaction};

/// Test function to be parameterized by test cases
async fn otc_swap_with_predicate(ask_amount: u64, ask_token: AssetId, receiver_address: Address) {
    // Set up a wallet and send some base asset to the predicate root
    let base_asset: AssetId = Default::default();

    let mut provider_config = Config::local_node();
    provider_config.predicates = true; // predicates are currently disabled by default
    let wallet = &launch_custom_provider_and_get_wallets(
        WalletsConfig::new_single(None, None),
        Some(provider_config),
    )
    .await[0];

    // Get provider and client
    let provider = wallet.get_provider().unwrap();
    let client = &provider.client;

    // Get predicate bytecode and root
    let predicate_bytecode =
        std::fs::read("../otc-swap-predicate/out/debug/otc-swap-predicate.bin").unwrap();
    let predicate_root: [u8; 32] = (*Contract::root_from_code(&predicate_bytecode)).into();
    let predicate_root = Address::from(predicate_root);

    // Transfer some coins to the predicate root
    let offered_amount = 1000;
    let _receipt = wallet
        .transfer(
            &predicate_root,
            offered_amount,
            base_asset,
            TxParameters::default(),
        )
        .await
        .unwrap();

    let initial_predicate_balance = utils::get_balance(&provider, predicate_root, base_asset).await;
    let initial_wallet_balance = utils::get_balance(&provider, wallet.address(), base_asset).await;
    let initial_receiver_balance = utils::get_balance(&provider, receiver_address, ask_token).await;

    // The predicate root has received the coin
    assert_eq!(initial_predicate_balance, offered_amount);

    // Get predicate coin to unlock
    let predicate_coin = &provider.get_coins(&predicate_root).await.unwrap()[0];
    let predicate_coin_utxo_id = predicate_coin.utxo_id.clone().into();

    // Get other coin to spend
    let swap_coin = &provider.get_coins(&wallet.address()).await.unwrap()[0];
    let swap_coin_utxo_id = swap_coin.utxo_id.clone().into();
    let swap_coin_amount: u64 = swap_coin.amount.clone().into();

    // Configure inputs and outputs to send coins from the predicate root to another address

    // The predicate allows to spend its tokens if `ask_amount` is sent to the offer maker.

    // Coin belonging to the predicate root
    let input_predicate = Input::CoinPredicate {
        utxo_id: predicate_coin_utxo_id,
        owner: predicate_root,
        amount: offered_amount,
        asset_id: base_asset,
        maturity: 0,
        predicate: predicate_bytecode,
        predicate_data: vec![1u8, 0u8], // Predicate data is the index of the input and output that pay the receiver
    };

    // Coin belonging to the wallet taking the order
    let input_from_taker = Input::CoinSigned {
        utxo_id: swap_coin_utxo_id,
        owner: wallet.address(),
        amount: swap_coin_amount,
        asset_id: ask_token,
        witness_index: 0,
        maturity: 0,
    };

    // Output for the coin transferred to the receiver
    let output_to_receiver = Output::Coin {
        to: receiver_address,
        amount: ask_amount,
        asset_id: ask_token,
    };

    // Output for the coin transferred to the order taker
    let output_to_taker = Output::Coin {
        to: wallet.address(),
        amount: offered_amount,
        asset_id: base_asset,
    };

    // Change output for unspent fees
    let output_change = Output::Change {
        to: wallet.address(),
        amount: 0,
        asset_id: Default::default(),
    };

    let mut tx = Transaction::Script {
        gas_price: 0,
        gas_limit: 10_000_000,
        maturity: 0,
        byte_price: 0,
        receipts_root: Default::default(),
        script: Opcode::RET(REG_ONE).to_bytes().to_vec(),
        script_data: vec![],
        inputs: vec![input_predicate, input_from_taker],
        outputs: vec![output_to_receiver, output_to_taker, output_change],
        witnesses: vec![],
        metadata: None,
    };

    // Sign and execute the transaction
    wallet.sign_transaction(&mut tx).await.unwrap();
    let script = Script::new(tx);
    let _receipts = script.call(&client).await.unwrap();

    let predicate_balance = utils::get_balance(&provider, predicate_root, base_asset).await;
    let wallet_balance = utils::get_balance(&provider, wallet.address(), base_asset).await;
    let receiver_balance = utils::get_balance(&provider, receiver_address, base_asset).await;

    // The predicate root's coin has been spent
    assert_eq!(predicate_balance, 0);

    // Receiver has been paid `ask_amount`
    assert_eq!(receiver_balance, initial_receiver_balance + ask_amount);

    // Taker has sent `ask_amount` tokens and received `offered_amount` in return
    assert_eq!(
        wallet_balance,
        initial_wallet_balance - ask_amount + offered_amount
    );
}

// Test cases

// These constants should match those hard-coded in the predicate
const CORRECT_ASK_AMOUNT: u64 = 42;
const CORRECT_ASK_TOKEN: AssetId = AssetId::new([0u8; 32]);
const CORRECT_RECEIVER_ADDRESS: Address = Address::new([3u8; 32]);

#[tokio::test]
async fn valid_predicate_spend() {
    otc_swap_with_predicate(
        CORRECT_ASK_AMOUNT,
        CORRECT_ASK_TOKEN,
        CORRECT_RECEIVER_ADDRESS,
    )
    .await;
}

#[tokio::test]
#[should_panic]
async fn incorrect_ask_amount() {
    otc_swap_with_predicate(41, CORRECT_ASK_TOKEN, CORRECT_RECEIVER_ADDRESS).await;
}

#[tokio::test]
#[should_panic]
async fn incorrect_ask_token() {
    otc_swap_with_predicate(
        CORRECT_ASK_AMOUNT,
        AssetId::new([1u8; 32]),
        CORRECT_RECEIVER_ADDRESS,
    )
    .await;
}

#[tokio::test]
#[should_panic]
async fn incorrect_receiver_address() {
    otc_swap_with_predicate(
        CORRECT_ASK_AMOUNT,
        CORRECT_ASK_TOKEN,
        Address::new([2u8; 32]),
    )
    .await;
}
