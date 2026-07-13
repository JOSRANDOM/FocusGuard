# Por qué un bloqueo a veces "no funciona" — DNS seguro del navegador

FocusGuard bloquea sitios editando el archivo `hosts` del sistema. Es el mismo método que usan
Cold Turkey, Freedom y bloqueadores similares, y funciona perfecto salvo por una cosa: si tu
navegador tiene activado **"DNS seguro" / "Secure DNS" (DNS-over-HTTPS)**, deja de preguntarle al
sistema operativo cómo resolver un dominio y le pregunta directo a un servidor externo (Google,
Cloudflare, etc.) — saltándose por completo el archivo `hosts`, y por lo tanto el bloqueo.

## Cómo se ve el síntoma

1. Bloqueas una plataforma (ej. YouTube) en FocusGuard.
2. Entras al sitio: por un segundo aparece "Conéctate a Internet" o un error de conexión — el
   bloqueo sí funcionó en ese primer intento.
3. A los pocos segundos la página se recarga sola y el sitio carga normal — el navegador reintentó
   con DNS seguro, que ignoró el bloqueo.

Esto **no es un error de FocusGuard**: el `hosts` sigue bloqueado todo el tiempo, es el navegador
el que decide no consultarlo.

## Solución: desactivar DNS seguro

### Google Chrome
1. Ir a `chrome://settings/security` (o Menú ⋮ → Configuración → Privacidad y seguridad → Seguridad).
2. Buscar la sección **"Usar DNS seguro"**.
3. Desactivar el interruptor (o elegir **"Con tu proveedor de servicios de Internet actual"**, que
   respeta más el `hosts` que un proveedor específico como Google o Cloudflare — pero lo más
   confiable es apagarlo del todo).

### Microsoft Edge
1. Ir a `edge://settings/privacy` → sección **"Servicios de seguridad"**.
2. Buscar **"Usar DNS seguro para especificar cómo se resuelven las búsquedas de DNS"**.
3. Desactivar el interruptor.

### Mozilla Firefox
1. Ir a `about:preferences#general` y bajar hasta **"Configuración de red"** → botón
   **"Configuración"**.
2. Desmarcar **"Habilitar DNS mediante HTTPS"** (o elegir "Desactivado" en el menú desplegable).

### Safari (macOS)
Safari usa el resolutor de DNS del sistema por defecto y normalmente **no** necesita este ajuste;
si el bloqueo no funciona ahí, revisar en su lugar `Preferencias del Sistema → Red → DNS` por si
hay un servidor DNS-over-HTTPS configurado a nivel de sistema.

## Nota para otros navegadores basados en Chromium

Brave, Opera y otros navegadores basados en Chromium tienen la misma opción, normalmente en
`Configuración → Privacidad y seguridad → Seguridad → DNS seguro` — los pasos son equivalentes a
los de Chrome.

## Por qué FocusGuard no puede arreglar esto por su cuenta

No hay forma de forzar al navegador a respetar el `hosts` si el usuario decide usar DNS seguro con
un proveedor externo — es una decisión que vive enteramente dentro del navegador. La única
alternativa real sería bloquear a nivel de firewall los rangos de IP de cada plataforma (mucho más
frágil, esas IPs cambian) o correr un resolutor DNS local que intercepte las consultas (mucho más
complejo para una app de este tamaño). Por eso la recomendación es simplemente desactivar DNS
seguro mientras se usa FocusGuard.
