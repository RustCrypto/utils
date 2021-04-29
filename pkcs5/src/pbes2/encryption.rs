//! PBES2 encryption implementation

use super::{EncryptionScheme, Parameters, Pbkdf2Params, Pbkdf2Prf};
use crate::CryptoError;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use hmac::{
    digest::{generic_array::ArrayLength, BlockInput, FixedOutput, Reset, Update},
    Hmac,
};
use pbkdf2::pbkdf2;
use sha2::Sha256;

type Aes128Cbc = Cbc<aes::Aes128, Pkcs7>;
type Aes256Cbc = Cbc<aes::Aes256, Pkcs7>;

/// Maximum size of a derived encryption key
const MAX_KEY_LEN: usize = 32;

pub fn encrypt_in_place<'b>(
    params: &Parameters<'_>,
    password: impl AsRef<[u8]>,
    buffer: &'b mut [u8],
    pos: usize,
) -> Result<&'b [u8], CryptoError> {
    let pbkdf2_params = params.kdf.pbkdf2().ok_or(CryptoError)?;

    let key = EncryptionKey::derive_with_pbkdf2::<Sha256>(
        password.as_ref(),
        &pbkdf2_params,
        params.encryption.key_size(),
    )?;

    match params.encryption {
        EncryptionScheme::Aes128Cbc { iv } => {
            let cipher = Aes128Cbc::new_from_slices(key.as_slice(), iv).map_err(|_| CryptoError)?;
            cipher.encrypt(buffer, pos).map_err(|_| CryptoError)
        }
        EncryptionScheme::Aes256Cbc { iv } => {
            let cipher = Aes256Cbc::new_from_slices(key.as_slice(), iv).map_err(|_| CryptoError)?;
            cipher.encrypt(buffer, pos).map_err(|_| CryptoError)
        }
    }
}

/// Decrypt a message encrypted with PBES2-based key derivation
pub fn decrypt_in_place<'a>(
    params: &Parameters<'_>,
    password: impl AsRef<[u8]>,
    buffer: &'a mut [u8],
) -> Result<&'a [u8], CryptoError> {
    let pbkdf2_params = params.kdf.pbkdf2().ok_or(CryptoError)?;

    let key = EncryptionKey::derive_with_pbkdf2::<Sha256>(
        password.as_ref(),
        &pbkdf2_params,
        params.encryption.key_size(),
    )?;

    match params.encryption {
        EncryptionScheme::Aes128Cbc { iv } => {
            let cipher = Aes128Cbc::new_from_slices(key.as_slice(), iv).map_err(|_| CryptoError)?;
            cipher.decrypt(buffer).map_err(|_| CryptoError)
        }
        EncryptionScheme::Aes256Cbc { iv } => {
            let cipher = Aes256Cbc::new_from_slices(key.as_slice(), iv).map_err(|_| CryptoError)?;
            cipher.decrypt(buffer).map_err(|_| CryptoError)
        }
    }
}

/// Encryption key as derived by PBKDF2
// TODO(tarcieri): zeroize?
struct EncryptionKey {
    buffer: [u8; MAX_KEY_LEN],
    length: usize,
}

impl EncryptionKey {
    /// Derive key using PBKDF2.
    fn derive_with_pbkdf2<D>(
        password: &[u8],
        params: &Pbkdf2Params<'_>,
        length: usize,
    ) -> Result<Self, CryptoError>
    where
        D: Update + BlockInput + FixedOutput + Reset + Default + Clone + Sync,
        D::BlockSize: ArrayLength<u8>,
    {
        // We only support PBKDF2-SHA256 for now
        if params.prf != Pbkdf2Prf::HmacWithSha256 {
            return Err(CryptoError);
        }

        // Ensure key length matches what is expected for the given algorithm
        if let Some(len) = params.key_length {
            if length != len as usize {
                return Err(CryptoError);
            }
        }

        if length > MAX_KEY_LEN {
            return Err(CryptoError);
        }

        let mut buffer = [0u8; MAX_KEY_LEN];

        pbkdf2::<Hmac<D>>(
            password,
            params.salt,
            params.iteration_count as u32,
            &mut buffer[..length],
        );

        Ok(Self { buffer, length })
    }

    /// Get the key material as a slice
    fn as_slice(&self) -> &[u8] {
        &self.buffer[..self.length]
    }
}
