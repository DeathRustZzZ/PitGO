# Диаграммы Customer Domain

## Source of Truth

Единственный источник истины для Mermaid-модели Customer Domain:

- `customer.md`

Файлы в `diagrams/` нельзя рассматривать как самостоятельные модели, пока
исходная схема явно не разделена. Они существуют только как физическое
разбиение и навигационная структура вокруг исходной схемы.

## Как пользоваться файлами

1. При проверке модели сначала читать `customer.md`.
2. Использовать `diagrams/00-overview.mmd` как навигационную точку входа.
3. Использовать детальные `diagrams/*.mmd` файлы как читаемые срезы исходной
   большой схемы.
4. При дальнейшем разбиении переносить содержимое из `customer.md` без
   переименования классов, изменения связей, изменения DDD-границ или изменения
   инвариантов.

## Обзорные диаграммы

- `diagrams/00-overview.mmd` - навигационная карта набора диаграмм.

## Детальные диаграммы

- `diagrams/01-customer-lifecycle.mmd` - область lifecycle-агрегата Customer.
- `diagrams/02-contact-book.mmd` - область агрегата Customer contact book.
- `diagrams/03-profile.mmd` - область агрегата Customer profile.
- `diagrams/04-preferences.mmd` - область агрегата Customer preferences.
- `diagrams/05-consent-ledger.mmd` - область Customer consent ledger.
- `diagrams/06-eventing-audit.mmd` - область eventing и audit context.
- `diagrams/07-activation-flow.mmd` - область activation policy и permit.
- `diagrams/08-identity-registry.mmd` - область bounded context Identity Registry.
- `diagrams/09-application-consistency.mmd` - область application orchestration,
  repositories, unit of work, optimistic locking и outbox.
- `diagrams/10-access-auth-readside.mmd` - область shared result types, action
  context, authorization context, auth reference, read-side projections и
  dispatch policy.

## Текущий статус

Эти файлы не заменяют, не упрощают и не переопределяют исходную Mermaid-схему в
`customer.md`.
