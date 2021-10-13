// Copyright (c) 2019-2021, MASQ (https://masq.ai) and/or its affiliates. All rights reserved.

use crate::blockchain::blockchain_interface::{
    MAINNET_CONTRACT_ADDRESS, MULTINODE_TESTNET_CONTRACT_ADDRESS, RINKEBY_TESTNET_CONTRACT_ADDRESS,
    ROPSTEN_TESTNET_CONTRACT_ADDRESS,
};
use itertools::Itertools;
use masq_lib::constants::{
    DEV_LABEL, ETH_MAINNET_LABEL, ETH_RINKEBY_LABEL, ETH_ROPSTEN_LABEL,
    MAINNET_CONTRACT_CREATION_BLOCK, MULTINODE_TESTNET_CONTRACT_CREATION_BLOCK,
    RINKEBY_TESTNET_CONTRACT_CREATION_BLOCK, ROPSTEN_TESTNET_CONTRACT_CREATION_BLOCK,
};
use serde_derive::{Deserialize, Serialize};
use std::fmt::Debug;
use web3::types::Address;
pub const DEFAULT_CHAIN: Chain = Chain::EthMainnet;

pub const CENTRAL_DELIMITER: char = '@';
pub const CHAIN_LABEL_DELIMITER: char = ':';

pub const CHAINS: [BlockchainRecord; 4] = [
    BlockchainRecord {
        literal_chain_id: Chain::EthMainnet,
        num_chain_id: 1,
        plain_text_name: "eth-mainnet",
        directory_by_platform: "eth",
        chain_label: ETH_MAINNET_LABEL,
        contract: MAINNET_CONTRACT_ADDRESS,
        contract_creation_block: MAINNET_CONTRACT_CREATION_BLOCK,
    },
    BlockchainRecord {
        literal_chain_id: Chain::EthRopsten,
        num_chain_id: 3,
        plain_text_name: "ropsten",
        directory_by_platform: "eth",
        chain_label: ETH_ROPSTEN_LABEL,
        contract: ROPSTEN_TESTNET_CONTRACT_ADDRESS,
        contract_creation_block: ROPSTEN_TESTNET_CONTRACT_CREATION_BLOCK,
    },
    BlockchainRecord {
        literal_chain_id: Chain::EthRinkeby,
        num_chain_id: 4,
        plain_text_name: "rinkeby",
        directory_by_platform: "eth",
        chain_label: ETH_RINKEBY_LABEL,
        contract: RINKEBY_TESTNET_CONTRACT_ADDRESS,
        contract_creation_block: RINKEBY_TESTNET_CONTRACT_CREATION_BLOCK,
    },
    BlockchainRecord {
        literal_chain_id: Chain::Dev,
        num_chain_id: 5,
        plain_text_name: "dev",
        directory_by_platform: "dev",
        chain_label: DEV_LABEL,
        contract: MULTINODE_TESTNET_CONTRACT_ADDRESS,
        contract_creation_block: MULTINODE_TESTNET_CONTRACT_CREATION_BLOCK,
    },
];

#[derive(Debug, PartialEq)]
pub struct BlockchainRecord {
    pub literal_chain_id: Chain,
    pub num_chain_id: u8,
    pub plain_text_name: &'static str,
    pub directory_by_platform: &'static str,
    pub chain_label: &'static str,
    pub contract: Address,
    pub contract_creation_block: u64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Chain {
    EthMainnet,
    EthRopsten,
    EthRinkeby,
    Dev,
}

impl Chain {
    pub fn record(&self) -> &BlockchainRecord {
        CHAINS
            .iter()
            .find(|b| &b.literal_chain_id == self)
            .unwrap_or_else(|| panic!("BlockchainRecord for '{:?}' doesn't exist", self))
        //untested panic - more secure in general if I don't define an extra Chain to test it
    }
    pub fn from_id(id: u8) -> Chain {
        match id {
            1 => Self::EthMainnet,
            3 => Self::EthRopsten,
            4 => Self::EthRinkeby,
            5 => Self::Dev,
            x => panic!("Undefined num id '{}' for ChainRecords", x),
        }
    }
}

impl From<&str> for Chain {
    fn from(str: &str) -> Self {
        match str {
            "eth-mainnet" => Chain::EthMainnet,
            "ropsten" => Chain::EthRopsten,
            "rinkeby" => Chain::EthRinkeby,
            "dev" => Chain::Dev,
            x => panic!("Undefined chain name '{}' was used", x), //TODO decide testing of this
        }
    }
}

pub fn blockchain_from_label_opt(label: &str) -> Option<Chain> {
    return_record_opt_standard_impl(Box::new(|b: &&BlockchainRecord| b.chain_label == label))
        .map(|record| record.literal_chain_id)
}

fn return_record_opt_standard_impl<'a, F>(closure: Box<F>) -> Option<&'a BlockchainRecord>
where
    F: FnMut(&&BlockchainRecord) -> bool,
{
    return_record_opt_body(closure, &CHAINS)
}

fn return_record_opt_body<F>(
    closure: Box<F>,
    collection_of_chains: &[BlockchainRecord],
) -> Option<&BlockchainRecord>
where
    F: FnMut(&&BlockchainRecord) -> bool,
{
    let mut filtered = collection_of_chains.iter().filter(closure).collect_vec();
    filtered.pop().map(|first| {
        if filtered.pop() != None {
            panic!("Not unique identifier used to query a BlockchainRecord")
        } else {
            first
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_returns_an_appropriate_blockchain_record() {
        assert_eq!(Chain::EthMainnet.record().num_chain_id, 1);
        assert_eq!(Chain::EthRopsten.record().num_chain_id, 3);
        assert_eq!(Chain::EthRinkeby.record().num_chain_id, 4);
        assert_eq!(Chain::Dev.record().num_chain_id, 5);
    }

    #[test]
    fn from_id_works() {
        assert_eq!(Chain::from_id(1), Chain::EthMainnet);
        assert_eq!(Chain::from_id(3), Chain::EthRopsten);
        assert_eq!(Chain::from_id(4), Chain::EthRinkeby);
        assert_eq!(Chain::from_id(5), Chain::Dev)
    }

    #[test]
    #[should_panic(expected = "Undefined num id '2' for ChainRecords")]
    fn from_id_panics_on_undefined_ids() {
        (1u8..).for_each(|num| {
            if find_record(Box::new(|record: &&BlockchainRecord| {
                record.num_chain_id == num
            }))
            .is_none()
            {
                Chain::from_id(num);
            }
        })
    }

    fn find_record<F>(closure: Box<F>) -> Option<&'static BlockchainRecord>
    where
        F: FnMut(&&BlockchainRecord) -> bool,
    {
        CHAINS.iter().find(closure)
    }

    #[test]
    fn from_string_works() {
        let first = Chain::EthMainnet;
        assert_eq!(Chain::from(first.record().plain_text_name), first);
        let second = Chain::EthRopsten;
        assert_eq!(Chain::from(second.record().plain_text_name), second);
        let third = Chain::EthRinkeby;
        assert_eq!(Chain::from(third.record().plain_text_name), third);
        let fourth = Chain::Dev;
        assert_eq!(Chain::from(fourth.record().plain_text_name), fourth);
    }

    #[test]
    #[should_panic(expected = "Not unique identifier used to query a BlockchainRecord")]
    fn return_record_opt_panics_if_more_records_meet_the_condition_from_the_closure() {
        let searched_name = "BruhBruh";
        let mut record_one = make_defaulted_blockchain_record();
        record_one.plain_text_name = searched_name;
        let mut record_two = make_defaulted_blockchain_record();
        record_two.plain_text_name = "Jooodooo";
        let mut record_three = make_defaulted_blockchain_record();
        record_three.plain_text_name = searched_name;
        let collection = [record_one, record_two, record_three];

        let _ = return_record_opt_body(
            Box::new(|b: &&BlockchainRecord| b.plain_text_name == searched_name),
            &collection,
        );
    }

    #[test]
    fn return_record_opt_standard_impl_uses_the_true_collection_of_chains() {
        CHAINS.iter().for_each(|record| {
            assert_eq!(
                record,
                return_record_opt_standard_impl(Box::new(
                    |b: &&BlockchainRecord| b.num_chain_id == record.num_chain_id
                ))
                .unwrap()
            )
        });
    }

    #[test]
    fn eth_mainnet_record_is_properly_defined() {
        let examined_chain = return_examined(Chain::EthMainnet);
        assert_eq!(
            examined_chain,
            &BlockchainRecord {
                num_chain_id: 1,
                literal_chain_id: Chain::EthMainnet,
                plain_text_name: "eth-mainnet",
                directory_by_platform: "eth",
                chain_label: "eth",
                contract: MAINNET_CONTRACT_ADDRESS,
                contract_creation_block: MAINNET_CONTRACT_CREATION_BLOCK,
            }
        )
    }

    #[test]
    fn ropsten_record_is_properly_defined() {
        let examined_chain = return_examined(Chain::EthRopsten);
        assert_eq!(
            examined_chain,
            &BlockchainRecord {
                num_chain_id: 3,
                literal_chain_id: Chain::EthRopsten,
                plain_text_name: "ropsten",
                directory_by_platform: "eth",
                chain_label: "eth_t1",
                contract: ROPSTEN_TESTNET_CONTRACT_ADDRESS,
                contract_creation_block: ROPSTEN_TESTNET_CONTRACT_CREATION_BLOCK,
            }
        )
    }

    #[test]
    fn rinkeby_record_is_properly_defined() {
        let examined_chain = return_examined(Chain::EthRinkeby);
        assert_eq!(
            examined_chain,
            &BlockchainRecord {
                num_chain_id: 4,
                literal_chain_id: Chain::EthRinkeby,
                plain_text_name: "rinkeby",
                directory_by_platform: "eth",
                chain_label: "eth_t2",
                contract: RINKEBY_TESTNET_CONTRACT_ADDRESS,
                contract_creation_block: RINKEBY_TESTNET_CONTRACT_CREATION_BLOCK
            }
        )
    }

    #[test]
    fn multinode_testnet_chain_record_is_properly_defined() {
        let examined_chain = return_examined(Chain::Dev);
        assert_eq!(
            examined_chain,
            &BlockchainRecord {
                num_chain_id: 5,
                literal_chain_id: Chain::Dev,
                plain_text_name: "dev",
                directory_by_platform: "dev",
                chain_label: "dev",
                contract: MULTINODE_TESTNET_CONTRACT_ADDRESS,
                contract_creation_block: 0
            }
        )
    }

    fn return_examined<'a>(blockchain: Chain) -> &'a BlockchainRecord {
        CHAINS
            .iter()
            .find(|chain| chain.literal_chain_id == blockchain)
            .unwrap()
    }

    #[test]
    fn blockchain_from_label_opt_works() {
        assert_blockchain_from_label_opt("eth", Some(Chain::EthMainnet));
        assert_blockchain_from_label_opt("eth_t1", Some(Chain::EthRopsten));
        assert_blockchain_from_label_opt("eth_t2", Some(Chain::EthRinkeby));
        assert_blockchain_from_label_opt("dev", Some(Chain::Dev));
        assert_blockchain_from_label_opt("bitcoin", None);
        assert_eq!(CHAINS.len(), 4)
    }

    fn assert_blockchain_from_label_opt(label: &str, expected_blockchain: Option<Chain>) {
        assert_eq!(blockchain_from_label_opt(label), expected_blockchain)
    }

    fn make_defaulted_blockchain_record() -> BlockchainRecord {
        BlockchainRecord {
            num_chain_id: 0,
            literal_chain_id: Chain::EthMainnet,
            plain_text_name: "",
            directory_by_platform: "",
            chain_label: "",
            contract: Default::default(),
            contract_creation_block: 0,
        }
    }
}
