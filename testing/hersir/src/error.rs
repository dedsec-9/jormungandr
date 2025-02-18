use crate::controller::Error as ControllerError;
use jormungandr_testing_utils::testing::{
    network::controller::ControllerError as InnerControllerError, node::ExplorerError,
    node::RestError, ConsumptionBenchmarkError, FragmentSenderError,
};
use jortestkit::prelude::InteractiveCommandError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    MonitorNode(#[from] crate::controller::NodeError),
    #[error(transparent)]
    InteractiveCommand(#[from] InteractiveCommandError),
    #[error(transparent)]
    InnerController(#[from] InnerControllerError),
    #[error(transparent)]
    Controller(#[from] ControllerError),
    #[error(transparent)]
    Verification(#[from] jormungandr_testing_utils::testing::VerificationError),
    #[error(transparent)]
    FragmentVerifier(#[from] jormungandr_testing_utils::testing::FragmentVerifierError),
    #[error(transparent)]
    ConsumptionBenchmark(#[from] ConsumptionBenchmarkError),
    #[error(transparent)]
    Explorer(#[from] ExplorerError),
    #[error(transparent)]
    FragmentSender(#[from] FragmentSenderError),
    #[error("Rest error: {0}")]
    Rest(#[from] RestError),
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_yaml::Error),
    #[error("Circular dependency in network topology")]
    CircularTrust,
    #[error("INTERNAL ERROR: {0}")]
    Internal(String),
}
