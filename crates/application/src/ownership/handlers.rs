//! Use-case handlers for the vehicle-ownership context.
//!
//! Обработчики сценариев использования для контекста владения автомобилем.

use crate::error::ApplicationError;
use crate::ownership::commands::StartVehicleOwnershipCommand;
use crate::ownership::ports::VehicleOwnershipRepository;
use chrono::Utc;
use domain::vehicle_ownership::aggregate::VehicleOwnership;
use domain::vehicle_ownership::snapshot::OwnershipEligibilitySnapshot;
use std::sync::Arc;

/// Handler for the "start a vehicle ownership" use case.
///
/// The repository is held as `Arc<dyn VehicleOwnershipRepository>` for two
/// reasons. `dyn` keeps the handler independent of which adapter is wired in,
/// so production and tests share the same code path. `Arc` allows one adapter
/// instance — and therefore one connection pool and one shared store — to be
/// used by many concurrent request tasks; each task holds a cheap refcount
/// clone rather than its own copy of the storage.
///
/// Обработчик сценария «начать владение автомобилем».
///
/// Репозиторий хранится как `Arc<dyn VehicleOwnershipRepository>` по двум
/// причинам. `dyn` делает обработчик независимым от того, какой адаптер
/// подключён, поэтому продакшен и тесты используют один и тот же путь
/// исполнения. `Arc` позволяет одному экземпляру адаптера — а значит, одному
/// пулу соединений и одному общему хранилищу — обслуживать множество
/// конкурентных задач-запросов; каждая задача держит дешёвую копию счётчика
/// ссылок, а не собственную копию хранилища.
pub struct StartVehicleOwnershipHandler {
    repository: Arc<dyn VehicleOwnershipRepository>,
}

impl StartVehicleOwnershipHandler {
    /// Builds the handler around a repository adapter.
    ///
    /// Создаёт обработчик поверх адаптера репозитория.
    pub fn new(repository: Arc<dyn VehicleOwnershipRepository>) -> Self {
        Self { repository }
    }

    /// Executes [`StartVehicleOwnershipCommand`].
    ///
    /// The sequence matters. The handler first asks the repository whether the
    /// vehicle is already occupied, then packages that answer as an
    /// [`OwnershipEligibilitySnapshot`] and hands it to the aggregate. The
    /// handler never inspects the flag itself — deciding what an occupied
    /// vehicle means is a business rule, and it stays in the domain. This is
    /// the layer's whole shape in miniature: fetch the facts the domain cannot
    /// reach, let the domain judge, persist the verdict.
    ///
    /// Note the read-then-write gap: two concurrent calls can both observe an
    /// unoccupied vehicle before either saves. The domain check is therefore a
    /// fast, precise rejection of the common case, not the ultimate guarantee —
    /// a partial unique index in the database is what finally enforces the rule
    /// under concurrency.
    ///
    /// `now` is read once here rather than inside the aggregate, so that every
    /// event produced by this command shares a single timestamp and the domain
    /// remains free of ambient clock access.
    ///
    /// Выполняет [`StartVehicleOwnershipCommand`].
    ///
    /// Порядок действий существенен. Обработчик сначала спрашивает у
    /// репозитория, занят ли автомобиль, затем упаковывает ответ в
    /// [`OwnershipEligibilitySnapshot`] и передаёт агрегату. Сам обработчик
    /// никогда не анализирует этот флаг: решение о том, что означает занятый
    /// автомобиль, — бизнес-правило, и оно остаётся в домене. Это вся суть слоя
    /// в миниатюре: получить факты, недоступные домену, дать домену вынести
    /// решение, сохранить вердикт.
    ///
    /// Обратите внимание на промежуток между чтением и записью: два
    /// конкурентных вызова могут увидеть незанятый автомобиль раньше, чем любой
    /// из них выполнит сохранение. Поэтому доменная проверка — это быстрое и
    /// точное отклонение типичного случая, а не окончательная гарантия;
    /// окончательно правило при конкурентном доступе обеспечивает частичный
    /// уникальный индекс в базе данных.
    ///
    /// `now` считывается здесь один раз, а не внутри агрегата, чтобы все
    /// события этой команды имели единую временную метку, а домен оставался
    /// свободным от неявного обращения к часам.
    pub async fn handle(&self, cmd: StartVehicleOwnershipCommand) -> Result<(), ApplicationError> {
        let now = Utc::now();
        let has_active_ownership = self.repository.has_open_ownership(cmd.vehicle_id).await?;
        let snapshot = OwnershipEligibilitySnapshot::new(cmd.vehicle_id, has_active_ownership);
        let ownership = VehicleOwnership::start(
            cmd.ownership_id,
            cmd.vehicle_id,
            cmd.owner_customer_id,
            cmd.ownership_type,
            snapshot,
            now,
        )?;

        self.repository.save(&ownership).await?;

        Ok(())
    }
}
