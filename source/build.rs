use std::{env, path::PathBuf, process::Command};
fn main() {
    println!("cargo:rerun-if-env-changed=WYZAR_SETUP_MANIFEST");
    println!("cargo:rerun-if-env-changed=WYZAR_SETUP_RESOURCE");
    let generated = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("setup_payload.rs");
    let manifest = env::var_os("WYZAR_SETUP_MANIFEST")
        .map(|p| {
            println!("cargo:rerun-if-changed={}", PathBuf::from(&p).display());
            std::fs::read_to_string(p).expect("Manifest do payload deve existir")
        })
        .unwrap_or_default();
    std::fs::write(
        generated,
        format!("pub const EMBEDDED_MANIFEST: &str = {manifest:?};"),
    )
    .expect("Manifest incorporado");
    println!("cargo:rerun-if-changed=assets/app.rc");
    println!("cargo:rerun-if-changed=assets/app.manifest");
    println!("cargo:rerun-if-changed=assets/brand/wyzar.ico");
    println!("cargo:rerun-if-env-changed=WYZAR_WINDOWS_SDK_VERSION");
    if !cfg!(target_os = "windows") {
        return;
    }
    // Redistribution notices are part of the binaries, never loose installer documents.
    let mut notices = String::from("WyzarCapture - third-party notices\n\n");
    for file in [
        "windows-0.61.3-license-mit.txt",
        "proc-macro2-1.0.107-LICENSE-MIT.txt",
        "quote-1.0.47-LICENSE-MIT.txt",
        "syn-2.0.119-LICENSE-MIT.txt",
        "unicode-ident-1.0.26-LICENSE-MIT.txt",
        "unicode-ident-1.0.26-LICENSE-UNICODE.txt",
    ] {
        let path = format!("licenses/{file}");
        println!("cargo:rerun-if-changed={path}");
        notices.push_str(file);
        notices.push('\n');
        notices.push_str(&std::fs::read_to_string(path).expect("Licença local necessária"));
        notices.push_str("\n\n");
    }
    std::fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("third-party-notices.txt"),
        notices,
    )
    .expect("Avisos de redistribuição incorporados");
    // Setup's manifest changes rerun this build script. Do not let link.exe's wall clock
    // change an otherwise identical application's SHA-256 during packaging.
    println!("cargo:rustc-link-arg-bin=WyzarCapture=/Brepro");
    println!("cargo:rustc-link-arg-bin=WyzarCaptureSetup=/Brepro");
    let kit = PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10\bin");
    let mut versions: Vec<_> = std::fs::read_dir(kit)
        .expect("Windows SDK necessário")
        .flatten()
        .map(|e| e.path().join("x64/rc.exe"))
        .filter(|p| p.exists())
        .collect();
    versions.sort();
    let detected = versions
        .pop()
        .expect("rc.exe não encontrado no Windows SDK");
    let rc = if let Some(version) = env::var_os("WYZAR_WINDOWS_SDK_VERSION") {
        let path = PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10\bin")
            .join(version)
            .join("x64/rc.exe");
        assert!(
            path.is_file(),
            "Windows SDK solicitado não está instalado: {}",
            path.display()
        );
        path
    } else {
        detected
    };
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("wyzar.res");
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    let manifest_source = std::fs::read_to_string("assets/app.manifest").unwrap();
    let app_manifest =
        manifest_source.replace("version=\"0.1.0.0\"", &format!("version=\"{version}.0\""));
    std::fs::write(out.with_file_name("app.generated.manifest"), &app_manifest).unwrap();
    std::fs::write(
        out.with_file_name("setup.generated.manifest"),
        app_manifest.replace("WyzarCapture.Desktop", "WyzarCapture.Setup"),
    )
    .unwrap();
    let numeric = version.split('-').next().unwrap().replace('.', ",") + ",0";
    std::fs::write(
        out.with_file_name("version.h"),
        format!(
            "#define WYZAR_VERSION_NUM {numeric}\n#define WYZAR_VERSION_STR \"{version}\\0\"\n"
        ),
    )
    .expect("Cabeçalho de versão");
    let status = Command::new(rc)
        .args(["/nologo", "/i"])
        .arg(out.parent().unwrap())
        .arg("/fo")
        .arg(&out)
        .arg("assets/app.rc")
        .status()
        .expect("rc.exe");
    assert!(status.success(), "Falha ao compilar recursos Windows");
    println!("cargo:rustc-link-arg-bin=WyzarCapture={}", out.display());
    let setup_out = out.with_file_name("setup.res");
    let mut setup_rc = std::fs::read_to_string("assets/app.rc").unwrap();
    setup_rc = setup_rc
        .replace(
            "VALUE \"FileDescription\", \"WyzarCapture\\0\"",
            "VALUE \"FileDescription\", \"WyzarCapture — Instalador\\0\"",
        )
        .replace("app.generated.manifest", "setup.generated.manifest")
        .replace("\"WyzarCapture.exe\\0\"", "\"WyzarCaptureSetup.exe\\0\"");
    // The SDK links the conventional resource before Authenticode signing.
    // This input affects only Setup resources; the App never embeds itself.
    if let Some(payload) = env::var_os("WYZAR_SETUP_RESOURCE") {
        let payload = std::fs::canonicalize(payload).expect("Aplicativo RCDATA local necessário");
        assert!(
            payload.is_file() && !manifest.is_empty(),
            "Payload exige manifest ancorado"
        );
        println!("cargo:rerun-if-changed={}", payload.display());
        let escaped = payload
            .to_string_lossy()
            .replace('\\', "\\\\")
            .replace('"', "\\\"");
        setup_rc.push_str(&format!("\n201 RCDATA \"{escaped}\"\n"));
    }
    let setup_source = out.with_file_name("setup.rc");
    std::fs::write(&setup_source, setup_rc).unwrap();
    let setup_rc_tool = if let Some(version) = env::var_os("WYZAR_WINDOWS_SDK_VERSION") {
        PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10\bin")
            .join(version)
            .join("x64/rc.exe")
    } else {
        detected_rc()
    };
    assert!(
        Command::new(setup_rc_tool)
            .args(["/nologo", "/i"])
            .arg(out.parent().unwrap())
            .arg("/i")
            .arg(env::current_dir().unwrap())
            .arg("/fo")
            .arg(&setup_out)
            .arg(setup_source)
            .status()
            .unwrap()
            .success()
    );
    println!(
        "cargo:rustc-link-arg-bin=WyzarCaptureSetup={}",
        setup_out.display()
    );
}
fn detected_rc() -> PathBuf {
    let mut tools: Vec<_> = std::fs::read_dir(r"C:\Program Files (x86)\Windows Kits\10\bin")
        .unwrap()
        .flatten()
        .map(|e| e.path().join("x64/rc.exe"))
        .filter(|p| p.is_file())
        .collect();
    tools.sort();
    tools.pop().expect("Windows SDK local")
}
