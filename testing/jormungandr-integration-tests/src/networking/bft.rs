use crate::networking::utils;
use jormungandr_testing_utils::testing::network::builder::NetworkBuilder;
use jormungandr_testing_utils::testing::network::wallet::template::builder::WalletTemplateBuilder;
use jormungandr_testing_utils::testing::network::Node;
use jormungandr_testing_utils::testing::network::SpawnParams;
use jormungandr_testing_utils::testing::network::Topology;
use jormungandr_testing_utils::testing::sync::MeasurementReportInterval;
use jormungandr_testing_utils::testing::FragmentSender;
use jormungandr_testing_utils::testing::FragmentVerifier;
use jormungandr_testing_utils::testing::SyncWaitParams;
use std::time::Duration;

const LEADER_1: &str = "Leader1";
const LEADER_2: &str = "Leader2";
const LEADER_3: &str = "Leader3";
const LEADER_4: &str = "Leader4";
const LEADER_5: &str = "Leader5";
const PASSIVE: &str = "Passive";

const ALICE: &str = "ALICE";
const BOB: &str = "BOB";

#[test]
pub fn bft_cascade() {
    let mut controller = NetworkBuilder::default()
        .topology(
            Topology::default()
                .with_node(Node::new(LEADER_1))
                .with_node(Node::new(LEADER_2).with_trusted_peer(LEADER_1))
                .with_node(
                    Node::new(LEADER_3)
                        .with_trusted_peer(LEADER_2)
                        .with_trusted_peer(LEADER_1),
                )
                .with_node(
                    Node::new(LEADER_4)
                        .with_trusted_peer(LEADER_3)
                        .with_trusted_peer(LEADER_2),
                )
                .with_node(
                    Node::new(LEADER_5)
                        .with_trusted_peer(LEADER_4)
                        .with_trusted_peer(LEADER_3),
                ),
        )
        .wallet_template(
            WalletTemplateBuilder::new(ALICE)
                .with(2_000_000_000)
                .build(),
        )
        .wallet_template(
            WalletTemplateBuilder::new(BOB)
                .with(2_000_000_000)
                .delegated_to(LEADER_1)
                .build(),
        )
        .build()
        .unwrap();

    let leader1 = controller
        .spawn(SpawnParams::new(LEADER_1).in_memory())
        .unwrap();

    let leader2 = controller
        .spawn(SpawnParams::new(LEADER_2).in_memory())
        .unwrap();

    let leader3 = controller
        .spawn(SpawnParams::new(LEADER_3).in_memory())
        .unwrap();

    let leader4 = controller
        .spawn(SpawnParams::new(LEADER_4).in_memory())
        .unwrap();

    let leader5 = controller
        .spawn(SpawnParams::new(LEADER_5).in_memory())
        .unwrap();

    let leaders = [&leader1, &leader2, &leader3, &leader4, &leader5];

    utils::measure_and_log_sync_time(
        &leaders,
        SyncWaitParams::network_size(5, 3).into(),
        "bft cascade sync",
        MeasurementReportInterval::Standard,
    )
    .unwrap();

    let mut alice = controller.wallet(ALICE).unwrap();
    let mut bob = controller.wallet(BOB).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(60));

    FragmentSender::from(controller.settings())
        .send_transactions_round_trip(40, &mut alice, &mut bob, &leader5, 1_000.into())
        .unwrap();

    utils::measure_and_log_sync_time(
        &leaders,
        SyncWaitParams::network_size(5, 3).into(),
        "bft cascade sync",
        MeasurementReportInterval::Standard,
    )
    .unwrap();
}

#[test]
pub fn bft_passive_propagation() {
    let mut controller = NetworkBuilder::default()
        .topology(
            Topology::default()
                .with_node(Node::new(LEADER_3))
                .with_node(Node::new(LEADER_1).with_trusted_peer(LEADER_3))
                .with_node(Node::new(LEADER_2).with_trusted_peer(LEADER_1))
                .with_node(
                    Node::new(PASSIVE)
                        .with_trusted_peer(LEADER_2)
                        .with_trusted_peer(LEADER_3),
                ),
        )
        .wallet_template(
            WalletTemplateBuilder::new(ALICE)
                .with(2_000_000_000)
                .build(),
        )
        .wallet_template(
            WalletTemplateBuilder::new(BOB)
                .with(2_000_000_000)
                .delegated_to(LEADER_3)
                .build(),
        )
        .build()
        .unwrap();

    let leader3 = controller
        .spawn(SpawnParams::new(LEADER_3).in_memory())
        .unwrap();

    let leader1 = controller
        .spawn(SpawnParams::new(LEADER_1).in_memory())
        .unwrap();

    let leader2 = controller
        .spawn(SpawnParams::new(LEADER_2).in_memory())
        .unwrap();

    let passive = controller
        .spawn(SpawnParams::new(PASSIVE).passive().in_memory())
        .unwrap();

    let nodes = [&leader1, &leader2, &leader3, &passive];

    utils::measure_and_log_sync_time(
        &nodes,
        SyncWaitParams::network_size(4, 3).into(),
        "bft passive propagation sync",
        MeasurementReportInterval::Standard,
    )
    .unwrap();

    let mut alice_wallet = controller.wallet(ALICE).unwrap();
    let bob_wallet = controller.wallet(BOB).unwrap();

    let mem_pool_check = FragmentSender::from(controller.settings())
        .send_transaction(&mut alice_wallet, &bob_wallet, &leader2, 1_000.into())
        .unwrap();

    FragmentVerifier::wait_and_verify_is_in_block(
        Duration::new(2, 0),
        mem_pool_check.clone(),
        &leader1,
    )
    .unwrap();

    FragmentVerifier::wait_and_verify_is_in_block(
        Duration::new(2, 0),
        mem_pool_check.clone(),
        &leader2,
    )
    .unwrap();

    FragmentVerifier::wait_and_verify_is_in_block(
        Duration::new(2, 0),
        mem_pool_check.clone(),
        &leader3,
    )
    .unwrap();

    FragmentVerifier::wait_and_verify_is_in_block(Duration::new(2, 0), mem_pool_check, &passive)
        .unwrap();
}
