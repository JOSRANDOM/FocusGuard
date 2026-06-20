import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Platform, Schedule, DAY_NAMES } from "./PlatformCard";

interface Props {
  platform: Platform;
  schedules: Schedule[];
  accentColor: string;
  onClose: () => void;
  onUpdate: () => void;
}

export default function ScheduleModal({
  platform,
  schedules,
  accentColor,
  onClose,
  onUpdate,
}: Props) {
  const [dayOfWeek, setDayOfWeek] = useState(7);
  const [startTime, setStartTime] = useState("09:00");
  const [endTime, setEndTime] = useState("17:00");
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);

  const DAY_OPTIONS = [{ value: 7, label: "Todos los días" }, ...DAY_NAMES.map((d, i) => ({ value: i, label: d }))];

  function dayLabel(day: number) {
    return day === 7 ? "Todos los días" : DAY_NAMES[day];
  }

  async function handleAdd() {
    if (startTime >= endTime) {
      setError("La hora de inicio debe ser anterior a la de fin.");
      return;
    }
    setError("");
    setSaving(true);
    try {
      await invoke("add_schedule", {
        schedule: {
          platform_id: platform.id,
          day_of_week: dayOfWeek,
          start_time: startTime,
          end_time: endTime,
        },
      });
      await onUpdate();
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete(id: number) {
    try {
      await invoke("delete_schedule", { id });
      await onUpdate();
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header" style={{ borderColor: accentColor }}>
          <h2>Horarios permitidos — {platform.name}</h2>
          <button className="btn-close" onClick={onClose}>✕</button>
        </div>

        <p className="modal-hint">
          Define cuándo está <strong>permitido</strong> usar esta plataforma. Fuera de estos
          horarios quedará bloqueada.
        </p>

        {schedules.length === 0 ? (
          <p className="empty-schedules">Sin horarios — bloqueado las 24 hs.</p>
        ) : (
          <ul className="schedule-list">
            {schedules.map((s) => (
              <li key={s.id} className="schedule-item">
                <span className="schedule-day">{dayLabel(s.day_of_week)}</span>
                <span className="schedule-time">
                  {s.start_time} – {s.end_time}
                </span>
                <button
                  className="btn-delete"
                  onClick={() => handleDelete(s.id)}
                  title="Eliminar"
                >
                  🗑
                </button>
              </li>
            ))}
          </ul>
        )}

        <div className="modal-form">
          <h3>Agregar ventana</h3>
          <div className="form-row">
            <label>Día</label>
            <select value={dayOfWeek} onChange={(e) => setDayOfWeek(Number(e.target.value))}>
              {DAY_OPTIONS.map((d) => (
                <option key={d.value} value={d.value}>
                  {d.label}
                </option>
              ))}
            </select>
          </div>
          <div className="form-row">
            <label>Desde</label>
            <input
              type="time"
              value={startTime}
              onChange={(e) => setStartTime(e.target.value)}
            />
          </div>
          <div className="form-row">
            <label>Hasta</label>
            <input
              type="time"
              value={endTime}
              onChange={(e) => setEndTime(e.target.value)}
            />
          </div>
          {error && <p className="form-error">{error}</p>}
          <button
            className="btn-add"
            style={{ background: accentColor }}
            onClick={handleAdd}
            disabled={saving}
          >
            {saving ? "Guardando…" : "Agregar horario"}
          </button>
        </div>
      </div>
    </div>
  );
}
