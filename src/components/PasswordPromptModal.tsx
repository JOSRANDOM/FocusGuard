import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  title: string;
  message?: string;
  onConfirm: (password: string) => void;
  onCancel: () => void;
}

export default function PasswordPromptModal({ title, message, onConfirm, onCancel }: Props) {
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [checking, setChecking] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!password) return;
    setChecking(true);
    setError("");
    try {
      const valid = await invoke<boolean>("verify_security_password", { password });
      if (valid) {
        onConfirm(password);
      } else {
        setError("Contraseña incorrecta.");
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setChecking(false);
    }
  }

  return (
    <div className="modal-backdrop" onClick={onCancel}>
      <div className="modal modal-narrow" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>🔒 {title}</h2>
          <button className="btn-close" onClick={onCancel}>✕</button>
        </div>

        {message && <p className="modal-hint">{message}</p>}

        <form className="modal-form" onSubmit={handleSubmit}>
          <div className="form-row">
            <label>Contraseña</label>
            <input
              type="password"
              autoFocus
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
          </div>
          {error && <p className="form-error">{error}</p>}
          <button className="btn-add" type="submit" disabled={checking || !password}>
            {checking ? "Verificando…" : "Confirmar"}
          </button>
        </form>
      </div>
    </div>
  );
}
