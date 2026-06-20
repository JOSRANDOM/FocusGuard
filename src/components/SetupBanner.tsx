import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  onDone: () => void;
}

export default function SetupBanner({ onDone }: Props) {
  const [installing, setInstalling] = useState(false);
  const [error, setError] = useState("");

  async function handleInstall() {
    setInstalling(true);
    setError("");
    try {
      await invoke("install_helper");
      onDone();
    } catch (e) {
      setError(String(e));
    } finally {
      setInstalling(false);
    }
  }

  return (
    <div className="setup-banner">
      <div className="setup-content">
        <span className="setup-icon">🔧</span>
        <div className="setup-text">
          <strong>Configuración inicial requerida</strong>
          <p>
            FocusGuard necesita instalar un helper de sistema para aplicar
            bloqueos <em>sin pedirte contraseña cada vez</em>. Solo se pide
            <strong> una única vez</strong>.
          </p>
          {error && <p className="setup-error">{error}</p>}
        </div>
        <button
          className="btn-setup"
          onClick={handleInstall}
          disabled={installing}
        >
          {installing ? "Instalando…" : "Configurar ahora"}
        </button>
      </div>
    </div>
  );
}
