pub mod config;
pub mod account;
pub mod actor;
pub mod position;
pub mod risk;
pub mod domain_event;
pub mod snapshot;

// Not: eski `db_writer` modülü (ayrı PersistEvent kanalı) kaldırıldı.
// Kalıcılık artık tek DomainEvent kanalından beslenen projection'lar
// (paper-service/src/sqlite_projection.rs + event store) ile yapılır.
// pub mod recovery;
