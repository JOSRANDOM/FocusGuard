import { useState } from "react";

export default function PrivateRelayBanner() {
  const [expanded, setExpanded] = useState(false);

  return (
    <div className="relay-banner">
      <div className="relay-banner-header">
        <div className="relay-banner-title">
          <span className="relay-icon">⚠️</span>
          <div>
            <strong>iCloud Private Relay está activo</strong>
            <p>
              Safari enruta su tráfico a través de Apple, lo que impide que
              FocusGuard bloquee el acceso en ese navegador.
            </p>
          </div>
        </div>
        <button
          className="relay-toggle"
          onClick={() => setExpanded((v) => !v)}
        >
          {expanded ? "Ocultar pasos" : "Ver cómo desactivarlo"}
        </button>
      </div>

      {expanded && (
        <ol className="relay-steps">
          <li>
            Abre <strong>Configuración del Sistema</strong> (ícono de engranaje
            en el Dock o menú Apple → Configuración del Sistema)
          </li>
          <li>
            Haz clic en tu <strong>nombre</strong> (Apple ID) en la parte
            superior del panel izquierdo
          </li>
          <li>
            Selecciona <strong>iCloud</strong>
          </li>
          <li>
            Busca <strong>Relay Privado de iCloud</strong> y desactiva el
            interruptor
          </li>
          <li>
            Confirma haciendo clic en <strong>"Desactivar"</strong> en el
            diálogo que aparece
          </li>
          <li>Vuelve a FocusGuard: los bloqueos se aplicarán automáticamente</li>
        </ol>
      )}
    </div>
  );
}
