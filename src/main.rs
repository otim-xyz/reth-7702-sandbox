//! # reth-7702-sandbox

#![warn(
    missing_docs,
    unreachable_pub,
    non_ascii_idents,
    unreachable_pub,
    unused_crate_dependencies,
    unused_results,
    unused_qualifications,
    nonstandard_style,
    rustdoc::all
)]
#![deny(rust_2018_idioms, unsafe_code)]

mod eip7702;

use alloy_primitives::U160;
use eip7702::TxEip7702;
use futures_util::StreamExt;
use reth::{
    builder::{NodeBuilder, NodeHandle},
    primitives::IntoRecoveredTransaction,
    providers::CanonStateSubscriptions,
    rpc::{
        compat::transaction::transaction_to_call_request,
        eth::EthTransactions,
        types::trace::{parity::TraceType, tracerequest::TraceCallRequest},
    },
    tasks::TaskManager,
    transaction_pool::TransactionPool,
};
use reth_node_core::{args::RpcServerArgs, node_config::NodeConfig};
use reth_node_ethereum::EthereumNode;
use reth_primitives::{b256, hex, ChainSpec, Genesis, U256};
use std::sync::Arc;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let k = "d42bf368dcc16cfa37bfec8f0529fd908a7049dfa45f651926b5b138450b8817";
    let _pk = "407935c2575e9a572a3ede4c772be0eb5cc557fc658f2cc4c7b56c694f1c8ed4a20338ce02c642531290909db39b52acda008f8783a7ac69d388d2faf5413563";
    let _address1 = "8ef4c3785d21f0e3c83e8518ce7fa70f0b00ba5b";
    let address2 = "6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b";

    let tasks = TaskManager::current();

    // create node config
    let node_config = NodeConfig::test()
        .dev()
        .with_rpc(RpcServerArgs::default().with_http())
        .with_chain(custom_chain());

    let NodeHandle {
        mut node,
        node_exit_future: _,
    } = NodeBuilder::new(node_config)
        .testing_node(tasks.executor())
        .node(EthereumNode::default())
        .launch()
        .await?;

    let mut pending_transactions =
        node.pool.new_pending_pool_transactions_listener();

    let traceapi = node.rpc_registry.trace_api();

    _ = node.task_executor.spawn(Box::pin(async move {
        while let Some(event) = pending_transactions.next().await {
            let tx = event.transaction;
            println!("transaction received: {tx:#?}");

            let callrequest =
                transaction_to_call_request(tx.to_recovered_transaction());
            let tracerequest = TraceCallRequest::new(callrequest)
                .with_trace_type(TraceType::Trace);
            if let Ok(trace_result) = traceapi.trace_call(tracerequest).await {
                let hash = tx.hash();
                println!(
                    "trace result for transaction {hash}: {trace_result:#?}"
                );
            }
        }
    }));

    let mut notifications = node.provider.canonical_state_stream();

    // 4 || rlp([chain_id, nonce, max_priority_fee_per_gas, max_fee_per_gas, gas_limit, destination, data, access_list, [[contract_code, y_parity, r, s], ...], signature_y_parity, signature_r, signature_s])
    let tx = TxEip7702 {
        chain_id: 1,
        nonce: 0,
        max_priority_fee_per_gas: 1_234,
        max_fee_per_gas: 4_567,
        gas_limit: 8_910,
        to: U160::from_be_bytes::<20>(
            hex::decode(address2).unwrap().try_into().unwrap(),
        ),
        amount: U256::from(1_000_000),
        data: 0,
        access_list: vec![],
        code_bundles: vec![],
    };
    let signed_tx = tx.sign(k);
    let encoded_tx = signed_tx.rlp_encode();

    let eth_api = node.rpc_registry.eth_api();

    let hash = eth_api.send_raw_transaction(encoded_tx.into()).await?;

    let expected = b256!(
        "b1c6512f4fc202c04355fbda66755e0e344b152e633010e8fd75ecec09b63398"
    );

    assert_eq!(hash, expected);
    println!("submitted transaction: {hash}");

    let head = notifications.next().await.unwrap();

    let tx = head.tip().transactions().next().unwrap();
    assert_eq!(tx.hash(), hash);
    println!("mined transaction: {hash}");
    Ok(())
}

fn custom_chain() -> Arc<ChainSpec> {
    let custom_genesis = r#"
{
    "nonce": "0x42",
    "timestamp": "0x0",
    "extraData": "0x5343",
    "gasLimit": "0x1388",
    "difficulty": "0x400000000",
    "mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "coinbase": "0x0000000000000000000000000000000000000000",
    "alloc": {
        "0x8ef4c3785d21f0e3c83e8518ce7fa70f0b00ba5b": {
            "balance": "0x4a47e3c12448f4ad000000"
        },
        "0x6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b": {
            "balance": "0x4a47e3c12448f4ad000000"
        }
    },
    "number": "0x0",
    "gasUsed": "0x0",
    "parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "config": {
        "ethash": {},
        "chainId": 2600,
        "homesteadBlock": 0,
        "eip150Block": 0,
        "eip155Block": 0,
        "eip158Block": 0,
        "byzantiumBlock": 0,
        "constantinopleBlock": 0,
        "petersburgBlock": 0,
        "istanbulBlock": 0,
        "berlinBlock": 0,
        "londonBlock": 0,
        "terminalTotalDifficulty": 0,
        "terminalTotalDifficultyPassed": true,
        "shanghaiTime": 0
    }
}
"#;
    let genesis: Genesis = serde_json::from_str(custom_genesis).unwrap();
    Arc::new(genesis.into())
}
