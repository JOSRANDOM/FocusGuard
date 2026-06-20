import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface GlobalSchedule {
  id: number;
  days: number[];
  start_time: string;
  end_time: string;
  platforms: number[];
}

interface Platform {
  id: number;
  name: string;
  enabled: boolean;
}

const DAY_LABELS = ["Lun", "Mar", "Mié", "Jue", "Vie", "Sáb", "Dom"];

const PLATFORM_META: Record<string, { icon: string; color: string }> = {
  TikTok:    { icon: "🎵", color: "#ff0050" },
  Facebook:  { icon: "📘", color: "#1877f2" },
  Instagram: { icon: "📸", color: "#e1306c" },
  YouTube:   { icon: "▶️", color: "#ff0000" },
};

function formatDays(days: number[]): string {
  if (days.length === 7) return "Todos los días";
  if (JSON.stringify([...days].sort()) === JSON.stringify([0,1,2,3,4])) return "Lun – Vie";
  if (JSON.stringify([...days].sort()) === JSON.stringify([5,6])) return "Sáb – Dom";
  return days.map((d) => DAY_LABELS[d]).join(", ");
}

function formatPlatforms(platformIds: number[], all: Platform[]): string {
  if (platformIds.length === 0) return "Todas las plataformas";
  return platformIds
    .map((id) => all.find((p) => p.id === id)?.name ?? `#${id}`)
    .join(", ");
}

export default function GlobalSchedule() {
  const [schedules, setSchedules] = useState<GlobalSchedule[]>([]);
  const [allPlatforms, setAllPlatforms] = useState<Platform[]>([]);
  const [selectedDays, setSelectedDays] = useState<number[]>([]);
  const [selectedPlatforms, setSelectedPlatforms] = useState<number[]>([]);
  const [startTime, setStartTime] = useState("22:00");
  const [endTime, setEndTime] = useState("07:00");
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);
  const [isActive, setIsActive] = useState(false);

  async function load() {
    const [data, platforms] = await Promise.all([
      invoke<GlobalSchedule[]>("get_global_schedules"),
      invoke<Platform[]>("get_platforms"),
    ]);
    setSchedules(data);
    setAllPlatforms(platforms);

    const now = new Date();
    const day = (now.getDay() + 6) % 7;
    const time = `${String(now.getHours()).padStart(2, "0")}:${String(now.getMinutes()).padStart(2, "0")}`;
    setIsActive(
      data.some((s) => s.days.includes(day) && time >= s.start_time && time < s.end_time)
    );
  }

  useEffect(() => { load(); }, []);

  function toggleDay(day: number) {
    setSelectedDays((prev) =>
      prev.includes(day) ? prev.filter((d) => d !== day) : [...prev, day].sort((a,b)=>a-b)
    );
  }

  function togglePlatform(id: number) {
    setSelectedPlatforms((prev) =>
      prev.includes(id) ? prev.filter((p) => p !== id) : [...prev, id]
    );
  }

  function toggleAllPlatforms() {
    if (selectedPlatforms.length === allPlatforms.length) {
      setSelectedPlatforms([]);
    } else {
      setSelectedPlatforms(allPlatforms.map((p) => p.id));
    }
  }

  async function handleAdd() {
    if (selectedDays.length === 0) { setError("Selecciona al menos un día."); return; }
    setError("");
    setSaving(true);
    try {
      await invoke("add_global_schedule", {
        schedule: {
          days: selectedDays,
          start_time: startTime,
          end_time: endTime,
          // vacío = todas las plataformas activas
          platforms: selectedPlatforms.length === allPlatforms.length ? [] : selectedPlatforms,
        },
      });
      setSelectedDays([]);
      setSelectedPlatforms([]);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete(id: number) {
    await invoke("delete_global_schedule", { id });
    await load();
  }

  const allSelected = selectedPlatforms.length === allPlatforms.length;
  const someSelected = selectedPlatforms.length > 0 && !allSelected;

  return (
    <section className="global-section">
      <div className="global-header">
        <div className="global-title">
          <span className="global-icon">🌐</span>
          <div>
            <h2>Bloqueo General por Horario</h2>
            <p>Define horarios de bloqueo para una o todas las plataformas</p>
          </div>
        </div>
        {isActive && <span className="global-active-badge">⚡ Activo ahora</span>}
      </div>

      {schedules.length === 0 ? (
        <p className="global-empty">Sin horarios globales configurados.</p>
      ) : (
        <ul className="global-list">
          {schedules.map((s) => (
            <li key={s.id} className="global-item">
              <div className="global-item-left">
                <div className="global-item-days">
                  {DAY_LABELS.map((label, i) => (
                    <span key={i} className={`day-chip ${s.days.includes(i) ? "day-chip-on" : "day-chip-off"}`}>
                      {label}
                    </span>
                  ))}
                </div>
                <span className="global-item-platforms">
                  🔒 {formatPlatforms(s.platforms, allPlatforms)}
                </span>
              </div>
              <span className="global-item-time">{s.start_time} – {s.end_time}</span>
              <button className="btn-delete" onClick={() => handleDelete(s.id)} title="Eliminar">🗑</button>
            </li>
          ))}
        </ul>
      )}

      <div className="global-form">
        <h3>Agregar horario</h3>

        {/* Plataformas */}
        <div className="form-block">
          <label className="form-block-label">Plataformas a bloquear</label>
          <div className="platform-checks">
            <label className={`platform-check-item ${allSelected ? "check-selected" : someSelected ? "check-partial" : ""}`}>
              <input
                type="checkbox"
                checked={allSelected}
                ref={(el) => { if (el) el.indeterminate = someSelected; }}
                onChange={toggleAllPlatforms}
              />
              <span>Todas las plataformas</span>
            </label>
            <div className="platform-checks-grid">
              {allPlatforms.map((p) => {
                const meta = PLATFORM_META[p.name] ?? { icon: "🌐", color: "#888" };
                const checked = selectedPlatforms.includes(p.id);
                return (
                  <label key={p.id} className={`platform-check-item ${checked ? "check-selected" : ""}`}>
                    <input
                      type="checkbox"
                      checked={checked}
                      onChange={() => togglePlatform(p.id)}
                    />
                    <span style={{ color: checked ? meta.color : undefined }}>
                      {meta.icon} {p.name}
                    </span>
                  </label>
                );
              })}
            </div>
          </div>
        </div>

        {/* Días */}
        <div className="form-block">
          <label className="form-block-label">Días</label>
          <div className="day-picker-row">
            <div className="day-picker">
              {DAY_LABELS.map((label, i) => (
                <button
                  key={i}
                  className={`day-btn ${selectedDays.includes(i) ? "day-btn-selected" : ""}`}
                  onClick={() => toggleDay(i)}
                >
                  {label}
                </button>
              ))}
            </div>
            <div className="day-shortcuts">
              <button className="btn-shortcut" onClick={() => setSelectedDays([0,1,2,3,4,5,6])}>Todos</button>
              <button className="btn-shortcut" onClick={() => setSelectedDays([0,1,2,3,4])}>Lun–Vie</button>
              <button className="btn-shortcut" onClick={() => setSelectedDays([5,6])}>Sáb–Dom</button>
            </div>
          </div>
        </div>

        {/* Horario */}
        <div className="form-block">
          <label className="form-block-label">Horario de bloqueo</label>
          <div className="global-time-row">
            <div className="time-field">
              <label>Desde</label>
              <input type="time" value={startTime} onChange={(e) => setStartTime(e.target.value)} />
            </div>
            <span className="time-separator">hasta</span>
            <div className="time-field">
              <label>Hasta</label>
              <input type="time" value={endTime} onChange={(e) => setEndTime(e.target.value)} />
            </div>
          </div>
        </div>

        {/* Preview */}
        {selectedDays.length > 0 && (
          <p className="global-preview">
            🔒 Bloqueará{" "}
            <strong>
              {selectedPlatforms.length === 0 || selectedPlatforms.length === allPlatforms.length
                ? "todas las plataformas"
                : selectedPlatforms.map((id) => allPlatforms.find((p) => p.id === id)?.name).join(", ")}
            </strong>{" "}
            los <strong>{formatDays(selectedDays)}</strong> de{" "}
            <strong>{startTime}</strong> a <strong>{endTime}</strong>
          </p>
        )}

        {error && <p className="form-error">{error}</p>}

        <button
          className="btn-global-add"
          onClick={handleAdd}
          disabled={saving || selectedDays.length === 0}
        >
          {saving ? "Guardando…" : "Agregar horario"}
        </button>
      </div>
    </section>
  );
}
