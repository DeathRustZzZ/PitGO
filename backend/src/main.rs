//! PitGO backend — the HTTP entry point.
//!
//! This binary is the composition root: the one place that chooses which
//! adapters implement the application's ports, builds the shared state and
//! mounts the routes. Every other layer is written against traits, so swapping
//! the in-memory repositories for PostgreSQL ones is a change confined to this
//! file.
//!
//! Responsibilities stop at transport. Routes translate HTTP into commands and
//! application results back into responses; business rules live in `domain` and
//! orchestration in `application`.
//!
//! Бэкенд PitGO — точка входа HTTP.
//!
//! Этот бинарный крейт является корнем композиции: единственным местом, где
//! выбирается, какие адаптеры реализуют порты приложения, где собирается общее
//! состояние и подключаются маршруты. Все остальные слои написаны в терминах
//! трейтов, поэтому замена репозиториев в памяти на PostgreSQL затрагивает
//! только этот файл.
//!
//! Ответственность ограничена транспортом. Маршруты преобразуют HTTP в команды
//! и результаты приложения обратно в ответы; бизнес-правила находятся в
//! `domain`, а оркестрация — в `application`.

use application::{
    customer::ports::CustomerRepository, ownership::ports::VehicleOwnershipRepository,
    vehicle::ports::VehicleRepository,
};
use axum::{
    Router,
    http::{HeaderValue, Method, header::CONTENT_TYPE},
    routing::{get, post},
};
use infrastructure::customer_repository::InMemoryCustomerRepository;
use infrastructure::vehicle_ownership_repository::InMemoryVehicleOwnershipRepository;
use infrastructure::vehicle_repository::InMemoryVehicleRepository;
mod error;
mod routers;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;

/// Shared application state injected into every handler.
///
/// Holds the repositories as `Arc<dyn Trait>` for two distinct reasons. `dyn`
/// means handlers depend on the port, not on a concrete adapter, so the whole
/// stack above this file is unaware of which storage is in use. `Arc` means the
/// `Clone` below is a refcount bump rather than a copy of the data: axum clones
/// this state for every request, so the repositories — and, later, the database
/// connection pool inside them — must be shared, not duplicated.
///
/// Общее состояние приложения, внедряемое в каждый обработчик.
///
/// Хранит репозитории как `Arc<dyn Trait>` по двум разным причинам. `dyn`
/// означает, что обработчики зависят от порта, а не от конкретного адаптера,
/// поэтому весь стек выше этого файла не знает, какое хранилище используется.
/// `Arc` означает, что `Clone` ниже — это увеличение счётчика ссылок, а не
/// копирование данных: axum клонирует это состояние на каждый запрос, поэтому
/// репозитории — а в дальнейшем и находящийся в них пул соединений с базой
/// данных — должны разделяться, а не дублироваться.
#[derive(Clone)]
struct AppState {
    customer_repository: Arc<dyn CustomerRepository>,
    vehicle_repository: Arc<dyn VehicleRepository>,
    vehicle_ownership_repository: Arc<dyn VehicleOwnershipRepository>,
}

/// Starts the HTTP server.
///
/// `#[tokio::main]` sets up the multi-threaded runtime the async repository
/// ports and axum both require: handlers await I/O, and the runtime keeps
/// worker threads serving other requests while they do.
///
/// Запускает HTTP-сервер.
///
/// `#[tokio::main]` разворачивает многопоточный рантайм, необходимый и
/// асинхронным портам репозиториев, и axum: обработчики ожидают ввод-вывод, а
/// рантайм в это время продолжает обслуживать другие запросы на рабочих
/// потоках.
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Scoped to the Vite dev server origin, with only the methods and header
    // the API actually uses. Deliberately not permissive: a wildcard origin
    // would have to be revisited before any deployment, and it is easier to
    // widen this later than to notice it was never narrowed.
    //
    // Ограничено origin'ом dev-сервера Vite, только теми методами и тем
    // заголовком, которые API действительно использует. Намеренно не
    // разрешительная настройка: подстановочный origin пришлось бы
    // пересматривать перед развёртыванием, а расширить это правило позже проще,
    // чем заметить, что его так и не сузили.
    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_static("http://localhost:5173"))
        .allow_headers([CONTENT_TYPE])
        .allow_methods([Method::GET, Method::POST]);

    // Composition root: the only place that names concrete adapters. Note the
    // repositories are in-memory, so all state is lost on restart.
    //
    // Корень композиции: единственное место, где называются конкретные
    // адаптеры. Обратите внимание, что репозитории работают в памяти, поэтому
    // всё состояние теряется при перезапуске.
    let state = AppState {
        customer_repository: Arc::new(InMemoryCustomerRepository::new()),
        vehicle_repository: Arc::new(InMemoryVehicleRepository::new()),
        vehicle_ownership_repository: Arc::new(InMemoryVehicleOwnershipRepository::new()),
    };

    // Ownerships are nested under their vehicle because an ownership record has
    // no meaning without one — the vehicle is part of its identity, not a
    // filter over a flat collection.
    //
    // Владения вложены в маршрут автомобиля, поскольку запись о владении не
    // имеет смысла без него: автомобиль входит в её идентичность, а не является
    // фильтром по плоской коллекции.
    let app = Router::new()
        .route("/health", get(health))
        .route("/customers", post(routers::customer::create_customer))
        .route("/customers/{id}", get(routers::customer::get_customer))
        .route("/vehicles", post(routers::vehicle::create_vehicle))
        .route("/vehicles/{id}", get(routers::vehicle::get_vehicle))
        .route(
            "/vehicles/{vehicle_id}/ownerships",
            post(routers::vehicle_ownership::create_vehicle_ownership),
        )
        .layer(cors)
        .with_state(state);

    // Bound to loopback only: this server has no authentication yet, so it must
    // not be reachable from outside the machine.
    //
    // Привязка только к loopback: у сервера пока нет аутентификации, поэтому он
    // не должен быть доступен извне машины.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    tracing::info!("Backend listening on {}", addr);

    // Startup failures are fatal by design: a server that cannot bind its port
    // or accept connections has nothing useful to do, and failing loudly at
    // boot is preferable to running in a silently broken state.
    //
    // Сбои при запуске намеренно фатальны: серверу, который не может занять
    // порт или принимать соединения, нечего делать, и громкий отказ при старте
    // предпочтительнее работы в незаметно сломанном состоянии.
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Faild to bind TCP listener");

    axum::serve(listener, app).await.expect("Server failed");
}

/// Handles `GET /health`.
///
/// A liveness probe for deployment tooling: it reports that the process is up
/// and serving, and deliberately checks no dependencies — a readiness endpoint
/// that probes storage is a separate concern.
///
/// Обрабатывает `GET /health`.
///
/// Проба живости для средств развёртывания: сообщает, что процесс запущен и
/// обслуживает запросы, и намеренно не проверяет зависимости — эндпоинт
/// готовности, опрашивающий хранилище, является отдельной задачей.
async fn health() -> &'static str {
    "OK"
}
