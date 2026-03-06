use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Debug)]
pub enum TpmRequest {
    Ping,
    Store { key: Vec<u8> },
    Retrieve,
    Delete,
    UpdateDataDir { path: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TpmResponse {
    Ok { key: Option<Vec<u8>> },
    Err(String),
}

/// Send a length-prefixed JSON message over a writer
pub fn send_message<W: Write>(writer: &mut W, msg: &impl Serialize) -> Result<(), String> {
    let bytes = serde_json::to_vec(msg).map_err(|e| e.to_string())?;
    let len = bytes.len() as u32;
    writer
        .write_all(&len.to_le_bytes())
        .map_err(|e| e.to_string())?;
    writer.write_all(&bytes).map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;
    Ok(())
}

/// Receive a length-prefixed JSON message from a reader
pub fn receive_message<R: Read, T: for<'a> Deserialize<'a>>(reader: &mut R) -> Result<T, String> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).map_err(|e| e.to_string())?;
    let len = u32::from_le_bytes(len_buf) as usize;

    let mut bytes = vec![0u8; len];
    reader.read_exact(&mut bytes).map_err(|e| e.to_string())?;

    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

#[cfg(feature = "tpm")]
pub mod async_io {
    use super::*;
    use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

    pub async fn send_message_async<W: AsyncWrite + Unpin>(
        writer: &mut W,
        msg: &impl Serialize,
    ) -> Result<(), String> {
        let bytes = serde_json::to_vec(msg).map_err(|e| e.to_string())?;
        let len = bytes.len() as u32;
        writer
            .write_all(&len.to_le_bytes())
            .await
            .map_err(|e| e.to_string())?;
        writer.write_all(&bytes).await.map_err(|e| e.to_string())?;
        writer.flush().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn receive_message_async<R: AsyncRead + Unpin, T: for<'a> Deserialize<'a>>(
        reader: &mut R,
    ) -> Result<T, String> {
        let mut len_buf = [0u8; 4];
        reader
            .read_exact(&mut len_buf)
            .await
            .map_err(|e| e.to_string())?;
        let len = u32::from_le_bytes(len_buf) as usize;

        let mut bytes = vec![0u8; len];
        reader
            .read_exact(&mut bytes)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::from_slice(&bytes).map_err(|e| e.to_string())
    }
}
