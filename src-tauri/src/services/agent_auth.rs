//! Geração e validação de código de pareamento de 6 dígitos.
//!
//! O código é válido por 2 minutos após a geração.
//! Usado quando outro ARK Manager quer se parear com este agente.

use rand::Rng;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// TTL do código de pareamento: 2 minutos.
const CODE_TTL: Duration = Duration::from_secs(120);

/// Código de pareamento com timestamp de emissão.
struct PairingCode {
    code: String,
    issued_at: Instant,
}

/// Estado thread-safe do código de pareamento atual.
#[derive(Clone)]
pub struct PairingState {
    inner: Arc<Mutex<Option<PairingCode>>>,
}

impl PairingState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    /// Gera e armazena um novo código de 6 dígitos. Retorna o código.
    pub fn generate_new_code(&self) -> String {
        let code = format!("{:06}", rand::thread_rng().gen_range(0u32..=999_999));
        let mut guard = self.inner.lock().unwrap();
        *guard = Some(PairingCode {
            code: code.clone(),
            issued_at: Instant::now(),
        });
        code
    }

    /// Valida o código submetido. Retorna `true` se correto e não expirado.
    pub fn validate(&self, submitted: &str) -> bool {
        let guard = self.inner.lock().unwrap();
        if let Some(ref pc) = *guard {
            if pc.issued_at.elapsed() > CODE_TTL {
                return false;
            }
            return pc.code == submitted;
        }
        false
    }

    /// Invalida o código atual (após pareamento bem-sucedido).
    pub fn invalidate(&self) {
        let mut guard = self.inner.lock().unwrap();
        *guard = None;
    }

    /// Retorna `true` se existe um código ativo e não expirado.
    pub fn has_active_code(&self) -> bool {
        let guard = self.inner.lock().unwrap();
        if let Some(ref pc) = *guard {
            return pc.issued_at.elapsed() <= CODE_TTL;
        }
        false
    }
}
