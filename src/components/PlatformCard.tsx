import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import ScheduleModal from "./ScheduleModal";

export interface Platform {
  id: number;
  name: string;
  domains: string[];
  enabled: boolean;
  currently_blocked: boolean;
}

export interface Schedule {
  id: number;
  platform_id: number;
  day_of_week: number;
  start_time: string;
  end_time: string;
}

interface Props {
  platform: Platform;
  onUpdate: () => void;
}

const PLATFORM_META: Record<string, { icon: string; color: string }> = {
  TikTok:    { icon: "🎵", color: "#ff0050" },
  Facebook:  { icon: "📘", color: "#1877f2" },
  Instagram: { icon: "📸", color: "#e1306c" },
  YouTube:   { icon: "▶️", color: "#ff0000" },
};

const DAY_NAMES = ["Lun", "Mar", "Mié", "Jue", "Vie", "Sáb", "Dom"];

export default function PlatformCard({ platform, onUpdate }: Props) {
  const [showModal, setShowModal] = useState(false);
  const [schedules, setSchedules] = useState<Schedule[]>([]);
  const [loading, setLoading] = useState(false);

  const meta = PLATFORM_META[platform.name] ?? { icon: "🌐", color: "#888" };

  async function handleToggle() {
    setLoading(true);
    try {
      await invoke("toggle_platform", {
        id: platform.id,
        enabled: !platform.enabled,
      });
      onUpdate();
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  }

  async function openModal() {
    const data = await invoke<Schedule[]>("get_schedules", {
      platformId: platform.id,
    });
    setSchedules(data);
    setShowModal(true);
  }

  function statusLabel() {
    if (!platform.enabled) return { text: "Desactivado", cls: "status-off" };
    if (platform.currently_blocked)
      return { text: "Bloqueado ahora", cls: "status-blocked" };
    return { text: "Permitido ahora", cls: "status-allowed" };
  }

  const status = statusLabel();

  return (
    <>
      <div className={`platform-card ${platform.enabled ? "card-active" : ""}`}>
        <div className="card-header">
          <div className="platform-icon" style={{ color: meta.color }}>
            {meta.icon}
          </div>
          <div className="platform-info">
            <h3 className="platform-name">{platform.name}</h3>
            <span className={`status-badge ${status.cls}`}>{status.text}</span>
          </div>
          <label className="toggle-switch">
            <input
              type="checkbox"
              checked={platform.enabled}
              onChange={handleToggle}
              disabled={loading}
            />
            <span className="slider" style={{ "--accent": meta.color } as React.CSSProperties} />
          </label>
        </div>

        {platform.enabled && (
          <div className="card-body">
            <p className="schedule-hint">
              {schedules.length === 0
                ? "Sin horarios → bloqueado 24/7"
                : `${schedules.length} ventana(s) permitida(s)`}
            </p>
            <button className="btn-schedule" onClick={openModal}>
              Gestionar horarios
            </button>
          </div>
        )}
      </div>

      {showModal && (
        <ScheduleModal
          platform={platform}
          schedules={schedules}
          accentColor={meta.color}
          onClose={() => setShowModal(false)}
          onUpdate={async () => {
            const data = await invoke<Schedule[]>("get_schedules", {
              platformId: platform.id,
            });
            setSchedules(data);
            onUpdate();
          }}
        />
      )}
    </>
  );
}

export { DAY_NAMES };
