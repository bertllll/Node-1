// Copyright (c) 2019, MASQ (https://masq.ai) and/or its affiliates. All rights reserved.

use multinode_integration_tests_lib::masq_node::MASQNode;
use multinode_integration_tests_lib::masq_node::MASQNodeUtils;
use multinode_integration_tests_lib::masq_node_cluster::MASQNodeCluster;
use multinode_integration_tests_lib::masq_real_node::{ConsumingWalletInfo, NodeStartupConfig, NodeStartupConfigBuilder};
use regex::escape;
use std::time::Duration;
use masq_lib::utils::find_free_port;
use multinode_integration_tests_lib::mock_blockchain_client_server::MBCSBuilder;

#[test]
fn debtors_are_credited_once_but_not_twice() {
    let mbcs_port = find_free_port();
    // Create and initialize mock blockchain client: prepare a receivable at block 2000
    let blockchain_client = MBCSBuilder::new (mbcs_port)
        .response (
            vec![
                LogObject {
                    removed: false,
                    log_index: Some("0x20".to_string()),
                    transaction_index: Some("0x30".to_string()),
                    transaction_hash: Some("0x2222222222222222222222222222222222222222222222222222222222222222".to_string()),
                    block_hash: Some("0x1111111111111111111111111111111111111111111111111111111111111111".to_string()),
                    block_number: Some("0x2000".to_string()),
                    address: "0x3333333333333333333333333333333333333333".to_string(),
                    data: "0x000000000000000000000000000000000000000000000000000000003b5dc100".to_string(),
                    topics: vec![
                        "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".to_string(),
                    ]
                }
            ],
            1
        )
        .start();
    // Start a real Node pointing at the mock blockchain client with a start block of 1000
    let mut cluster = MASQNodeCluster::start().unwrap();
    let node = cluster.start_real_node (NodeStartupConfigBuilder::standard().start_block (0x1000));
    // Get the config DAO
    // Get the receivable DAO
    // Create a receivable record to match the client receivable
    // Wait for a scan log
    // Kill the real Node
    // Use the receivable DAO to verify that the receivable's balance has been adjusted
    // Use the config DAO to verify that the start block has been advanced to 2001
    todo!("Complete me");
}

#[test]
fn blockchain_bridge_logs_when_started() {
    let mut cluster = MASQNodeCluster::start().unwrap();
    let private_key = "0011223300112233001122330011223300112233001122330011223300112233";
    let subject = cluster.start_real_node(
        NodeStartupConfigBuilder::zero_hop()
            .consuming_wallet_info(ConsumingWalletInfo::PrivateKey(private_key.to_string()))
            .chain(cluster.chain)
            .build(),
    );

    let escaped_pattern = escape(&format!(
        "DEBUG: BlockchainBridge: Received BindMessage; consuming wallet address {}",
        subject.consuming_wallet().unwrap()
    ));
    MASQNodeUtils::wrote_log_containing(
        subject.name(),
        &escaped_pattern,
        Duration::from_millis(1000),
    )
}

#[derive (Serialize)]
struct LogObject { // Strings are all hexadecimal
    removed: bool,
    #[serde(rename = "logIndex")]
    log_index: Option<String>,
    #[serde(rename = "transactionIndex")]
    transaction_index: Option<String>,
    #[serde(rename = "transactionHash")]
    transaction_hash: Option<String>,
    #[serde(rename = "blockHash")]
    block_hash: Option<String>,
    #[serde(rename = "blockNumber")]
    block_number: Option<String>,
    address: String,
    data: String,
    topics: Vec<String>,
}