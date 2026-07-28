use base64::{engine::general_purpose, Engine as _};
use chrono::{
    format::{DelayedFormat, StrftimeItems},
    DateTime, Local,
};
use clap::{error::Result, Parser, Subcommand};
use rand::Rng;
use reqwest::{Client, Error, Response};
use serde_json::{from_str, Value};
use sha2::{Digest, Sha256};
use std::net::UdpSocket;
use uuid::Uuid;

#[derive(Parser)]
pub struct ClapArgs {
    #[clap(subcommand)]
    command: Args,
}

#[derive(Subcommand)]
enum Args {
    /// Obtén la versión actual de lym.
    Version,

    /// Obtén la fecha y hora actual.
    Time,

    /// Traduce texto de inglés al español.
    En { text: Vec<String> },

    /// Traduce texto de español al inglés.
    Es { text: Vec<String> },

    /// Realiza operaciones matemáticas.
    Cal { operation: String },

    /// Genera una contraseña aleatoria.
    #[clap(value_parser, allow_hyphen_values = true)]
    Pass { length: Option<String> },

    /// Muestra la IP local de la máquina.
    Ip,

    /// Muestra la IP pública de esta máquina.
    Pubip,

    /// Hace ping a un host y muestra la latencia.
    Ping { host: String },

    /// Genera un UUID v4.
    Uuid,

    /// Genera el hash SHA-256 de un texto.
    Hash { text: Vec<String> },

    /// Codifica texto en Base64.
    Encode { text: Vec<String> },

    /// Decodifica texto desde Base64.
    Decode { text: Vec<String> },

    /// Convierte texto a mayúsculas.
    Upper { text: Vec<String> },

    /// Convierte texto a formato slug (para URLs o nombres de archivo).
    Slug { text: Vec<String> },

    /// Cuenta caracteres y palabras de un texto.
    Count { text: Vec<String> },

    /// Lanza una moneda al aire (cara o cruz).
    Coin,

    /// Tira un dado de N caras (por defecto: 6).
    Roll { sides: Option<u32> },

    /// Elige aleatoriamente un elemento de una lista.
    Pick { options: Vec<String> },
}

pub async fn run(args: ClapArgs) {
    const EN_TO_ES: &str = "en|es";
    const ES_TO_EN: &str = "es|en";

    match args.command {
        Args::Version => Command::get_version(),
        Args::Time => Command::get_time(),
        Args::Cal { operation } => Command::calculate(&operation),
        Args::En { text } => Command::translate(EN_TO_ES, &text)
            .await
            .expect("Error en el comando 'en'"),
        Args::Es { text } => Command::translate(ES_TO_EN, &text)
            .await
            .expect("Error en el comando 'es'"),
        Args::Pass { length } => Command::generate_password(length),
        Args::Ip => Command::get_local_ip(),
        Args::Pubip => Command::get_public_ip()
            .await
            .expect("Error en el comando 'pubip'"),
        Args::Ping { host } => Command::ping(&host),
        Args::Uuid => Command::generate_uuid(),
        Args::Hash { text } => Command::hash_sha256(&text),
        Args::Encode { text } => Command::encode_base64(&text),
        Args::Decode { text } => Command::decode_base64(&text),
        Args::Upper { text } => Command::to_upper(&text),
        Args::Slug { text } => Command::to_slug(&text),
        Args::Count { text } => Command::count(&text),
        Args::Coin => Command::coin_flip(),
        Args::Roll { sides } => Command::roll_dice(sides),
        Args::Pick { options } => Command::pick(&options),
    }
}

struct Command;

impl Command {
    fn get_version() {
        // FIX: usa env! para leer la versión directo del Cargo.toml en tiempo de compilación
        println!("versión de lym: {}", env!("CARGO_PKG_VERSION"));
    }

    fn get_time() {
        let local_time: DateTime<Local> = Local::now();
        let date: DelayedFormat<StrftimeItems> = local_time.format("%H:%M:%S - %d/%m/%y");
        println!("{}", date);
    }

    // FIX: toma &str en lugar de &String (más idiomático en Rust)
    // FIX: eliminado el chequeo de longitud incorrecto y el panic!
    //      meval ya maneja expresiones inválidas devolviendo Err
    fn calculate(operation: &str) {
        match meval::eval_str(operation) {
            Ok(result) => println!("{result}"),
            Err(error) => eprintln!("lym: operación inválida: {error}"),
        }
    }

    // FIX: &Vec<String> → &[String] (elimina el warning de Clippy ptr_arg)
    // FIX: mensajes de error traducidos al español
    async fn translate(language: &str, sentence: &[String]) -> Result<(), Error> {
        const URL: &str = "https://api.mymemory.translated.net/get";
        let sentence = sentence.join(" ");

        let params: [(&str, &str); 2] = [("q", &sentence), ("langpair", language)];

        let client: Client = Client::new();
        let response: Response = match client.post(URL).form(&params).send().await {
            Ok(res) => res,
            Err(error) => {
                eprintln!("lym: error al enviar la solicitud de traducción: {error}");
                return Ok(());
            }
        };

        if response.status().is_success() {
            let body_text: String = match response.text().await {
                Ok(text) => text,
                Err(error) => {
                    eprintln!("lym: error al leer la respuesta de traducción: {error}");
                    return Ok(());
                }
            };

            let body: Value = match from_str(&body_text) {
                Ok(value) => value,
                Err(error) => {
                    eprintln!("lym: error al procesar la respuesta JSON: {error}");
                    return Ok(());
                }
            };

            if let Some(translated_word) = body["responseData"]["translatedText"].as_str() {
                println!("{}", translated_word);
            }
        } else {
            eprintln!(
                "lym: la solicitud de traducción falló con estado {}",
                response.status()
            );
        }
        Ok(())
    }

    fn generate_password(length: Option<String>) {
        const DEFAULT_VAL: i32 = 8;
        const MAX: i32 = 10000;
        const CHARSET: &[u8; 69] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
            abcdefghijklmnopqrstuvwxyz\
            0123456789\
            !@#$*_.";

        let length_pass: i32 = match length {
            Some(len) => match len.parse::<i32>() {
                Ok(len) if len > 0 && len < MAX => len,
                _ => return eprintln!("Ingresa un número entero positivo y/o menor a {MAX}"),
            },
            None => DEFAULT_VAL,
        };

        let mut rng = rand::thread_rng();
        let password: String = (0..length_pass)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        println!("{}", password);
    }

    // --- Utilidades del sistema ---

    fn get_local_ip() {
        match UdpSocket::bind("0.0.0.0:0") {
            Ok(socket) => match socket.connect("8.8.8.8:80") {
                Ok(_) => match socket.local_addr() {
                    Ok(addr) => println!("{}", addr.ip()),
                    Err(e) => eprintln!("lym: no se pudo obtener la IP local: {e}"),
                },
                Err(e) => eprintln!("lym: no se pudo determinar la IP local: {e}"),
            },
            Err(e) => eprintln!("lym: error al crear socket: {e}"),
        }
    }

    async fn get_public_ip() -> Result<(), Error> {
        let client = Client::new();
        let response = match client.get("https://api.ipify.org").send().await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("lym: error obteniendo IP pública: {e}");
                return Ok(());
            }
        };

        match response.text().await {
            Ok(ip) => println!("{}", ip.trim()),
            Err(e) => eprintln!("lym: error leyendo respuesta: {e}"),
        }
        Ok(())
    }

    fn ping(host: &str) {
        use std::process::Command as SysCmd;

        let output = match SysCmd::new("ping").args(["-c", "1", "-W", "2", host]).output() {
            Ok(o) => o,
            Err(e) => {
                eprintln!("lym: error al ejecutar ping: {e}");
                return;
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        if output.status.success() {
            if let Some(time_str) = stdout
                .lines()
                .find(|l| l.contains("time="))
                .and_then(|l| l.split("time=").nth(1))
            {
                let time: String = time_str
                    .split_whitespace()
                    .take(2)
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("{host}: ok ({time})");
            } else {
                println!("{host}: ok");
            }
        } else {
            eprintln!("{host}: sin respuesta");
        }
    }

    // --- Generadores ---

    fn generate_uuid() {
        println!("{}", Uuid::new_v4());
    }

    // FIX: &Vec<String> → &[String]
    fn hash_sha256(text: &[String]) {
        let input = text.join(" ");
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        println!("{:x}", hasher.finalize());
    }

    // FIX: &Vec<String> → &[String]
    fn encode_base64(text: &[String]) {
        let input = text.join(" ");
        println!("{}", general_purpose::STANDARD.encode(input.as_bytes()));
    }

    // FIX: &Vec<String> → &[String]
    // FIX: join("") en lugar de join(" ") — base64 no puede contener espacios internos
    fn decode_base64(text: &[String]) {
        let input = text.join("");
        match general_purpose::STANDARD.decode(input.trim()) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(decoded) => println!("{}", decoded),
                Err(_) => eprintln!("lym: el resultado no es texto UTF-8 válido"),
            },
            Err(e) => eprintln!("lym: input Base64 inválido: {e}"),
        }
    }

    // --- Texto ---

    // FIX: &Vec<String> → &[String]
    fn to_upper(text: &[String]) {
        println!("{}", text.join(" ").to_uppercase());
    }

    // FIX: &Vec<String> → &[String]
    fn to_slug(text: &[String]) {
        let slug = text
            .join(" ")
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");
        println!("{}", slug);
    }

    // FIX: &Vec<String> → &[String]
    fn count(text: &[String]) {
        let input = text.join(" ");
        let chars = input.chars().count();
        let words = input.split_whitespace().count();
        println!("caracteres: {chars}, palabras: {words}");
    }

    // --- Azar ---

    fn coin_flip() {
        let mut rng = rand::thread_rng();
        println!("{}", if rng.gen_bool(0.5) { "cara" } else { "cruz" });
    }

    fn roll_dice(sides: Option<u32>) {
        let n = sides.unwrap_or(6);
        if n < 2 {
            eprintln!("lym: el dado debe tener al menos 2 caras");
            return;
        }
        let mut rng = rand::thread_rng();
        println!("{}", rng.gen_range(1..=n));
    }

    // FIX: &Vec<String> → &[String]
    fn pick(options: &[String]) {
        if options.is_empty() {
            eprintln!("lym: debes proporcionar al menos una opción");
            return;
        }
        let mut rng = rand::thread_rng();
        println!("{}", options[rng.gen_range(0..options.len())]);
    }
}

#[cfg(test)]
mod tests;
