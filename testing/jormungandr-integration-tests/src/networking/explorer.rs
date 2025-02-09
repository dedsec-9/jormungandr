use jormungandr_lib::interfaces::BlockDate;
use jormungandr_lib::interfaces::Explorer;
use jormungandr_testing_utils::testing::network::blockchain::BlockchainBuilder;
use jormungandr_testing_utils::testing::network::builder::NetworkBuilder;
use jormungandr_testing_utils::testing::network::wallet::template::builder::WalletTemplateBuilder;
use jormungandr_testing_utils::testing::network::Node;
use jormungandr_testing_utils::testing::network::SpawnParams;
use jormungandr_testing_utils::testing::network::Topology;
use jormungandr_testing_utils::testing::node::time;
use jormungandr_testing_utils::testing::FragmentSender;
const LEADER_1: &str = "Leader_1";
const LEADER_2: &str = "Leader_2";
const LEADER_3: &str = "Leader_3";
const PASSIVE: &str = "Passive";

const ALICE: &str = "ALICE";
const BOB: &str = "BOB";
const CLARICE: &str = "CLARICE";

#[test]
pub fn passive_node_explorer() {
    let mut controller = NetworkBuilder::default()
        .topology(
            Topology::default()
                .with_node(Node::new(LEADER_1))
                .with_node(Node::new(LEADER_2).with_trusted_peer(LEADER_1))
                .with_node(Node::new(LEADER_3).with_trusted_peer(LEADER_1))
                .with_node(
                    Node::new(PASSIVE)
                        .with_trusted_peer(LEADER_1)
                        .with_trusted_peer(LEADER_2)
                        .with_trusted_peer(LEADER_3),
                ),
        )
        .blockchain_config(
            BlockchainBuilder::default()
                .slots_per_epoch(60)
                .slot_duration(2)
                .build(),
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
        .wallet_template(
            WalletTemplateBuilder::new(CLARICE)
                .with(2_000_000_000)
                .delegated_to(LEADER_2)
                .build(),
        )
        .build()
        .unwrap();

    let leader_1 = controller
        .spawn(SpawnParams::new(LEADER_1).in_memory())
        .unwrap();
    let _leader_2 = controller
        .spawn(SpawnParams::new(LEADER_2).in_memory())
        .unwrap();
    let _leader_3 = controller
        .spawn(SpawnParams::new(LEADER_3).in_memory())
        .unwrap();

    let passive = controller
        .spawn(
            SpawnParams::new(PASSIVE)
                .passive()
                .in_memory()
                .explorer(Explorer { enabled: true }),
        )
        .unwrap();
    let mut alice = controller.wallet(ALICE).unwrap();
    let bob = controller.wallet(BOB).unwrap();

    let mem_pool_check = FragmentSender::from(&controller)
        .send_transaction(&mut alice, &bob, &leader_1, 1_000.into())
        .unwrap();

    // give some time to update explorer
    time::wait_for_date(BlockDate::new(0, 30), leader_1.rest());

    let transaction_id = passive
        .explorer()
        .transaction((*mem_pool_check.fragment_id()).into())
        .unwrap()
        .data
        .unwrap()
        .transaction
        .id;

    assert_eq!(
        &transaction_id,
        &mem_pool_check.fragment_id().to_string(),
        "Wrong transaction id in explorer",
    );
}
