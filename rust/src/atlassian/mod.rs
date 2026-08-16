//! Integracion con Atlassian (feature #15): el arnes aprende a que proyecto
//! Jira y a que space de Confluence pertenece cada repo, y cada transicion del
//! flujo deja su rastro del otro lado.
//!
//! Arquitectura (decision OBS-1, hibrida):
//!
//! ```text
//! flujo (add/start/advance/approve-spec/close)
//!    |__ emit    -> outbox/*.json        (que deberia existir del otro lado)
//!                      |__ drain  -> plan de llamadas MCP  -> ack  (agente)
//!                      |__ apply  -> REST con token                (solo)
//!                                        |__ state.json (mapa local -> remoto)
//! ```
//!
//! El binario no habla MCP — el MCP vive en el agente —, asi que la outbox es
//! el contrato comun: los dos ejecutores producen exactamente lo mismo.

pub mod binding;
pub mod confluence;
pub mod emit;
pub mod http;
pub mod jira;
pub mod markdown;
pub mod outbox;
pub mod state;
