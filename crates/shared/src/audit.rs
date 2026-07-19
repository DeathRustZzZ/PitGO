//! Audit context for an action. Contains no access-control logic.
//!
//! The types here answer "who did this, and as part of which business process",
//! which is an audit question. Whether the actor was *allowed* to do it is a
//! separate concern, resolved before this context is ever constructed.
//!
//! Аудит-контекст действия. Не содержит логики проверки прав доступа.
//!
//! Типы в этом модуле отвечают на вопрос «кто выполнил действие и в рамках
//! какого бизнес-процесса» — это вопрос аудита. Имел ли актор *право* на это
//! действие — отдельная задача, решаемая до создания данного контекста.

use crate::ids::{ActorId, CausationId, CorrelationId};

/// Snapshot of the actor's role at the moment the action was performed.
///
/// Deliberately a thin type: it stores only the role name as a string, for the
/// audit trail. Roles are captured as a snapshot rather than looked up later
/// because a role granted today may be revoked tomorrow — the audit record must
/// reflect the permissions in effect at the time, not the current ones.
/// Live authorization logic lives separately in the Authorization bounded
/// context.
///
/// Снимок роли актора на момент выполнения действия.
///
/// Намеренно тонкий тип: хранит только строковое имя роли для аудита. Роли
/// фиксируются снимком, а не запрашиваются позже, потому что выданная сегодня
/// роль может быть отозвана завтра — запись аудита должна отражать права,
/// действовавшие на момент события, а не текущие. Живая логика авторизации
/// реализуется отдельно в Authorization BC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleSnapshot {
    /// Role name as it was known at the time of the action.
    ///
    /// Имя роли в том виде, в каком оно было известно на момент действия.
    pub name: String,
}

impl RoleSnapshot {
    /// Creates a role snapshot from anything convertible into a `String`.
    ///
    /// Создаёт снимок роли из любого значения, преобразуемого в `String`.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Action context carried across application layers for audit and correlation.
///
/// `correlation_id` groups every event belonging to one business process, while
/// `causation_id` points at the single command or event that directly triggered
/// this action. Together they let an operator reconstruct a distributed call
/// chain from the event log alone — correlation gives the whole tree, causation
/// gives the parent edge.
///
/// Carries no permission checks: authorization is performed separately, before
/// this context is created.
///
/// Контекст действия, переносимый через слои приложения для аудита и корреляции.
///
/// `correlation_id` объединяет все события одного бизнес-процесса, а
/// `causation_id` указывает на конкретную команду или событие, непосредственно
/// вызвавшее данное действие. Вместе они позволяют восстановить распределённую
/// цепочку вызовов по одному лишь журналу событий — корреляция даёт всё дерево,
/// каузация даёт ребро к родителю.
///
/// Не содержит проверок прав: авторизация выполняется отдельно, до создания
/// этого контекста.
#[derive(Debug, Clone)]
pub struct ActionContext {
    /// Actor that performed the action.
    ///
    /// Актор, выполнивший действие.
    pub actor_id: ActorId,
    /// Roles held by the actor at the time of the action.
    ///
    /// Роли, которыми обладал актор на момент действия.
    pub roles: Vec<RoleSnapshot>,
    /// Groups all events belonging to the same business process.
    ///
    /// Объединяет все события, принадлежащие одному бизнес-процессу.
    pub correlation_id: CorrelationId,
    /// Command or event that directly caused this action.
    ///
    /// Команда или событие, непосредственно вызвавшее данное действие.
    pub causation_id: CausationId,
}

impl ActionContext {
    /// Assembles an action context from an already-authorized actor.
    ///
    /// Собирает контекст действия для уже авторизованного актора.
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
