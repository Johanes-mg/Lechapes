use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;

pub struct Plugins {
    pub open: bool,
    pub word_wrap: bool,
    pub installing: bool,
    pub status: String,
    result_rx: Option<Receiver<Result<(), String>>>,
}

impl Default for Plugins {
    fn default() -> Self {
        Self {
            open: false,
            word_wrap: true,
            installing: false,
            status: String::new(),
            result_rx: None,
        }
    }
}

impl Plugins {
    pub fn root() -> PathBuf {
        let base = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("LOCALAPPDATA").map(PathBuf::from))
            .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
            .unwrap_or_else(std::env::temp_dir);
        base.join("Lechapes").join("plugins")
    }

    pub fn prettier_ready() -> bool {
        let root = Self::root();
        root.join("node").join("node.exe").exists()
            && root.join("prettier").join("node_modules").join("prettier")
                .join("bin").join("prettier.cjs").exists()
    }

    pub fn install_prettier(&mut self) {
        if self.installing || Self::prettier_ready() {
            return;
        }
        self.installing = true;
        self.status = "Téléchargement de Node.js portable et de Prettier…".into();
        let (tx, rx) = mpsc::channel();
        self.result_rx = Some(rx);
        thread::spawn(move || {
            let result = Self::install_prettier_blocking();
            let _ = tx.send(result);
        });
    }

    fn install_prettier_blocking() -> Result<(), String> {
        let root = Self::root();
        std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;

        #[cfg(target_os = "windows")]
        {
            let root_q = root.to_string_lossy().replace('\'', "''");
            let script = format!(
                "$ErrorActionPreference='Stop'; \
                 $root='{}'; \
                 $zip=Join-Path $root 'node-portable.zip'; \
                 Invoke-WebRequest -Uri 'https://nodejs.org/dist/v22.14.0/node-v22.14.0-win-x64.zip' -OutFile $zip; \
                 Expand-Archive -LiteralPath $zip -DestinationPath $root -Force; \
                 $old=Join-Path $root 'node-v22.14.0-win-x64'; \
                 $node=Join-Path $root 'node'; \
                 if (Test-Path $node) {{ Remove-Item $node -Recurse -Force }}; \
                 Move-Item $old $node; \
                 $npm=Join-Path $node 'npm.cmd'; \
                 & $npm install --prefix (Join-Path $root 'prettier') 'prettier@3.6.2'; \
                 if ($LASTEXITCODE -ne 0) {{ throw 'npm install prettier a échoué' }}; \
                 Remove-Item $zip -Force",
                root_q
            );
            let output = Command::new("powershell")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
                .output()
                .map_err(|e| format!("Impossible de lancer PowerShell : {e}"))?;
            if !output.status.success() {
                let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
                return Err(if detail.is_empty() { "Installation de Prettier échouée".into() } else { detail });
            }
            return Ok(());
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err("L’installation automatique du plugin Prettier est actuellement configurée pour Windows x64.".into())
        }
    }

    pub fn poll(&mut self) {
        let result = self.result_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(result) = result {
            self.installing = false;
            self.result_rx = None;
            self.status = match result {
                Ok(()) => "Prettier est installé et prêt.".into(),
                Err(error) => format!("Échec de l’installation : {error}"),
            };
        }
    }

    pub fn format_file(path: &PathBuf) -> Result<bool, String> {
        let ext = path.extension().and_then(|x| x.to_str()).unwrap_or("").to_ascii_lowercase();
        if ext == "rs" {
            let output = Command::new("rustfmt")
                .args(["--edition", "2021"])
                .arg(path)
                .output()
                .map_err(|e| format!("Impossible de lancer rustfmt : {e}"))?;
            if !output.status.success() {
                return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
            }
            return Ok(true);
        }
        let supported = matches!(ext.as_str(),
            "js"|"jsx"|"cjs"|"mjs"|"ts"|"tsx"|"cts"|"mts"|"json"|"jsonc"|
            "html"|"htm"|"css"|"scss"|"less"|"md"|"mdx"|"yaml"|"yml"|
            "graphql"|"gql"|"vue"|"svelte"|"astro");
        if !supported { return Ok(false); }
        if !Self::prettier_ready() {
            return Err("Prettier n’est pas installé. Ouvre Plugins et installe le formateur.".into());
        }
        let root = Self::root();
        let node = root.join("node").join("node.exe");
        let prettier = root.join("prettier").join("node_modules").join("prettier").join("bin").join("prettier.cjs");
        let output = Command::new(node)
            .arg(prettier)
            .arg("--write")
            .arg(path)
            .output()
            .map_err(|e| format!("Impossible de lancer Prettier : {e}"))?;
        if !output.status.success() {
            let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(if detail.is_empty() { "Prettier a échoué".into() } else { detail });
        }
        Ok(true)
    }
}
