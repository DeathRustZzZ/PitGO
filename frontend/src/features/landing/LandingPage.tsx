import { Link, useNavigate } from "react-router-dom";
import { Button, Logo } from "../../shared/ui";
import styles from "./LandingPage.module.css";

// Простые inline-иконки (stroke, currentColor) — без внешних зависимостей.
function CalendarIcon() {
  return (
    <svg
      width="22"
      height="22"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <rect x="3" y="4.5" width="18" height="16" rx="2.5" />
      <path d="M3 9h18M8 2.5v4M16 2.5v4" />
    </svg>
  );
}

function HistoryIcon() {
  return (
    <svg
      width="22"
      height="22"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M3.5 12a8.5 8.5 0 1 0 2.6-6.1" />
      <path d="M5 3v3.5h3.5" />
      <path d="M12 7.5V12l3 1.8" />
    </svg>
  );
}

function ShieldIcon() {
  return (
    <svg
      width="22"
      height="22"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M12 3l7 3v5c0 4.5-3 8-7 9-4-1-7-4.5-7-9V6l7-3z" />
      <path d="M9 12l2 2 4-4" />
    </svg>
  );
}

const FEATURES = [
  {
    icon: <CalendarIcon />,
    title: "Запись онлайн",
    text: "Клиент находит свободное окно в подходящем автосервисе и записывается за пару касаний.",
  },
  {
    icon: <HistoryIcon />,
    title: "История обслуживания",
    text: "Все работы по автомобилю собираются в одну проверяемую историю — она остаётся с машиной.",
  },
  {
    icon: <ShieldIcon />,
    title: "Проверенные СТО",
    text: "Сервисы получают заявки и управляют заказами в одном месте, клиенты — прозрачность и доверие.",
  },
];

export function LandingPage() {
  const navigate = useNavigate();
  const goRegister = () => navigate("/register");

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div className={[styles.container, styles.headerInner].join(" ")}>
          <Link to="/" className={styles.logo} aria-label="PitGO — на главную">
            <Logo />
          </Link>

          <nav className={styles.nav}>
            <a href="#features">Возможности</a>
            <a href="#garages">Для СТО</a>
            <a href="#clients">Клиенты</a>
          </nav>

          <div className={styles.actions}>
            <Button
              className={styles.loginAction}
              variant="ghost"
              size="sm"
              onClick={goRegister}
            >
              Войти
            </Button>
            <Button size="sm" onClick={goRegister}>
              Попробовать
            </Button>
          </div>
        </div>
      </header>

      <main>
        {/* Hero */}
        <section className={styles.hero}>
          <div className={[styles.container, styles.heroInner].join(" ")}>
            <span className={styles.eyebrow}>
              <span className={styles.eyebrowDot} />
              Платформа для СТО и автовладельцев
            </span>
            <h1 className={styles.heroTitle}>
              Умная платформа для СТО и клиентов
            </h1>
            <p className={styles.heroText}>
              PitGO помогает клиентам быстро находить автосервисы и записываться
              на обслуживание, а СТО — получать заявки и вести заказы в одном
              месте.
            </p>
            <div className={styles.heroActions}>
              <Button size="md" onClick={goRegister}>
                Начать бесплатно
              </Button>
              <Button variant="secondary" size="md" onClick={goRegister}>
                Для автосервисов
              </Button>
            </div>
            <p className={styles.trust}>
              Запись, история обслуживания и заявки — без звонков и блокнотов.
            </p>

            {/* Декоративное превью продукта */}
            <div className={styles.preview} aria-hidden="true">
              <div className={styles.previewRow}>
                <span className={styles.previewIcon}>
                  <CalendarIcon />
                </span>
                <div className={styles.previewMain}>
                  <span className={styles.previewTitle}>Замена масла</span>
                  <span className={styles.previewMeta}>
                    СТО «Гараж №7» · завтра, 10:30
                  </span>
                </div>
                <span className={styles.previewBadge}>Подтверждено</span>
              </div>
              <div className={styles.previewRow}>
                <span className={styles.previewIcon}>
                  <HistoryIcon />
                </span>
                <div className={styles.previewMain}>
                  <span className={styles.previewTitle}>
                    Диагностика подвески
                  </span>
                  <span className={styles.previewMeta}>
                    Toyota Camry · 12 марта
                  </span>
                </div>
                <span className={styles.previewBadge}>Выполнено</span>
              </div>
            </div>
          </div>
        </section>

        {/* Features */}
        <section id="features" className={styles.section}>
          <div className={styles.container}>
            <div className={styles.sectionHead}>
              <h2 className={styles.sectionTitle}>
                Всё для обслуживания автомобиля
              </h2>
              <p className={styles.sectionSubtitle}>
                От поиска сервиса до проверяемой истории работ — в одном
                приложении для веба и Telegram.
              </p>
            </div>

            <div className={styles.featureGrid}>
              {FEATURES.map((f) => (
                <article key={f.title} className={styles.feature}>
                  <span className={styles.featureIcon}>{f.icon}</span>
                  <h3 className={styles.featureTitle}>{f.title}</h3>
                  <p className={styles.featureText}>{f.text}</p>
                </article>
              ))}
            </div>
          </div>
        </section>

        {/* CTA */}
        <section className={styles.ctaBand}>
          <div className={styles.container}>
            <div className={styles.ctaInner}>
              <h2 className={styles.ctaTitle}>Готовы попробовать PitGO?</h2>
              <p className={styles.ctaText}>
                Создайте аккаунт за минуту и запишитесь в автосервис уже
                сегодня.
              </p>
              <Button variant="secondary" size="md" onClick={goRegister}>
                Создать аккаунт
              </Button>
            </div>
          </div>
        </section>
      </main>

      <footer className={styles.footer}>
        <div className={[styles.container, styles.footerInner].join(" ")}>
          <span className={styles.footerBrand}>
            <Logo variant="mark" size={26} />© {new Date().getFullYear()} PitGO
          </span>
          <nav className={styles.footerLinks}>
            <a href="#features">Возможности</a>
            <a href="#garages">Для СТО</a>
            <a href="#clients">Клиенты</a>
          </nav>
        </div>
      </footer>
    </div>
  );
}
