#![allow(dead_code)]

use super::Error;
use crate::controller::monitor::ProgressBarController;
use crate::style;
use chain_impl_mockchain::fragment::Fragment;
use chain_impl_mockchain::fragment::FragmentId;
use chain_impl_mockchain::header::HeaderId;
use jormungandr_lib::interfaces::FragmentsProcessingSummary;
use jormungandr_lib::multiaddr;
use jormungandr_lib::{
    crypto::hash::Hash,
    interfaces::{BlockDate, FragmentLog},
};
use jormungandr_testing_utils::testing::node::configuration::legacy::NodeConfig as LegacyConfig;
use jormungandr_testing_utils::testing::node::LogLevel;
use jormungandr_testing_utils::testing::SyncNode;
pub use jormungandr_testing_utils::testing::{
    jormungandr::{JormungandrProcess, StartupVerificationMode, Status},
    network::{LeadershipMode, NodeAlias, NodeBlock0, NodeSetting, PersistenceMode, Settings},
    node::{grpc::JormungandrClient, BackwardCompatibleRest, JormungandrLogger, JormungandrRest},
    FragmentNode, FragmentNodeError, MemPoolCheck,
};
use std::collections::HashMap;

use std::io::{BufRead, BufReader};
use std::process::ExitStatus;
use std::time::Duration;
use yaml_rust::{Yaml, YamlLoader};

pub struct LegacyNode {
    pub process: JormungandrProcess,
    pub progress_bar: ProgressBarController,
    pub legacy_settings: LegacyConfig,
}

impl LegacyNode {
    pub fn new(
        process: JormungandrProcess,
        progress_bar: ProgressBarController,
        legacy_settings: LegacyConfig,
    ) -> Self {
        let node = LegacyNode {
            process,
            progress_bar,
            legacy_settings,
        };
        node.progress_bar_start();
        node
    }

    pub fn alias(&self) -> NodeAlias {
        self.process.alias()
    }

    pub fn status(&self) -> Status {
        self.process.status(&StartupVerificationMode::Rest)
    }

    pub fn check_running(&self) -> bool {
        self.status() == Status::Running
    }

    pub fn progress_bar(&self) -> &ProgressBarController {
        &self.progress_bar
    }

    pub fn log(&self, info: &str) {
        self.progress_bar.log_info(info);
    }

    pub fn genesis_block_hash(&self) -> Result<HeaderId, Error> {
        Ok(self.process.grpc().get_genesis_block_hash())
    }

    pub fn legacy_rest(&self) -> BackwardCompatibleRest {
        BackwardCompatibleRest::new(self.process.rest_address().to_string(), Default::default())
    }

    pub fn rest(&self) -> JormungandrRest {
        self.process.rest()
    }

    pub fn stats(&self) -> Result<Yaml, Error> {
        let stats = self.legacy_rest().stats()?;
        let docs = YamlLoader::load_from_str(&stats)?;
        Ok(docs.get(0).unwrap().clone())
    }

    pub fn log_stats(&self) {
        self.progress_bar
            .log_info(format!("node stats ({:?})", self.stats()));
    }

    pub fn wait_for_bootstrap(&self) -> Result<(), Error> {
        self.process
            .wait_for_bootstrap(&StartupVerificationMode::Rest, Duration::from_secs(150))
            .map_err(|e| Error::NodeFailedToBootstrap {
                alias: self.alias(),
                e,
            })
    }

    pub fn wait_for_shutdown(&mut self) -> Result<Option<ExitStatus>, Error> {
        self.process
            .wait_for_shutdown(Duration::from_secs(30))
            .map_err(|e| Error::NodeFailedToShutdown {
                alias: self.alias(),
                e,
            })
    }

    #[allow(deprecated)]
    fn ports_are_opened(&self) -> bool {
        self.port_opened(self.legacy_settings.rest.listen.port())
            && self.port_opened(
                multiaddr::to_tcp_socket_addr(&self.legacy_settings.p2p.public_address)
                    .unwrap()
                    .port(),
            )
    }

    fn port_opened(&self, port: u16) -> bool {
        use std::net::TcpListener;
        TcpListener::bind(("127.0.0.1", port)).is_ok()
    }

    pub fn logger(&self) -> &JormungandrLogger {
        &self.process.logger
    }

    pub fn is_up(&self) -> bool {
        matches!(self.status(), Status::Running)
    }

    pub fn shutdown(&mut self) -> Result<Option<ExitStatus>, Error> {
        self.progress_bar.log_info("shutting down..");
        let message = self.rest().shutdown()?;
        if message.is_empty() {
            self.wait_for_shutdown()
        } else {
            Err(Error::ShutdownProcedure {
                alias: self.alias(),
                message,
                logs: self.logger().get_lines_as_string(),
            })
        }
    }

    pub fn capture_logs(&mut self) {
        let stderr = self.process.child.stderr.take().unwrap();
        let reader = BufReader::new(stderr);
        for line_result in reader.lines() {
            let line = line_result.expect("failed to read a line from log output");
            self.progress_bar.log_info(&line);
        }
    }

    pub fn progress_bar_start(&self) {
        self.progress_bar.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("{spinner:.green} {wide_msg}")
                .tick_chars(style::TICKER),
        );
        self.progress_bar.enable_steady_tick(100);
        self.progress_bar.set_message(&format!(
            "{} {} ... [{}]",
            *style::icons::jormungandr,
            style::binary.apply_to(self.alias()),
            self.legacy_settings.rest.listen,
        ));
    }

    fn progress_bar_failure(&self) {
        self.progress_bar.finish_with_message(&format!(
            "{} {} {}",
            *style::icons::jormungandr,
            style::binary.apply_to(self.alias()),
            style::error.apply_to(*style::icons::failure)
        ));
    }

    fn progress_bar_success(&self) {
        self.progress_bar.finish_with_message(&format!(
            "{} {} {}",
            *style::icons::jormungandr,
            style::binary.apply_to(self.alias()),
            style::success.apply_to(*style::icons::success)
        ));
    }
}

impl FragmentNode for LegacyNode {
    fn alias(&self) -> NodeAlias {
        self.alias()
    }
    fn fragment_logs(
        &self,
    ) -> std::result::Result<HashMap<FragmentId, FragmentLog>, FragmentNodeError> {
        //TODO: implement conversion
        self.rest()
            .fragment_logs()
            .map_err(|_| FragmentNodeError::UnknownError)
    }
    fn send_fragment(
        &self,
        fragment: Fragment,
    ) -> std::result::Result<MemPoolCheck, FragmentNodeError> {
        //TODO: implement conversion
        self.rest()
            .send_fragment(fragment)
            .map_err(|_| FragmentNodeError::UnknownError)
    }

    fn send_batch_fragments(
        &self,
        _fragments: Vec<Fragment>,
        _fail_fast: bool,
    ) -> std::result::Result<FragmentsProcessingSummary, FragmentNodeError> {
        //TODO implement
        unimplemented!()
    }

    fn log_pending_fragment(&self, fragment_id: FragmentId) {
        self.progress_bar()
            .log_info(format!("Fragment '{}' is still pending", fragment_id));
    }
    fn log_rejected_fragment(&self, fragment_id: FragmentId, reason: String) {
        self.progress_bar()
            .log_info(format!("Fragment '{}' rejected: {}", fragment_id, reason));
    }
    fn log_in_block_fragment(&self, fragment_id: FragmentId, date: BlockDate, block: Hash) {
        self.progress_bar().log_info(format!(
            "Fragment '{}' in block: {} ({})",
            fragment_id, block, date
        ));
    }
    fn log_content(&self) -> Vec<String> {
        self.logger().get_lines_as_string()
    }
}

impl SyncNode for LegacyNode {
    fn alias(&self) -> NodeAlias {
        self.alias()
    }

    fn last_block_height(&self) -> u32 {
        self.stats().unwrap()["lastBlockHeight"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap()
    }

    fn log_stats(&self) {
        println!("Node: {} -> {:?}", self.alias(), self.stats());
    }

    fn tip(&self) -> Hash {
        self.rest().tip().expect("cannot get tip from rest")
    }

    fn log_content(&self) -> String {
        self.logger().get_log_content()
    }

    fn get_lines_with_error_and_invalid(&self) -> Vec<String> {
        self.logger()
            .get_lines_with_level(LogLevel::ERROR)
            .map(|x| x.to_string())
            .collect()
    }

    fn is_running(&self) -> bool {
        self.stats().unwrap()["state"].as_str().unwrap() == "Running"
    }
}