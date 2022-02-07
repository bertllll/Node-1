use std::str::FromStr;
use futures::Future;
use web3;
use web3::contract::Options;
use ethabi;
use ethabi::Address;
use masq_lib::blockchains::chains::Chain;
use node_lib::sub_lib::wallet::Wallet;


fn assert_contract(blockchain_url: &str, chain: &Chain, expected_token_name: &str, expected_decimals:u32) {
    let (_event_loop, transport) = web3::transports::Http::new(blockchain_url).unwrap();
    let web3 = web3::Web3::new(transport);
    let address = chain.rec().contract;
    let min_abi_json = r#"[{
        "constant": true,
        "inputs": [],
        "name": "name",
        "outputs": [
            {
                "name": "",
                "type": "string"
            }
        ],
        "payable": false,
        "stateMutability": "view",
        "type": "function"
    },
        {
        "constant": true,
        "inputs": [],
        "name": "decimals",
        "outputs": [
            {
                "name": "",
                "type": "uint8"
            }
        ],
        "payable": false,
        "stateMutability": "view",
        "type": "function"
    }]"#;
    let abi = ethabi::Contract::load(min_abi_json.as_bytes()).unwrap();
    let contract = web3::contract::Contract::new(web3.eth(), address, abi);

    let token_name: String = contract
        .query("name", (), None, Options::default(), None)
        .wait()
        .unwrap();

    let decimals: u32 = contract
        .query("decimals", (), None, Options::default(), None)
        .wait()
        .unwrap();

    assert_eq!(token_name, expected_token_name);
    assert_eq!(decimals, expected_decimals);
}

#[test]
fn exists_on_polygon_mumbai() {
    let blockchain_url = "https://polygon-mumbai.g.alchemy.com/v2/wFOdk2UWjB8SqeZvzsiwmYX3iBEOq3UC";
    let chain = Chain::PolyMumbai;

    assert_contract(blockchain_url, &chain, "tMASQ", 18)
}