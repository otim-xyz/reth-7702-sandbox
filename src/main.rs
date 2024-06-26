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
use reth_primitives::{b256, ChainSpec, Genesis};
use std::sync::Arc;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Address A
    // e15ae99ce6d60a47c81e6500bc4ca643d670c0ba591f7539df72229feb0cee07
    // 0x81B6e2Aa3AF93E5e3299d692b6e9a9957ED1d724

    // Address B
    // ab0dd18b1e1db01dc7af7042bdc8c97c59245d5cde13c5b87096893ac00318c8
    // 0x2d0FFF0846D9e9660103Dd1a176e133D1b246553

    // Signed by address A
    let tx_7702 = "0x04f8b2820a288084163ef00185081527974c82f6f594dac17f958d2ee523a2206206994597c13d831ec7b844a9059cbb0000000000000000000000005a96834046c1dff63119eb0eed6330fc5007a1d700000000000000000000000000000000000000000000000000000001a1432720c0c080a0f1239b70d8d60e1470337164c5851727f4569725c90a9e26120ae74d3071e98da041254f877871353c35b4191eca4b0486531ae17f3b7d231e4f00225bacb718e4";

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

    let eth_api = node.rpc_registry.eth_api();

    let hash = eth_api
        .send_raw_transaction(
            hex::decode(tx_7702.trim_start_matches("0x"))
                .unwrap()
                .into(),
        )
        .await?;

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
        "0x81B6e2Aa3AF93E5e3299d692b6e9a9957ED1d724": {
            "balance": "0x4a47e3c12448f4ad000000"
        },
        "0x2d0FFF0846D9e9660103Dd1a176e133D1b246553": {
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
