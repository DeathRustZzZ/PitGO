export default function App() {
  return (
    <div className="app">
      <header className="header">
        <div className="container header__container">
          <a href="#" className="logo">
            <span className="logo__mark">P</span>
            <span className="logo__text">PitGo</span>
          </a>

          <nav className="nav">
            <a href="#features">Возможности</a>
            <a href="#garages">Для СТО</a>
            <a href="#clients">Клиенты</a>
            <a href="#pricing">Цены</a>
          </nav>

          <div className="actions">
            <a href="#" className="login">
              Войти
            </a>
            <a href="#" className="signup">
              Попробовать
            </a>
          </div>
        </div>
      </header>

      <main>
        <section className="hero">
          <div className="container">
            <h1>Умная платформа для СТО и клиентов</h1>
            <p>
              PitGo помогает клиентам быстро находить автосервисы, а СТО —
              получать заявки и управлять заказами.
            </p>

            <a href="#" className="hero__button">
              Начать
            </a>
          </div>
        </section>
      </main>
    </div>
  );
}
