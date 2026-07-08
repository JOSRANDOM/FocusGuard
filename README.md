# FocusGuard

Recupera tu enfoque. App de escritorio para **macOS y Windows** que bloquea redes sociales y sitios que distraen, por horario — editando el archivo `hosts` del sistema para redirigir esos dominios a `0.0.0.0`.

## Qué hace

- **Plataformas configurables**: viene con TikTok, Facebook, Instagram y YouTube precargadas (cada una con su lista de dominios asociados); se activan/desactivan individualmente.
- **Horarios por plataforma**: ventanas *permitidas* — fuera de esas horas, la plataforma queda bloqueada. Sin horario definido, la plataforma queda bloqueada 24/7 mientras esté activada.
- **Horarios globales**: ventanas *bloqueadas* que aplican a varias plataformas a la vez (o a todas, si no se elige ninguna en particular) — por rango de días y hora. Es lo que se ve en la pantalla "Días / Horario de bloqueo" de la app.
- **Actualización automática**: un scheduler en segundo plano revisa cada 60 segundos si algo cambió (cambio de hora, de día, de configuración) y reescribe el `hosts` cuando corresponde.
- **Setup mínimo por plataforma**:
  - *macOS*: instala un helper privilegiado una sola vez (con `sudoers` `NOPASSWD`) para no pedir contraseña en cada bloqueo; si no está instalado, cae a un diálogo de administrador vía `osascript`. También detecta si iCloud Private Relay está activo (puede interferir con el bloqueo por DNS/hosts).
  - *Windows*: pide elevación de administrador (UAC) cada vez que se aplica un cambio al `hosts`.

## Stack

- **Frontend**: React 19 + TypeScript + Vite.
- **Backend**: Rust vía [Tauri 2](https://tauri.app/).
- **Persistencia**: SQLite embebido (`rusqlite`, bundled) — plataformas y horarios se guardan localmente, sin backend externo.

## Estructura del proyecto

```
src/                        Frontend (React)
├─ App.tsx                  Shell principal
└─ components/
   ├─ PlatformCard.tsx       Tarjeta de cada red social con su toggle
   ├─ ScheduleModal.tsx      Crear/editar horario por plataforma
   ├─ GlobalSchedule.tsx     Horarios globales (días + rango horario)
   ├─ SetupBanner.tsx        Aviso de instalación del helper (macOS)
   └─ PrivateRelayBanner.tsx Aviso de iCloud Private Relay activo (macOS)

src-tauri/src/               Backend (Rust)
├─ lib.rs                    Comandos Tauri expuestos al frontend + entry point
├─ db.rs                     Modelo de datos y acceso a SQLite
├─ scheduler.rs              Lógica de "¿debe estar bloqueado ahora?" + loop de 60s
└─ blocker.rs                Lectura/escritura del hosts del sistema (por SO)

landing/                     Landing page estática (marketing, no forma parte de la app)
.github/workflows/release.yml  Build + release automático al pushear un tag `vX.Y.Z`
```

## Desarrollo local

Requisitos: Node.js 22+, Rust (toolchain estable) y, en Windows, las Build Tools de Visual Studio (workload de C++, para el linker MSVC).

```bash
npm install
npm run tauri dev      # levanta la app en modo desarrollo (hot reload)
```

Otros comandos útiles:

```bash
npm run build           # build del frontend (tsc + vite build)
cargo check              # (dentro de src-tauri/) verificar que el backend compila
```

## Build y release

Cada push de un tag `v*` (ej. `v1.0.2`) dispara `release.yml` en GitHub Actions, que compila para **macOS (Apple Silicon)** y **Windows x64** y publica los instaladores en la release de GitHub correspondiente.
