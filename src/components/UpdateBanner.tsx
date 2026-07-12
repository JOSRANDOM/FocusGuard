import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function UpdateBanner() {
  const [update, setUpdate] = useState<Awaited<
    ReturnType<typeof import("@tauri-apps/plugin-updater").check>
  > | null>(null);
  const [installing, setInstalling] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    (async () => {
      try {
        const isWindows = await invoke<boolean>("is_windows");
        if (!isWindows) return;

        const { check } = await import("@tauri-apps/plugin-updater");
        const result = await check();
        if (result?.available) setUpdate(result);
      } catch (e) {
        console.error("No se pudo comprobar actualizaciones:", e);
      }
    })();
  }, []);

  async function handleInstall() {
    if (!update) return;
    setInstalling(true);
    setError("");
    try {
      await update.downloadAndInstall();
      const { relaunch } = await import("@tauri-apps/plugin-process");
      await relaunch();
    } catch (e) {
      setError(String(e));
      setInstalling(false);
    }
  }

  if (!update) return null;

  return (
    <div className="update-banner">
      <div className="update-banner-content">
        <span className="update-icon">⬆️</span>
        <div className="update-text">
          <strong>Nueva versión disponible: {update.version}</strong>
          <p>Actualiza para tener las últimas mejoras y correcciones.</p>
          {error && <p className="update-error">{error}</p>}
        </div>
        <button
          className="btn-update"
          onClick={handleInstall}
          disabled={installing}
        >
          {installing ? "Instalando…" : "Actualizar ahora"}
        </button>
      </div>
    </div>
  );
}
