use base64::{engine::general_purpose, Engine as _};
use sha2::{Digest, Sha256};

// --- Slugs ---

#[test]
fn test_slug_basico() {
    let slug = make_slug("Mi Proyecto CLI");
    assert_eq!(slug, "mi-proyecto-cli");
}

#[test]
fn test_slug_espacios_multiples() {
    let slug = make_slug("hola   mundo");
    assert_eq!(slug, "hola-mundo");
}

#[test]
fn test_slug_caracteres_especiales() {
    let slug = make_slug("hello world!!");
    assert_eq!(slug, "hello-world");
}

#[test]
fn test_slug_ya_es_slug() {
    let slug = make_slug("ya-es-slug");
    assert_eq!(slug, "ya-es-slug");
}

fn make_slug(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

// --- Base64 ---

#[test]
fn test_base64_encode_conocido() {
    let encoded = general_purpose::STANDARD.encode(b"hola mundo");
    assert_eq!(encoded, "aG9sYSBtdW5kbw==");
}

#[test]
fn test_base64_round_trip() {
    let original = "lym-cli 123 !@#";
    let encoded = general_purpose::STANDARD.encode(original.as_bytes());
    let decoded_bytes = general_purpose::STANDARD.decode(&encoded).unwrap();
    assert_eq!(String::from_utf8(decoded_bytes).unwrap(), original);
}

#[test]
fn test_decode_base64_sin_espacios() {
    // Verifica el fix: partes de base64 se unen sin espacio intermedio
    let parts = ["aG9s", "YSBt", "dW5k", "bw=="];
    let joined = parts.join(""); // join("") — el fix
    let decoded = general_purpose::STANDARD.decode(joined.trim()).unwrap();
    assert_eq!(String::from_utf8(decoded).unwrap(), "hola mundo");
}

#[test]
fn test_decode_base64_invalido() {
    let result = general_purpose::STANDARD.decode("esto no es base64 !!!");
    assert!(result.is_err());
}

// --- SHA-256 ---

#[test]
fn test_sha256_determinista() {
    // El mismo input siempre produce el mismo hash
    let hash = |input: &[u8]| -> String {
        let mut h = Sha256::new();
        h.update(input);
        format!("{:x}", h.finalize())
    };
    assert_eq!(hash(b"lym-cli"), hash(b"lym-cli"));
    assert_ne!(hash(b"lym-cli"), hash(b"lym-CLI")); // case-sensitive
}

#[test]
fn test_sha256_longitud_correcta() {
    let mut hasher = Sha256::new();
    hasher.update(b"cualquier texto");
    let result = format!("{:x}", hasher.finalize());
    // SHA-256 siempre produce 64 caracteres hexadecimales (256 bits / 4 bits por hex)
    assert_eq!(result.len(), 64);
}

// --- Matemáticas ---

#[test]
fn test_cal_suma() {
    assert_eq!(meval::eval_str("2 + 2").unwrap(), 4.0);
}

#[test]
fn test_cal_operaciones_compuestas() {
    assert_eq!(meval::eval_str("10 * 5 - 8").unwrap(), 42.0);
}

#[test]
fn test_cal_division() {
    assert_eq!(meval::eval_str("10 / 4").unwrap(), 2.5);
}

#[test]
fn test_cal_expresion_invalida() {
    assert!(meval::eval_str("esto no es math").is_err());
}

// --- Versión ---

#[test]
fn test_version_no_vacia() {
    let version = env!("CARGO_PKG_VERSION");
    assert!(!version.is_empty());
    // Debe tener formato semver: dígitos separados por puntos
    assert!(version.chars().all(|c| c.is_ascii_digit() || c == '.'));
}
