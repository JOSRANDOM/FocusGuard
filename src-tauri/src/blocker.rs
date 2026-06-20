const BEGIN_MARKER: &str = "# BEGIN FOCUSGUARD";
const END_MARKER: &str = "# END FOCUSGUARD";

const HELPER_PATH: &str = "/usr/local/bin/focusguard-helper";
const SUDOERS_PATH: &str = "/etc/sudoers.d/focusguard";

pub fn apply_blocks(domains: &[String]) -> Result<(), String> {
    let new_content = build_hosts_content(domains)?;
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

    let ps_cmd = format!(
        "Start-Process powershell -Verb RunAs -Wait -ArgumentList \
        '-Command Copy-Item \\'{}\\' \\'{}\\' -Force'",
        temp_path.display(),
        hosts_path
    );

    let output = std::process::Command::new("powershell")
        .args(["-ExecutionPolicy", "Bypass", "-Command", &ps_cmd])
        .output()
        .map_err(|e| format!("PowerShell falló: {}", e))?;

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
