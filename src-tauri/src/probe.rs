use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

const PROBE_TIMEOUT: Duration = Duration::from_secs(3);
const CONNECT_TIMEOUT: Duration = Duration::from_millis(800);

#[derive(Debug, Clone, Serialize)]
pub struct ProbeResult {
    pub port: u16,
    pub tcp_open: bool,
    pub tcp_latency_ms: u64,
    pub probes: Vec<ProbeEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeEntry {
    pub protocol: String,
    pub success: bool,
    pub status: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub latency_ms: u64,
    pub error: Option<String>,
}

pub async fn probe(port: u16) -> ProbeResult {
    let addr = format!("127.0.0.1:{port}");

    let t = Instant::now();
    let tcp_open = timeout(CONNECT_TIMEOUT, TcpStream::connect(&addr))
        .await
        .map(|r| r.is_ok())
        .unwrap_or(false);
    let tcp_latency_ms = t.elapsed().as_millis() as u64;

    if !tcp_open {
        return ProbeResult { port, tcp_open: false, tcp_latency_ms, probes: vec![] };
    }

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(PROBE_TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let (http_r, https_r, ws_r, redis_r, pg_r, mysql_r) = tokio::join!(
        probe_http(&client, port, false),
        probe_http(&client, port, true),
        probe_ws(port),
        probe_redis(port),
        probe_postgres(port),
        probe_mysql(port),
    );

    ProbeResult {
        port,
        tcp_open,
        tcp_latency_ms,
        probes: vec![http_r, https_r, ws_r, redis_r, pg_r, mysql_r],
    }
}

async fn probe_http(client: &reqwest::Client, port: u16, tls: bool) -> ProbeEntry {
    let proto = if tls { "https" } else { "http" };
    let url = format!("{proto}://127.0.0.1:{port}/");
    let t = Instant::now();
    match client.get(&url).send().await {
        Ok(resp) => {
            let status = resp.status().to_string();
            let headers: HashMap<String, String> = resp
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();
            ProbeEntry {
                protocol: proto.to_string(),
                success: true,
                status: Some(status),
                headers: Some(headers),
                latency_ms: t.elapsed().as_millis() as u64,
                error: None,
            }
        }
        Err(e) => ProbeEntry {
            protocol: proto.to_string(),
            success: false,
            status: None,
            headers: None,
            latency_ms: t.elapsed().as_millis() as u64,
            error: Some(truncate_err(&e.to_string())),
        },
    }
}

async fn probe_ws(port: u16) -> ProbeEntry {
    let t = Instant::now();
    let request = format!(
        "GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\
         Upgrade: websocket\r\nConnection: Upgrade\r\n\
         Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
         Sec-WebSocket-Version: 13\r\n\r\n"
    );
    match timeout(PROBE_TIMEOUT, async {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).await?;
        stream.write_all(request.as_bytes()).await?;
        let mut buf = vec![0u8; 512];
        let n = stream.read(&mut buf).await?;
        Ok::<_, tokio::io::Error>(buf[..n].to_vec())
    })
    .await
    {
        Ok(Ok(bytes)) => {
            let response = String::from_utf8_lossy(&bytes);
            let first_line = response.lines().next().unwrap_or("").trim().to_string();
            let success = first_line.contains("101");
            ProbeEntry {
                protocol: "websocket".to_string(),
                success,
                status: Some(first_line.clone()),
                headers: None,
                latency_ms: t.elapsed().as_millis() as u64,
                error: if success { None } else { Some("no 101 Switching Protocols".to_string()) },
            }
        }
        Ok(Err(e)) => proto_error("websocket", e.to_string(), t),
        Err(_) => proto_timeout("websocket", t),
    }
}

async fn probe_redis(port: u16) -> ProbeEntry {
    let t = Instant::now();
    match timeout(PROBE_TIMEOUT, async {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).await?;
        stream.write_all(b"*1\r\n$4\r\nPING\r\n").await?;
        let mut buf = vec![0u8; 64];
        let n = stream.read(&mut buf).await?;
        Ok::<_, tokio::io::Error>(buf[..n].to_vec())
    })
    .await
    {
        Ok(Ok(bytes)) => {
            let response = String::from_utf8_lossy(&bytes);
            let success = response.starts_with("+PONG") || response.contains("PONG");
            ProbeEntry {
                protocol: "redis".to_string(),
                success,
                status: Some(response.trim().chars().take(40).collect()),
                headers: None,
                latency_ms: t.elapsed().as_millis() as u64,
                error: if success { None } else { Some("not a Redis PONG response".to_string()) },
            }
        }
        Ok(Err(e)) => proto_error("redis", e.to_string(), t),
        Err(_) => proto_timeout("redis", t),
    }
}

async fn probe_postgres(port: u16) -> ProbeEntry {
    let t = Instant::now();
    // Startup message: total_len(4 BE) + protocol(4: 0x0003 0x0000) + "user\0postgres\0\0"
    let params = b"user\0postgres\0\0";
    let total_len = (4u32 + 4 + params.len() as u32).to_be_bytes();
    let mut msg = Vec::new();
    msg.extend_from_slice(&total_len);
    msg.extend_from_slice(&[0, 3, 0, 0]);
    msg.extend_from_slice(params);

    match timeout(PROBE_TIMEOUT, async {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).await?;
        stream.write_all(&msg).await?;
        let mut buf = vec![0u8; 64];
        let n = stream.read(&mut buf).await?;
        Ok::<_, tokio::io::Error>(buf[..n].to_vec())
    })
    .await
    {
        Ok(Ok(bytes)) if !bytes.is_empty() => {
            // PG responds with 'R' (auth req), 'E' (error), 'N' (notice)
            let first = bytes[0];
            let success = matches!(first, b'R' | b'E' | b'N' | b'S');
            ProbeEntry {
                protocol: "postgresql".to_string(),
                success,
                status: Some(format!("response: 0x{first:02x} ({})", first as char)),
                headers: None,
                latency_ms: t.elapsed().as_millis() as u64,
                error: if success { None } else { Some("unexpected startup response".to_string()) },
            }
        }
        Ok(Ok(_)) => proto_error("postgresql", "empty response".to_string(), t),
        Ok(Err(e)) => proto_error("postgresql", e.to_string(), t),
        Err(_) => proto_timeout("postgresql", t),
    }
}

async fn probe_mysql(port: u16) -> ProbeEntry {
    let t = Instant::now();
    match timeout(PROBE_TIMEOUT, async {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).await?;
        let mut buf = vec![0u8; 128];
        let n = stream.read(&mut buf).await?;
        Ok::<_, tokio::io::Error>(buf[..n].to_vec())
    })
    .await
    {
        Ok(Ok(bytes)) if bytes.len() > 5 => {
            // MySQL greeting: 3-byte len + 1-byte seq(0) + 1-byte protocol(10 or 9)
            let proto_ver = bytes[4];
            let success = proto_ver == 10 || proto_ver == 9;
            let version = if success {
                let available = bytes.len() - 5;
                let end = bytes[5..].iter().position(|&b| b == 0).unwrap_or(available).min(30).min(available);
                String::from_utf8_lossy(&bytes[5..5 + end]).into_owned()
            } else {
                String::new()
            };
            ProbeEntry {
                protocol: "mysql".to_string(),
                success,
                status: Some(if success {
                    format!("MySQL {version}")
                } else {
                    "not MySQL".to_string()
                }),
                headers: None,
                latency_ms: t.elapsed().as_millis() as u64,
                error: if success { None } else { Some("unexpected greeting".to_string()) },
            }
        }
        Ok(Ok(_)) => proto_error("mysql", "empty response".to_string(), t),
        Ok(Err(e)) => proto_error("mysql", e.to_string(), t),
        Err(_) => proto_timeout("mysql", t),
    }
}

fn proto_error(proto: &str, msg: String, t: Instant) -> ProbeEntry {
    ProbeEntry {
        protocol: proto.to_string(),
        success: false,
        status: None,
        headers: None,
        latency_ms: t.elapsed().as_millis() as u64,
        error: Some(msg),
    }
}

fn proto_timeout(proto: &str, t: Instant) -> ProbeEntry {
    ProbeEntry {
        protocol: proto.to_string(),
        success: false,
        status: None,
        headers: None,
        latency_ms: t.elapsed().as_millis() as u64,
        error: Some("timeout".to_string()),
    }
}

fn truncate_err(e: &str) -> String {
    e.chars().take(100).collect()
}
