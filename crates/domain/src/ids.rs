// Типобезопасные идентификаторы для различных сущностей в домене.

use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientId(Uuid);

impl ClientId {
    /// Cоздает новый уникальный идентификатор клиента.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
