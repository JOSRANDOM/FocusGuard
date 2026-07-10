import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  onStatusChange: (enabled: boolean) => void;
}

export default function SecuritySettings({ onStatusChange }: Props) {
  const [hasPassword, setHasPassword] = useState(false);
  const [loading, setLoading] = useState(true);

  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");

  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");
  const [saving, setSaving] = useState(false);

  async function load() {
    setLoading(true);
    try {
      const enabled = await invoke<boolean>("get_security_status");
      setHasPassword(enabled);
      onStatusChange(enabled);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => { load(); }, []);

  function resetForm() {
    setCurrentPassword("");
    setNewPassword("");
    setConfirmPassword("");
  }

  async function handleSave(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    setSuccess("");

    if (newPassword.length < 4) {
      setError("La contraseña debe tener al menos 4 caracteres.");
      return;
    }
    if (newPassword !== confirmPassword) {
      setError("Las contraseñas no coinciden.");
      return;
    }

    setSaving(true);
    try {
      await invoke("set_security_password", {
        newPassword,
        currentPassword: hasPassword ? currentPassword : null,
      });
      setSuccess(hasPassword ? "Contraseña actualizada." : "Contraseña activada. El bloqueo ahora requiere confirmarla.");
      resetForm();
      await load();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  }

  async function handleRemove(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    setSuccess("");
    if (!currentPassword) {
      setError("Ingresa la contraseña actual para quitar la protección.");
      return;
    }
    setSaving(true);
    try {
      await invoke("remove_security_password", { currentPassword });
      setSuccess("Protección por contraseña desactivada.");
      resetForm();
      await load();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  }

  if (loading) return <div className="loading">Cargando…</div>;

  return (
    <section className="global-section">
      <div className="global-header">
        <div className="global-title">
          <span className="global-icon">🔐</span>
          <div>
            <h2>Seguridad</h2>
            <p>
              {hasPassword
                ? "El bloqueo está protegido: activar o desactivar cualquier plataforma pide esta contraseña."
                : "Sin protección: cualquiera puede activar o desactivar el bloqueo libremente."}
            </p>
          </div>
        </div>
        {hasPassword && <span className="global-active-badge">🔒 Protegido</span>}
      </div>

      <div className="global-form">
        <h3>{hasPassword ? "Cambiar contraseña" : "Activar protección con contraseña"}</h3>

        <form onSubmit={handleSave}>
          {hasPassword && (
            <div className="form-row">
              <label>Contraseña actual</label>
              <input
                type="password"
                value={currentPassword}
                onChange={(e) => setCurrentPassword(e.target.value)}
              />
            </div>
          )}
          <div className="form-row">
            <label>{hasPassword ? "Nueva contraseña" : "Contraseña"}</label>
            <input
              type="password"
              value={newPassword}
              onChange={(e) => setNewPassword(e.target.value)}
            />
          </div>
          <div className="form-row">
            <label>Confirmar</label>
            <input
              type="password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
            />
          </div>

          {error && <p className="form-error">{error}</p>}
          {success && <p className="form-success">{success}</p>}

          <button className="btn-add" type="submit" disabled={saving}>
            {saving ? "Guardando…" : hasPassword ? "Actualizar contraseña" : "Activar protección"}
          </button>
        </form>

        {hasPassword && (
          <>
            <div className="security-divider" />
            <h3>Quitar protección</h3>
            <form onSubmit={handleRemove}>
              <div className="form-row">
                <label>Contraseña actual</label>
                <input
                  type="password"
                  value={currentPassword}
                  onChange={(e) => setCurrentPassword(e.target.value)}
                />
              </div>
              <button className="btn-delete-wide" type="submit" disabled={saving}>
                Quitar protección por contraseña
              </button>
            </form>
          </>
        )}
      </div>
    </section>
  );
}
