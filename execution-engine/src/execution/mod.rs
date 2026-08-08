//! Emir yürütme çekirdeği: doğrulama, yaşam döngüsü, idempotency, actor.

pub mod actor;
pub mod idempotency;
pub mod lifecycle;
pub mod preflight;

pub use actor::{Command, ExecutionActor, UserEvent};
pub use preflight::{new_client_order_id, Preflight};
