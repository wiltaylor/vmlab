//! QEMU stream-netdev framing codec.
//!
//! QEMU's `-netdev stream` over SOCK_STREAM prefixes every ethernet frame
//! with a 4-byte big-endian length. This module reads and writes that
//! framing over any `AsyncRead`/`AsyncWrite`.

use std::io;

use bytes::Bytes;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Sanity cap on a single frame: 64 KiB of L3 payload + ethernet header +
/// slack for the length prefix. Anything larger is a corrupt stream.
pub const MAX_FRAME_LEN: usize = 65536 + 14 + 4;

/// Read one length-prefixed frame. Returns `Ok(None)` on clean EOF (stream
/// closed exactly on a frame boundary); EOF mid-prefix or mid-body is an
/// `UnexpectedEof` error; a length above [`MAX_FRAME_LEN`] is `InvalidData`.
pub async fn read_frame<R: AsyncRead + Unpin>(r: &mut R) -> io::Result<Option<Bytes>> {
    let mut len_buf = [0u8; 4];
    let mut filled = 0;
    while filled < len_buf.len() {
        let n = r.read(&mut len_buf[filled..]).await?;
        if n == 0 {
            if filled == 0 {
                return Ok(None); // clean EOF on a frame boundary
            }
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "stream closed mid length prefix",
            ));
        }
        filled += n;
    }
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > MAX_FRAME_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("frame length {len} exceeds cap {MAX_FRAME_LEN}"),
        ));
    }
    let mut frame = vec![0u8; len];
    r.read_exact(&mut frame).await?;
    Ok(Some(Bytes::from(frame)))
}

/// Write one frame with its 4-byte big-endian length prefix and flush.
pub async fn write_frame<W: AsyncWrite + Unpin>(w: &mut W, frame: &[u8]) -> io::Result<()> {
    if frame.len() > MAX_FRAME_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("frame length {} exceeds cap {MAX_FRAME_LEN}", frame.len()),
        ));
    }
    let len = frame.len() as u32;
    w.write_all(&len.to_be_bytes()).await?;
    w.write_all(frame).await?;
    w.flush().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn roundtrip_multiple_frames() {
        let (mut a, mut b) = tokio::io::duplex(4096);
        let frames: [&[u8]; 3] = [b"first", b"", b"third frame with more bytes"];
        for f in frames {
            write_frame(&mut a, f).await.unwrap();
        }
        for f in frames {
            let got = read_frame(&mut b).await.unwrap().unwrap();
            assert_eq!(&got[..], f);
        }
    }

    #[tokio::test]
    async fn clean_eof_returns_none() {
        let (mut a, mut b) = tokio::io::duplex(64);
        write_frame(&mut a, b"only").await.unwrap();
        drop(a);
        assert_eq!(&read_frame(&mut b).await.unwrap().unwrap()[..], b"only");
        assert!(read_frame(&mut b).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn dribbled_bytes_reassemble() {
        // Feed the encoded stream one byte at a time to exercise partial
        // reads of both the prefix and the body.
        let (mut a, mut b) = tokio::io::duplex(1);
        let payload = b"dribble me gently".to_vec();
        let writer = tokio::spawn(async move {
            let mut encoded = (payload.len() as u32).to_be_bytes().to_vec();
            encoded.extend_from_slice(&payload);
            for byte in encoded {
                a.write_all(&[byte]).await.unwrap();
                a.flush().await.unwrap();
                tokio::task::yield_now().await;
            }
            drop(a);
        });
        let got = read_frame(&mut b).await.unwrap().unwrap();
        assert_eq!(&got[..], b"dribble me gently");
        assert!(read_frame(&mut b).await.unwrap().is_none());
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn eof_mid_prefix_is_error() {
        let (mut a, mut b) = tokio::io::duplex(64);
        a.write_all(&[0x00, 0x00]).await.unwrap();
        drop(a);
        let err = read_frame(&mut b).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[tokio::test]
    async fn eof_mid_body_is_error() {
        let (mut a, mut b) = tokio::io::duplex(64);
        a.write_all(&10u32.to_be_bytes()).await.unwrap();
        a.write_all(b"short").await.unwrap();
        drop(a);
        let err = read_frame(&mut b).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[tokio::test]
    async fn oversize_length_is_error() {
        let (mut a, mut b) = tokio::io::duplex(64);
        a.write_all(&(MAX_FRAME_LEN as u32 + 1).to_be_bytes())
            .await
            .unwrap();
        let err = read_frame(&mut b).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[tokio::test]
    async fn write_rejects_oversize_frame() {
        let (mut a, _b) = tokio::io::duplex(64);
        let huge = vec![0u8; MAX_FRAME_LEN + 1];
        let err = write_frame(&mut a, &huge).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[tokio::test]
    async fn max_size_frame_roundtrips() {
        let (mut a, mut b) = tokio::io::duplex(MAX_FRAME_LEN + 4);
        let frame = vec![0xA5u8; MAX_FRAME_LEN];
        write_frame(&mut a, &frame).await.unwrap();
        let got = read_frame(&mut b).await.unwrap().unwrap();
        assert_eq!(got.len(), MAX_FRAME_LEN);
        assert!(got.iter().all(|&x| x == 0xA5));
    }
}
