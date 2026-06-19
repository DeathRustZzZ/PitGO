//! Аудит-контекст действия. Не содержит логики проверки прав доступа.

use crate::ids::{ActorId, CausationId, CorrelationId};

/// Снимок роли актора на момент выполнения действия.
///
/// Намеренно тонкий тип: хранит только строковое имя роли для аудита.
/// Живая логика авторизации реализуется отдельно в Authorization BC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleSnapshot {
    pub name: String,
}

impl RoleSnapshot {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Контекст действия, переносимый через слои приложения для аудита и корреляции.
///
/// Не содержит проверок прав: авторизация выполняется отдельно до создания этого
/// контекста.
#[derive(Debug, Clone)]
pub struct ActionContext {
    pub actor_id: ActorId,
    pub roles: Vec<RoleSnapshot>,
    pub correlation_id: CorrelationId,
    pub causation_id: CausationId,
}

impl ActionContext {
    pub fn new(
        actor_id: ActorId,
        roles: Vec<RoleSnapshot>,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Self {
        Self {
            actor_id,
            roles,
            correlation_id,
            causation_id,
        }
    }
}
