# Customer Domain diagram style guide

Customer Domain является эталонным доменом PitGO для Mermaid-схем. Визуальная система нужна не для украшения, а для быстрого чтения модели без изменения архитектуры, имен классов или доменных сущностей.

## Совместимость с Mermaid to Excalidraw

Файлы `docs/domain/customer/diagrams/*.mmd` должны оставаться совместимыми с Obsidian Excalidraw Mermaid import.

Поэтому в `.mmd` намеренно не используются групповые Mermaid style directives:

- `classDef`
- `cssClass`
- `class SomeNode someStyle`

Эти директивы корректны для обычного Mermaid-рендера, но часто ломают `Mermaid to Excalidraw`, особенно для `classDiagram`.

Для окраски используется только прямой синтаксис:

```mermaid
style Customer fill:#1D4ED8,stroke:#1E3A8A,stroke-width:3px,color:#FFFFFF
```

Такой формат не вводит дополнительные CSS-классы и лучше переносится через `Mermaid to Excalidraw`.

`Full.mmd` дополнительно использует `namespace` только как визуальные зоны. Это не доменные сущности и не изменение модели; зоны нужны, чтобы Excalidraw import не превращал полный граф в одну нечитаемую массу.

## Найденные категории

- Customer Aggregate Roots
- Entities
- Value Objects
- State Machines / Status
- Domain Events
- Errors
- Policies
- Invariants
- External References

## Цветовая легенда

| Категория | Визуальный стиль | Что означает |
| --- | --- | --- |
| Customer Aggregate Roots | Насыщенный синий фон, толстая рамка | Основные границы консистентности, которыми владеет Customer Domain. |
| Entities | Светло-синий фон, обычная рамка | Доменные объекты с идентичностью, которые не являются основными Customer aggregate roots. |
| Value Objects | Светло-зеленый фон, пунктирная рамка | Неизменяемые или валидируемые значения внутри агрегатов и политик. |
| State Machines / Status | Светло-фиолетовый фон | Enum/status/lifecycle-концепции, описывающие допустимые состояния. |
| Domain Events | Светло-желтый фон | События и pending event records, которые поднимаются агрегатами или поддерживающими domain services. |
| Errors | Светло-красный фон | Domain, policy и application error enums, видимые в модели. |
| Policies | Светло-оранжевый фон | Domain policies и domain services, принимающие решения, особенно между агрегатами. |
| Invariants | Светло-розовый фон | Локальные и cross-aggregate правила, защищающие корректные переходы состояния. |
| External References | Светло-серый фон, пунктирная рамка | Application services, repositories, projections, auth/access, outbox, соседние bounded contexts и обзорные узлы. |

## Примеры из Customer Domain

- Aggregate Roots: `Customer`, `CustomerContactBook`, `CustomerProfile`, `CustomerPreferences`, `CustomerConsentLedger`.
- Entities: `IdentityRegistryEntry` в срезе Identity Registry.
- Value Objects: `ActivationPermit`, `PhoneContact`, `EmailContact`, `TelegramContact`, `PrimaryContact`, `CustomerLocation`, `QuietHours`, `IdentityReservationProof`.
- State Machines / Status: `CustomerStatus`, `VerificationStatus`, `EffectiveConsent`, `IdentityRegistryStatus`, `IdentityReservationStatus`.
- Domain Events: `CustomerEvent`, `ContactEvent`, `PreferencesEvent`, `PendingCustomerEvent`, `PendingContactEvent`, `PendingPreferencesEvent`, `IdentityRegistryEvent`.
- Errors: `CustomerError`, `CustomerActivationError`, `ContactError`, `PreferencesError`, `EligibilityError`, `IdentityRegistryError`.
- Policies: `CustomerActivationPolicy`, `ContactLifecyclePolicy`, `NotificationDispatchPolicy`.
- Invariants: `CustomerInvariants`, `ContactBookInvariants`, `ActivationInvariants`, `EventInvariants`, `OutboxInvariants`.
- External References: `CustomerActivationApplicationService`, `CustomerContactApplicationService`, repositories, `UnitOfWork`, `AccessContext`, `AuthContext`, read-side projections и соседний `IdentityRegistry`.

## Как читать схему сверху вниз

Начинайте с насыщенных синих узлов. Они показывают, какой aggregate root владеет состоянием и какие команды могут его менять.

Затем идите по composition-связям от агрегата к зеленым Value Objects и фиолетовым статусам. Так читается внутреннее устройство агрегата и его lifecycle.

После этого смотрите розовые invariants и оранжевые policies. Они объясняют, почему команда разрешается, отклоняется или требует cross-aggregate решения.

Дальше переходите к желтым event-узлам. Они показывают, что домен записывает после успешного изменения и что могут потреблять projections или audit views.

Серые пунктирные узлы читайте последними. Это важные элементы orchestration, persistence, authorization, read-side или соседних bounded contexts, но они не являются внутренностями Customer aggregate.
