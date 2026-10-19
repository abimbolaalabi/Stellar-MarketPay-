use soroban_sdk::{Address, Env, String, Vec};

use crate::types::{DataKey, EscrowStatus, EscrowTransition, EscrowTransitionKind};

/// Maximum number of transitions retained per job.
///
/// Escrow history is stored on-chain so disputes and audits can verify state
/// changes without trusting the off-chain indexer. To keep the storage cost
/// bounded, only the most recent `MAX_ESCROW_HISTORY` transitions are kept;
/// the oldest entry is evicted whenever a new one would exceed the cap.
pub(crate) const MAX_ESCROW_HISTORY: u32 = 20;

/// Append a transition to a job's bounded on-chain history.
pub(crate) fn record_transition(
    env: &Env,
    job_id: &String,
    kind: EscrowTransitionKind,
    from: EscrowStatus,
    to: EscrowStatus,
    actor: &Address,
) {
    let mut history: Vec<EscrowTransition> = env
        .storage()
        .instance()
        .get(&DataKey::EscrowHistory(job_id.clone()))
        .unwrap_or_else(|| Vec::new(env));

    // Evict the oldest transitions so the log never grows past the cap.
    while history.len() >= MAX_ESCROW_HISTORY {
        history.remove(0);
    }

    history.push_back(EscrowTransition {
        kind,
        from,
        to,
        actor: actor.clone(),
        ledger: env.ledger().sequence(),
        timestamp: env.ledger().timestamp(),
    });

    env.storage()
        .instance()
        .set(&DataKey::EscrowHistory(job_id.clone()), &history);
}

/// Return a job's recorded escrow transitions, oldest first.
///
/// Unknown jobs return an empty vector rather than panicking so callers can
/// use the view while probing for a job that has not been created yet.
pub(crate) fn get_escrow_history(env: Env, job_id: String) -> Vec<EscrowTransition> {
    env.storage()
        .instance()
        .get(&DataKey::EscrowHistory(job_id))
        .unwrap_or_else(|| Vec::new(&env))
}
