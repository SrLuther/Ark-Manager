use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IniParserError {
    #[error("Arquivo INI não encontrado: {0}")]
    NotFound(String),
    #[error("Erro ao ler arquivo INI: {0}")]
    ReadError(String),
    #[error("Encoding inválido: {0}")]
    EncodingError(String),
}

/// Resultado do parsing: mapa de seções → mapa de chave → valor.
pub type IniData = HashMap<String, HashMap<String, String>>;

/// Lê um arquivo INI do ARK (UTF-16 LE com BOM ou UTF-8 fallback).
/// Retorna um mapa hierárquico: seção → chave → valor.
pub async fn read_ini(path: &Path) -> Result<IniData, IniParserError> {
    if !path.exists() {
        return Err(IniParserError::NotFound(path.display().to_string()));
    }

    let raw = tokio::fs::read(path)
        .await
        .map_err(|e| IniParserError::ReadError(e.to_string()))?;

    let content = decode_ini_bytes(&raw)?;
    Ok(parse_ini_content(&content))
}

/// Decodifica bytes de INI: tenta UTF-16 LE (com BOM FF FE), senão usa UTF-8.
fn decode_ini_bytes(raw: &[u8]) -> Result<String, IniParserError> {
    // UTF-16 LE com BOM: primeiros 2 bytes são FF FE
    if raw.len() >= 2 && raw[0] == 0xFF && raw[1] == 0xFE {
        let u16_data: Vec<u16> = raw[2..]
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect();
        String::from_utf16(&u16_data)
            .map_err(|e| IniParserError::EncodingError(e.to_string()))
    } else {
        String::from_utf8(raw.to_vec())
            .map_err(|e| IniParserError::EncodingError(e.to_string()))
    }
}

/// Faz o parsing linha a linha do conteúdo INI.
fn parse_ini_content(content: &str) -> IniData {
    let mut result: IniData = HashMap::new();
    let mut current_section = String::from("_root");

    for raw_line in content.lines() {
        let line = raw_line.trim();

        // Ignora linhas vazias e comentários
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        // Seção: [NomeDaSeção]
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].trim().to_string();
            result.entry(current_section.clone()).or_default();
            continue;
        }

        // Chave=Valor
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim().to_string();
            let value = line[eq_pos + 1..].trim().to_string();
            result
                .entry(current_section.clone())
                .or_default()
                .insert(key, value);
        }
    }

    result
}

/// Obtém um valor de uma seção/chave específica.
pub fn get_value<'a>(data: &'a IniData, section: &str, key: &str) -> Option<&'a str> {
    data.get(section)?.get(key).map(|s| s.as_str())
}

/// Obtém um valor como f64, retornando o default se ausente ou inválido.
pub fn get_f64(data: &IniData, section: &str, key: &str, default: f64) -> f64 {
    get_value(data, section, key)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Obtém um valor como i64, retornando o default se ausente ou inválido.
pub fn get_i64(data: &IniData, section: &str, key: &str, default: i64) -> i64 {
    get_value(data, section, key)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Obtém um valor como bool (true/True/1 → true).
pub fn get_bool(data: &IniData, section: &str, key: &str, default: bool) -> bool {
    get_value(data, section, key)
        .map(|v| matches!(v.to_lowercase().as_str(), "true" | "1" | "yes"))
        .unwrap_or(default)
}

