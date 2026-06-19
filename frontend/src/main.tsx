import React from "react";
import ReactDOM from "react-dom/client";
import App from "./app/App";
import { initPlatform } from "./shared/platform/telegram";
import "./shared/styles/tokens.css";
import "./shared/styles/global.css";

// Инициализируем платформу до рендера: внутри Telegram это применит тему
// и развернёт окно; в обычном вебе — no-op.
initPlatform();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
