// Copyright (c) 2019, MASQ (https://masq.ai) and/or its affiliates. All rights reserved.
use crate::command::Command;
use crate::masq_mock_node::MASQMockNode;
use crate::masq_node::{MASQNode, MASQNodeUtils};
use crate::masq_real_node::MASQRealNode;
use crate::masq_real_node::NodeStartupConfig;
use masq_lib::blockchains::chains::Chain;
use masq_lib::test_utils::utils::TEST_DEFAULT_MULTINODE_CHAIN;
use node_lib::sub_lib::cryptde::PublicKey;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::net::{IpAddr, Ipv4Addr};

pub struct MASQNodeCluster {
    startup_configs: HashMap<(String, usize), NodeStartupConfig>,
    real_nodes: HashMap<String, MASQRealNode>,
    mock_nodes: HashMap<String, MASQMockNode>,
    host_node_parent_dir: Option<String>,
    next_index: usize,
    pub chain: Chain,
}

impl MASQNodeCluster {
    pub fn start() -> Result<MASQNodeCluster, String> {
        MASQNodeCluster::cleanup()?;
        MASQNodeCluster::create_network()?;
        let host_node_parent_dir = match env::var("HOST_NODE_PARENT_DIR") {
            Ok(ref hnpd) if !hnpd.is_empty() => Some(hnpd.clone()),
            _ => None,
        };
        if Self::is_in_jenkins() {
            MASQNodeCluster::interconnect_network()?;
        }
        Ok(MASQNodeCluster {
            startup_configs: HashMap::new(),
            real_nodes: HashMap::new(),
            mock_nodes: HashMap::new(),
            host_node_parent_dir,
            next_index: 1,
            chain: TEST_DEFAULT_MULTINODE_CHAIN,
        })
    }

    pub fn host_ip_addr() -> IpAddr {
        if Self::is_in_jenkins() {
            IpAddr::V4(Ipv4Addr::new(172, 18, 0, 2))
        } else {
            IpAddr::V4(Ipv4Addr::new(172, 18, 0, 1))
        }
    }

    pub fn next_index(&self) -> usize {
        self.next_index
    }

    pub fn prepare_real_node(&mut self, config: &NodeStartupConfig) -> (String, usize) {
        let index = self.startup_configs.len() + 1;
        let name = MASQRealNode::make_name(index);
        self.next_index = index + 1;
        self.startup_configs
            .insert((name.clone(), index), config.clone());
        MASQRealNode::prepare(&name);

        (name, index)
    }

    pub fn start_real_node(&mut self, config: NodeStartupConfig) -> MASQRealNode {
        let index = self.next_index;
        self.next_index += 1;
        let node = MASQRealNode::start(config, index, self.host_node_parent_dir.clone());
        let name = node.name().to_string();
        self.real_nodes.insert(name.clone(), node);
        self.real_nodes.get(&name).unwrap().clone()
    }

    pub fn start_named_real_node(
        &mut self,
        name: String,
        index: usize,
        config: NodeStartupConfig,
    ) -> MASQRealNode {
        MASQRealNode::start_prepared(name, config, index, self.host_node_parent_dir.clone())
    }

    pub fn start_mock_node_with_real_cryptde(&mut self, ports: Vec<u16>) -> MASQMockNode {
        self.start_mock_node(ports, None)
    }

    pub fn start_mock_node_with_public_key(
        &mut self,
        ports: Vec<u16>,
        public_key: &PublicKey,
    ) -> MASQMockNode {
        self.start_mock_node(ports, Some(public_key))
    }

    fn start_mock_node(
        &mut self,
        ports: Vec<u16>,
        public_key_opt: Option<&PublicKey>,
    ) -> MASQMockNode {
        let index = self.next_index;
        self.next_index += 1;
        let node = match public_key_opt {
            Some(public_key) => MASQMockNode::start_with_public_key(
                ports,
                index,
                self.host_node_parent_dir.clone(),
                public_key,
                self.chain,
            ),
            None => {
                MASQMockNode::start(ports, index, self.host_node_parent_dir.clone(), self.chain)
            }
        };
        let name = node.name().to_string();
        self.mock_nodes.insert(name.clone(), node);
        self.mock_nodes.get(&name).unwrap().clone()
    }

    pub fn stop(self) {
        MASQNodeCluster::cleanup().unwrap()
    }

    pub fn stop_node(&mut self, name: &str) {
        match self.real_nodes.remove(name) {
            Some(node) => drop(node),
            None => match self.mock_nodes.remove(name) {
                Some(node) => drop(node),
                None => panic!("Node {} was not found in cluster", name),
            },
        };
    }

    pub fn running_node_names(&self) -> HashSet<String> {
        let mut node_name_refs = vec![];
        node_name_refs.extend(self.real_nodes.keys());
        node_name_refs.extend(self.mock_nodes.keys());
        node_name_refs.into_iter().cloned().collect()
    }

    pub fn get_real_node_by_name(&self, name: &str) -> Option<MASQRealNode> {
        self.real_nodes.get(name).cloned()
    }

    pub fn get_real_node_by_key(&self, key: &PublicKey) -> Option<MASQRealNode> {
        self.real_nodes
            .values()
            .find(|node| node.main_public_key() == key)
            .cloned()
    }

    pub fn get_mock_node_by_name(&self, name: &str) -> Option<MASQMockNode> {
        self.mock_nodes.get(name).cloned()
    }

    pub fn get_node_by_name(&self, name: &str) -> Option<Box<dyn MASQNode>> {
        match self.real_nodes.get(name) {
            Some(node_ref) => Some(Box::new(node_ref.clone())),
            None => self
                .mock_nodes
                .get(name)
                .map(|node_ref| Box::new(node_ref.clone()) as Box<dyn MASQNode>),
        }
    }

    pub fn get_real_node_home_dir_path_by_name(&self, name: String) -> String {
        MASQRealNode::node_home_dir(
            &self
                .host_node_parent_dir
                .clone()
                .unwrap_or_else(MASQNodeUtils::find_project_root),
            &name,
        )
    }

    pub fn is_in_jenkins() -> bool {
        match env::var("HOST_NODE_PARENT_DIR") {
            Ok(ref value) if value.is_empty() => false,
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn cleanup() -> Result<(), String> {
        MASQNodeCluster::stop_running_containers()?;
        if Self::is_in_jenkins() {
            Self::disconnect_network()
        }
        MASQNodeCluster::remove_network_if_running()
    }

    fn stop_running_containers() -> Result<(), String> {
        let mut command = Command::new(
            "docker",
            Command::strings(vec!["ps", "-q", "--filter", "ancestor=test_node_image"]),
        );
        if command.wait_for_exit() != 0 {
            return Err(format!(
                "Could not stop running nodes: {}",
                command.stderr_as_string()
            ));
        }
        let output = command.stdout_as_string();
        let results: Vec<String> = output
            .split('\n')
            .filter(|result| !result.is_empty())
            .map(|container_id| {
                let mut command = Command::new(
                    "docker",
                    Command::strings(vec!["stop", "-t", "0", container_id]),
                );
                match command.wait_for_exit() {
                    0 => Ok(()),
                    _ => Err(format!(
                        "Could not stop node '{}': {}",
                        container_id,
                        command.stderr_as_string()
                    )),
                }
            })
            .filter(|result| result.is_err())
            .map(|result| result.err().unwrap())
            .collect();
        if results.is_empty() {
            Ok(())
        } else {
            Err(results.join("; "))
        }
    }

    fn disconnect_network() {
        let mut command = Command::new(
            "docker",
            Command::strings(vec![
                "network",
                "disconnect",
                "integration_net",
                "subjenkins",
            ]),
        );
        command.wait_for_exit();
    }

    fn remove_network_if_running() -> Result<(), String> {
        let mut command = Command::new("docker", Command::strings(vec!["network", "ls"]));
        if command.wait_for_exit() != 0 {
            return Err(format!(
                "Could not list networks: {}",
                command.stderr_as_string()
            ));
        }
        let output = command.stdout_as_string();
        if !output.contains("integration_net") {
            return Ok(());
        }
        let mut command = Command::new(
            "docker",
            Command::strings(vec!["network", "rm", "integration_net"]),
        );
        match command.wait_for_exit() {
            0 => Ok(()),
            _ => Err(format!(
                "Could not remove network integration_net: {}",
                command.stderr_as_string()
            )),
        }
    }

    fn create_network() -> Result<(), String> {
        let mut command = Command::new(
            "docker",
            Command::strings(vec![
                "network",
                "create",
                "--subnet=172.18.0.0/16",
                "integration_net",
            ]),
        );
        match command.wait_for_exit() {
            0 => Ok(()),
            _ => Err(format!(
                "Could not create network integration_net: {}",
                command.stderr_as_string()
            )),
        }
    }

    fn interconnect_network() -> Result<(), String> {
        let mut command = Command::new(
            "docker",
            Command::strings(vec!["network", "connect", "integration_net", "subjenkins"]),
        );
        match command.wait_for_exit() {
            0 => Ok(()),
            _ => Err(format!(
                "Could not connect subjenkins to integration_net: {}",
                command.stderr_as_string()
            )),
        }
    }
}
