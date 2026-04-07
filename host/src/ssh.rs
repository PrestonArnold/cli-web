use anyhow::Result;
use russh::keys::{Algorithm, PrivateKey};
use russh::keys::key::safe_rng;
use russh::server::{self, Auth, Server, Session};
use russh::{Channel, ChannelId};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::runtime::WasmRuntime;

struct SshHandler {
    rt: Arc<Mutex<WasmRuntime>>,
    buf: Vec<u8>,
}

impl server::Handler for SshHandler {
    type Error = anyhow::Error;

    async fn auth_password(&mut self, _user: &str, _pass: &str) -> Result<Auth> {
        Ok(Auth::Accept)
    }

    async fn auth_none(&mut self, _user: &str) -> Result<Auth> {
        Ok(Auth::Accept)
    }

    async fn channel_open_session(
        &mut self,
        _channel: Channel<server::Msg>,
        _session: &mut Session,
    ) -> Result<bool> {
        Ok(true)
    }

    async fn pty_request(
        &mut self,
        channel: ChannelId,
        _term: &str,
        _col_width: u32,
        _row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _modes: &[(russh::Pty, u32)],
        session: &mut Session,
    ) -> Result<()> {
        session.channel_success(channel);
        Ok(())
    }

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<()> {
        session.channel_success(channel);
        session.data(
            channel,
            bytes::Bytes::from_static(b"\x1b[2J\x1b[HWelcome to MyShell!\r\n\r\n> "),
        )?;
        Ok(())
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<()> {
        for &byte in data {
            if byte == b'\r' || byte == b'\n' {
                let line = String::from_utf8_lossy(&self.buf).trim().to_string();
                self.buf.clear();

                if line.is_empty() {
                    session.data(channel, bytes::Bytes::from_static(b"\r\n> "))?;
                    continue;
                }

                if line == "exit" {
                    session.close(channel);
                    return Ok(());
                }

                let parts: Vec<&str> = line.split_whitespace().collect();

                let mut rt = self.rt.lock().await;
                let output = rt
                    .run(parts[0], &parts[1..])
                    .unwrap_or_else(|e| e.to_string());

                let msg = format!("\r\n{}\r\n> ", output);
                session.data(channel, bytes::Bytes::from(msg.into_bytes()))?;
            } else {
                session.data(channel, bytes::Bytes::copy_from_slice(&[byte]))?;
                self.buf.push(byte);
            }
        }
        Ok(())
    }
}

struct SshServer {
    rt: Arc<Mutex<WasmRuntime>>,
}

impl server::Server for SshServer {
    type Handler = SshHandler;

    fn new_client(&mut self, _addr: Option<std::net::SocketAddr>) -> SshHandler {
        SshHandler {
            rt: Arc::clone(&self.rt),
            buf: Vec::new(),
        }
    }
}

pub async fn serve(rt: WasmRuntime) -> Result<()> {
    let mut rng = safe_rng();
    let key = PrivateKey::random(&mut rng, Algorithm::Ed25519)?;

    let config = Arc::new(server::Config {
        auth_rejection_time: std::time::Duration::from_secs(0),
        keys: vec![key],
        ..Default::default()
    });

    let rt = Arc::new(Mutex::new(rt));
    let mut server = SshServer { rt };

    println!("SSH server running on 0.0.0.0:2222");

    server.run_on_address(config, ("0.0.0.0", 2222)).await?;

    Ok(())
}