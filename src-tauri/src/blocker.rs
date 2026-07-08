const BEGIN_MARKER: &str = "# BEGIN FOCUSGUARD";
const END_MARKER: &str = "# END FOCUSGUARD";

const HELPER_PATH: &str = "/usr/local/bin/focusguard-helper";
const SUDOERS_PATH: &str = "/etc/sudoers.d/focusguard";

pub fn apply_blocks(domains: &[String]) -> Result<(), String> {
    let new_content = build_hosts_content(domains)?;

    // El scheduler llama apply_blocks cada 60s sin importar si algo cambió.
    // Sin esta comparación, cada llamada reescribe el hosts y en Windows eso
    // dispara un prompt de UAC nuevo cada minuto (en macOS, un prompt de
    // contraseña si el helper no está instalado). Si el contenido resultante
    // es idéntico al que ya está en el hosts, no hay nada que aplicar.
    if let Ok(current) = read_hosts() {
        if current == new_content {
            return Ok(());
        }
    }

    write_hosts(new_content)
}

/// Verifica si el helper privilegiado ya está instalado.
pub fn is_helper_installed() -> bool {
    #[cfg(target_os = "macos")]
    {
        std::path::Path::new(HELPER_PATH).exists()
            && std::path::Path::new(SUDOERS_PATH).exists()
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// Instala el helper y la entrada sudoers (pide contraseña UNA sola vez).
#[cfg(target_os = "macos")]
pub fn install_helper() -> Result<(), String> {
    let script = format!(
        r#"do shell script "
set -e
cat > {helper} << 'HELPEREOF'
#!/bin/bash
if [ \"$1\" = apply ]; then
  cp /tmp/focusguard_hosts /etc/hosts
  dscacheutil -flushcache
  killall -HUP mDNSResponder 2>/dev/null || true
fi
HELPEREOF
chmod +x {helper}
echo '%admin ALL=(ALL) NOPASSWD: {helper}' > {sudoers}
chmod 440 {sudoers}
" with administrator privileges"#,
        helper = HELPER_PATH,
        sudoers = SUDOERS_PATH,
    );

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("osascript falló: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        if err.contains("User cancelled") || err.contains("(-128)") {
            return Err("Configuración cancelada por el usuario".to_string());
        }
        return Err(format!("Error al instalar helper: {}", err));
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn install_helper() -> Result<(), String> {
    Ok(())
}

fn build_hosts_content(domains_to_block: &[String]) -> Result<String, String> {
    let current = read_hosts()?;

    let mut lines: Vec<&str> = Vec::new();
    let mut in_section = false;

    for line in current.lines() {
        if line.trim() == BEGIN_MARKER {
            in_section = true;
            continue;
        }
        if line.trim() == END_MARKER {
            in_section = false;
            continue;
        }
        if !in_section {
            lines.push(line);
        }
    }

    while lines.last().map(|l| l.trim().is_empty()).unwrap_or(false) {
        lines.pop();
    }

    let mut result = lines.join("\n");

    if !domains_to_block.is_empty() {
        result.push_str(&format!("\n\n{}\n", BEGIN_MARKER));
        for domain in domains_to_block {
            result.push_str(&format!("0.0.0.0 {}\n", domain));
        }
        result.push_str(&format!("{}\n", END_MARKER));
    } else {
        result.push('\n');
    }

    Ok(result)
}

// ─── macOS ────────────────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
fn read_hosts() -> Result<String, String> {
    std::fs::read_to_string("/etc/hosts")
        .map_err(|e| format!("No se pudo leer /etc/hosts: {}", e))
}

#[cfg(target_os = "macos")]
fn write_hosts(content: String) -> Result<(), String> {
    let temp_path = "/tmp/focusguard_hosts";
    std::fs::write(temp_path, &content)
        .map_err(|e| format!("No se pudo escribir archivo temporal: {}", e))?;

    if is_helper_installed() {
        // Sin diálogo de contraseña gracias al sudoers NOPASSWD
        let output = std::process::Command::new("sudo")
            .args([HELPER_PATH, "apply"])
            .output()
            .map_err(|e| format!("sudo helper falló: {}", e))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
    } else {
        // Fallback: osascript (pide contraseña)
        let script = r#"do shell script "cp /tmp/focusguard_hosts /etc/hosts && dscacheutil -flushcache && killall -HUP mDNSResponder 2>/dev/null; true" with administrator privileges"#;
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| format!("osascript falló: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            if err.contains("User cancelled") || err.contains("(-128)") {
                return Err("Operación cancelada".to_string());
            }
            return Err(format!("Error al aplicar bloqueos: {}", err));
        }
    }

    Ok(())
}

// ─── Windows ──────────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn read_hosts() -> Result<String, String> {
    std::fs::read_to_string(r"C:\Windows\System32\drivers\etc\hosts")
        .map_err(|e| format!("No se pudo leer hosts: {}", e))
}

#[cfg(target_os = "windows")]
fn write_hosts(content: String) -> Result<(), String> {
    let hosts_path = r"C:\Windows\System32\drivers\etc\hosts";
    let temp_path = std::env::temp_dir().join("focusguard_hosts.txt");

    std::fs::write(&temp_path, &content)
        .map_err(|e| format!("No se pudo escribir temporal: {}", e))?;

    // Se usa un script .ps1 temporal en vez de anidar el Copy-Item dentro de
    // -ArgumentList como un solo string: en PowerShell una comilla simple se
    // escapa duplicándola (''), no con \' — el string se cerraba antes de
    // tiempo y Start-Process recibía el comando partido en varios argumentos
    // posicionales sueltos (el error "PositionalParameterNotFound").
    let script_path = std::env::temp_dir().join("focusguard_apply_hosts.ps1");
    let script_content = format!(
        "Copy-Item -LiteralPath '{}' -Destination '{}' -Force",
        temp_path.display().to_string().replace('\'', "''"),
        hosts_path.replace('\'', "''"),
    );
    std::fs::write(&script_path, &script_content)
        .map_err(|e| format!("No se pudo escribir script temporal: {}", e))?;

    // -ArgumentList como lista separada por comas: cada elemento llega como
    // un argumento propio al proceso elevado, sin depender de que el shell
    // interno vuelva a partir un string plano.
    let ps_cmd = format!(
        "Start-Process powershell -Verb RunAs -Wait -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-File','{}'",
        script_path.display().to_string().replace('\'', "''"),
    );

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_cmd])
        .output()
        .map_err(|e| format!("PowerShell falló: {}", e))?;

    let _ = std::fs::remove_file(&script_path);

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

// ─── Otros SO ─────────────────────────────────────────────────────────────────

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn read_hosts() -> Result<String, String> {
    Ok(String::new())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn write_hosts(_content: String) -> Result<(), String> {
    Err("Sistema operativo no soportado".to_string())
}
