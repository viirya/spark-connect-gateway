use std::sync::Arc;

use scg_routing::{AffinityStore, BackendMember, Pool, Router, SessionKey};
use scg_store_memory::MemoryStore;

// Two healthy members; placement uses Router's default round-robin.
struct TwoBackends;
impl Pool for TwoBackends {
    fn members(&self) -> Vec<BackendMember> {
        vec![BackendMember::new("A"), BackendMember::new("B")]
    }
}

fn router() -> Router {
    Router::single_pool(
        Arc::new(TwoBackends),
        Arc::new(MemoryStore::new()) as Arc<dyn AffinityStore>,
    )
}

// Encodes the CloneSession flow as the handler performs it: resolve
// the parent, remember the cloned session id on that backend, then
// resolve the cloned id and require the same address.
#[tokio::test]
async fn cloned_session_stays_on_parent_backend() {
    let r = router();

    let parent = SessionKey::with_tenant("t", "alice", "orig");
    let parent_addr = r.resolve_session(&parent).await.unwrap().unwrap();

    let clone_backend = r.resolve_session(&parent).await.unwrap().unwrap();
    assert_eq!(clone_backend, parent_addr);
    let new_sid = "cloned";
    let new_key = SessionKey::with_tenant("t", "alice", new_sid);
    r.remember_session(&new_key, clone_backend.clone()).await;

    let followup = r.resolve_session(&new_key).await.unwrap().unwrap();
    assert_eq!(
        followup, clone_backend,
        "cloned session must stay on the parent's backend"
    );
}
