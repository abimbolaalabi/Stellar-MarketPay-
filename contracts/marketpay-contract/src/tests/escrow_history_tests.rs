use crate::escrow_history::MAX_ESCROW_HISTORY;
use crate::*;
use soroban_sdk::{testutils::Address as _, testutils::Ledger, token, Address, Env, String};

fn setup(env: &Env) -> (MarketPayContractClient, Address, Address, Address, Address) {
    env.mock_all_auths();
    let id = env.register(MarketPayContract, ());
    let contract = MarketPayContractClient::new(env, &id);
    let admin = Address::generate(env);
    let treasury = Address::generate(env);
    contract.initialize(&admin, &treasury, &String::from_str(env, "1.0.0"));

    let client = Address::generate(env);
    let freelancer = Address::generate(env);
    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_id = token_contract.address();
    let token_admin = token::StellarAssetClient::new(env, &token_id);
    token_admin.mint(&client, &1_000);

    (contract, admin, client, freelancer, token_id)
}

fn create_escrow(
    env: &Env,
    contract: &MarketPayContractClient,
    client: &Address,
    freelancer: &Address,
    token_id: &Address,
    job: &str,
    timeout_ledgers: Option<u32>,
) -> String {
    let job_id = String::from_str(env, job);
    contract.create_escrow(
        &job_id,
        client,
        &CreateEscrowParams {
            freelancer: freelancer.clone(),
            token: token_id.clone(),
            amount: 1_000,
            milestones: None,
            timeout_ledgers,
            referrer: None,
        },
    );
    job_id
}

#[test]
fn test_history_records_create_fund_start_and_release() {
    let env = Env::default();
    let (contract, _admin, client, freelancer, token_id) = setup(&env);
    let job_id = create_escrow(
        &env,
        &contract,
        &client,
        &freelancer,
        &token_id,
        "hist-1",
        None,
    );

    let history = contract.get_escrow_history(&job_id);
    assert_eq!(history.len(), 1);
    let created = history.get(0).unwrap();
    assert_eq!(created.kind, EscrowTransitionKind::Created);
    assert_eq!(created.from, EscrowStatus::Locked);
    assert_eq!(created.to, EscrowStatus::Locked);
    assert_eq!(created.actor, client);
    assert_eq!(created.ledger, env.ledger().sequence());
    assert_eq!(created.timestamp, env.ledger().timestamp());

    contract.start_work(&job_id, &freelancer);
    contract.release_escrow(&job_id, &client);

    let history = contract.get_escrow_history(&job_id);
    assert_eq!(history.len(), 3);

    let started = history.get(1).unwrap();
    assert_eq!(started.kind, EscrowTransitionKind::WorkStarted);
    assert_eq!(started.from, EscrowStatus::Locked);
    assert_eq!(started.to, EscrowStatus::InProgress);
    assert_eq!(started.actor, freelancer);

    let released = history.get(2).unwrap();
    assert_eq!(released.kind, EscrowTransitionKind::Released);
    assert_eq!(released.from, EscrowStatus::InProgress);
    assert_eq!(released.to, EscrowStatus::Released);
    assert_eq!(released.actor, client);
}

#[test]
fn test_history_records_refund() {
    let env = Env::default();
    let (contract, _admin, client, freelancer, token_id) = setup(&env);
    let job_id = create_escrow(
        &env,
        &contract,
        &client,
        &freelancer,
        &token_id,
        "hist-2",
        None,
    );

    contract.refund_escrow(&job_id, &client);

    let history = contract.get_escrow_history(&job_id);
    assert_eq!(history.len(), 2);
    let refunded = history.get(1).unwrap();
    assert_eq!(refunded.kind, EscrowTransitionKind::Refunded);
    assert_eq!(refunded.from, EscrowStatus::Locked);
    assert_eq!(refunded.to, EscrowStatus::Refunded);
    assert_eq!(refunded.actor, client);
}

#[test]
fn test_history_records_timeout_refund() {
    let env = Env::default();
    let (contract, _admin, client, freelancer, token_id) = setup(&env);
    let job_id = create_escrow(
        &env,
        &contract,
        &client,
        &freelancer,
        &token_id,
        "hist-3",
        Some(10),
    );

    let mut ledger = env.ledger().get();
    ledger.sequence_number += 11;
    ledger.timestamp += (DEFAULT_TIMEOUT_SECONDS + 1) as u64;
    env.ledger().set(ledger);

    contract.timeout_refund(&job_id, &client);

    let history = contract.get_escrow_history(&job_id);
    let refunded = history.get(1).unwrap();
    assert_eq!(refunded.kind, EscrowTransitionKind::Refunded);
    assert_eq!(refunded.from, EscrowStatus::Locked);
    assert_eq!(refunded.to, EscrowStatus::Refunded);
    assert_eq!(refunded.actor, client);
}

#[test]
fn test_history_records_dispute_and_resolution() {
    let env = Env::default();
    let (contract, admin, client, freelancer, token_id) = setup(&env);
    let job_id = create_escrow(
        &env,
        &contract,
        &client,
        &freelancer,
        &token_id,
        "hist-4",
        None,
    );

    contract.raise_dispute(&job_id, &client);

    let history = contract.get_escrow_history(&job_id);
    let disputed = history.get(1).unwrap();
    assert_eq!(disputed.kind, EscrowTransitionKind::Disputed);
    assert_eq!(disputed.from, EscrowStatus::Locked);
    assert_eq!(disputed.to, EscrowStatus::Disputed);
    assert_eq!(disputed.actor, client);

    let arbitrator = Address::generate(&env);
    contract.set_arbitrator(&admin, &arbitrator);
    contract.resolve_dispute(&job_id, &arbitrator, &client, &100, &0);

    let history = contract.get_escrow_history(&job_id);
    let resolved = history.get(2).unwrap();
    assert_eq!(resolved.kind, EscrowTransitionKind::DisputeResolved);
    assert_eq!(resolved.from, EscrowStatus::Disputed);
    assert_eq!(resolved.to, EscrowStatus::Released);
    assert_eq!(resolved.actor, arbitrator);
}

#[test]
fn test_history_records_freeze_and_unfreeze() {
    let env = Env::default();
    let (contract, admin, client, freelancer, token_id) = setup(&env);
    let job_id = create_escrow(
        &env,
        &contract,
        &client,
        &freelancer,
        &token_id,
        "hist-5",
        None,
    );

    contract.freeze_escrow(&job_id, &admin);
    contract.unfreeze_escrow(&job_id, &admin);

    let history = contract.get_escrow_history(&job_id);
    assert_eq!(history.len(), 3);

    let frozen = history.get(1).unwrap();
    assert_eq!(frozen.kind, EscrowTransitionKind::Frozen);
    assert_eq!(frozen.from, EscrowStatus::Locked);
    assert_eq!(frozen.to, EscrowStatus::Frozen);
    assert_eq!(frozen.actor, admin);

    let unfrozen = history.get(2).unwrap();
    assert_eq!(unfrozen.kind, EscrowTransitionKind::Unfrozen);
    assert_eq!(unfrozen.from, EscrowStatus::Frozen);
    assert_eq!(unfrozen.to, EscrowStatus::Locked);
    assert_eq!(unfrozen.actor, admin);
}

#[test]
fn test_history_is_bounded_to_most_recent_transitions() {
    let env = Env::default();
    let (contract, admin, client, freelancer, token_id) = setup(&env);
    let job_id = create_escrow(
        &env,
        &contract,
        &client,
        &freelancer,
        &token_id,
        "hist-6",
        None,
    );

    // Each cycle adds two transitions (Frozen + Unfrozen).
    for _ in 0..(MAX_ESCROW_HISTORY / 2 + 5) {
        contract.freeze_escrow(&job_id, &admin);
        contract.unfreeze_escrow(&job_id, &admin);
    }

    let history = contract.get_escrow_history(&job_id);
    assert_eq!(history.len(), MAX_ESCROW_HISTORY);

    // The original creation entry has been evicted.
    let mut contains_created = false;
    for transition in history.iter() {
        if transition.kind == EscrowTransitionKind::Created {
            contains_created = true;
        }
    }
    assert!(!contains_created);

    // The newest transition is retained.
    assert_eq!(
        history.get(MAX_ESCROW_HISTORY - 1).unwrap().kind,
        EscrowTransitionKind::Unfrozen
    );
}

#[test]
fn test_get_escrow_history_returns_empty_for_unknown_job() {
    let env = Env::default();
    let (contract, _admin, _client, _freelancer, _token_id) = setup(&env);

    let history = contract.get_escrow_history(&String::from_str(&env, "missing"));
    assert_eq!(history.len(), 0);
}
