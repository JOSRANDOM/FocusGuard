import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import PlatformCard, { Platform } from "./components/PlatformCard";
import GlobalSchedule from "./components/GlobalSchedule";
import SecuritySettings from "./components/SecuritySettings";
import PrivateRelayBanner from "./components/PrivateRelayBanner";
import SetupBanner from "./components/SetupBanner";
import UpdateBanner from "./components/UpdateBanner";
import "./App.css";

type Tab = "general" | "individual" | "security";

export default function App() {
  const [tab, setTab] = useState<Tab>("general");
  const [platforms, setPlatforms] = useState<Platform[]>([]);
  const [loading, setLoading] = useState(true);
  const [privateRelayOn, setPrivateRelayOn] = useState(false);
  const [helperInstalled, setHelperInstalled] = useState(true);
  const [securityEnabled, setSecurityEnabled] = useState(false);

  async function loadPlatforms() {
    try {
      const data = await invoke<Platform[]>("get_platforms");
      setPlatforms(data);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  }

  async function checkRelay() {
    try {
      const active = await invoke<boolean>("check_private_relay");
      setPrivateRelayOn(active);
    } catch {
      setPrivateRelayOn(false);
    }
  }

  async function checkHelper() {
    try {
      const installed = await invoke<boolean>("check_helper_installed");
      setHelperInstalled(installed);
    } catch {
      setHelperInstalled(true);
    }
  }

  async function checkSecurity() {
    try {
      const enabled = await invoke<boolean>("get_security_status");
      setSecurityEnabled(enabled);
    } catch {
      setSecurityEnabled(false);
    }
  }

  useEffect(() => {
    loadPlatforms();
    checkRelay();
    checkHelper();
    checkSecurity();
    const interval = setInterval(() => {
      loadPlatforms();
      checkRelay();
    }, 60_000);
    return () => clearInterval(interval);
  }, []);

  const activeCount = platforms.filter((p) => p.enabled).length;
  const requirePasswordSetup = () => setTab("security");

  return (
    <div className="app">
      <header className="app-header">
        <div className="header-title">
          <span className="header-icon">🛡️</span>
          <div>
            <h1>FocusGuard</h1>
            <p className="header-sub">Control de acceso a redes sociales</p>
          </div>
        </div>
      </header>

      <UpdateBanner />
      {!helperInstalled && <SetupBanner onDone={() => setHelperInstalled(true)} />}
      {privateRelayOn && <PrivateRelayBanner />}

      <div className="tabs">
        <button
          className={`tab-btn ${tab === "general" ? "tab-active" : ""}`}
          onClick={() => setTab("general")}
        >
          🌐 Bloqueo general por horario
        </button>
        <button
          className={`tab-btn ${tab === "individual" ? "tab-active" : ""}`}
          onClick={() => setTab("individual")}
        >
          🔒 Bloqueo individual
        </button>
        <button
          className={`tab-btn ${tab === "security" ? "tab-active" : ""}`}
          onClick={() => setTab("security")}
        >
          🔐 Seguridad
        </button>
      </div>

      <main className="app-main">
        {loading ? (
          <div className="loading">Cargando…</div>
        ) : tab === "general" ? (
          <GlobalSchedule
            securityEnabled={securityEnabled}
            onRequirePasswordSetup={requirePasswordSetup}
          />
        ) : tab === "security" ? (
          <SecuritySettings onStatusChange={setSecurityEnabled} />
        ) : (
          <>
            <div className="stats-bar">
              <span>
                <strong>{activeCount}</strong> de {platforms.length} plataformas protegidas
              </span>
              <span className="stats-hint">
                Los horarios definen cuándo se <em>permite</em> el acceso
              </span>
            </div>
            <div className="platform-grid">
              {platforms.map((p) => (
                <PlatformCard
                  key={p.id}
                  platform={p}
                  securityEnabled={securityEnabled}
                  onUpdate={loadPlatforms}
                  onRequirePasswordSetup={requirePasswordSetup}
                />
              ))}
            </div>
          </>
        )}
      </main>

      <footer className="app-footer">
        FocusGuard · actualización automática cada 60 s
      </footer>
    </div>
  );
}
