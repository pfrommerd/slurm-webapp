use anyhow::{Context, Result};
use log::{debug, info, warn};
use russh::client::{Handle, Handler};
use russh::keys::{PrivateKey, PrivateKeyWithHashAlg, PublicKey};
use russh::MethodKind;
use russh_config::Config as RusshConfig;
use russh_sftp::client::fs::Metadata;
use russh_sftp::client::SftpSession;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll};
use tokio::io::AsyncRead;
use tokio::io::{AsyncBufRead, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Child;
use tokio::sync::mpsc;

pub trait Process {
    fn stdout(&mut self) -> Option<Box<dyn AsyncBufRead + Unpin + Send>>;
    fn stderr(&mut self) -> Option<Box<dyn AsyncBufRead + Unpin + Send>>;
}

impl Process for Child {
    fn stdout(&mut self) -> Option<Box<dyn AsyncBufRead + Unpin + Send>> {
        self.stdout
            .take()
            .map(|s| Box::new(BufReader::new(s)) as Box<dyn AsyncBufRead + Unpin + Send>)
    }

    fn stderr(&mut self) -> Option<Box<dyn AsyncBufRead + Unpin + Send>> {
        self.stderr
            .take()
            .map(|s| Box::new(BufReader::new(s)) as Box<dyn AsyncBufRead + Unpin + Send>)
    }
}

#[derive(Clone, Debug, clap::Parser)]
pub struct SshOptions {
    #[arg(long = "ssh-host")]
    pub host: Option<String>,
    #[arg(long = "ssh-server-public-key")]
    pub server_public_key: Option<String>,
    #[arg(long = "ssh-user")]
    pub user: Option<String>,
    #[arg(long = "ssh-port")]
    pub port: Option<u16>,
    #[arg(long = "ssh-key")]
    pub key_path: Option<PathBuf>,
}

impl SshOptions {
    #[rustfmt::skip]
    pub fn resolve(&self) -> Result<Option<SshConfig>> {
        let host = match &self.host {
            Some(host) => host,
            None => return Ok(None),
        };
        // TODO: Handle config matching better than this
        // i.e. proxyjump, etc.
        let config = match russh_config::parse_home(host.as_str()) {
            Ok(c) => c,
            Err(_) => RusshConfig::default(host.as_str()),
        };
        let host = config.host().to_string();
        let user = self.user.clone().unwrap_or(config.user());
        let port = self.port.unwrap_or(config.port());
        // TODO: somehow get the IdentityFile from the config
        let mut key_paths = vec![
            dirs::home_dir().unwrap().join(".ssh/id_rsa"),
            dirs::home_dir().unwrap().join(".ssh/id_ed25519"),
        ];
        if let Some(key_path) = &self.key_path {
            key_paths.clear();
            key_paths.push(key_path.clone());
        }
        let auth_keys = key_paths
            .into_iter()
            .filter_map(|path| russh::keys::load_secret_key(path, None).ok().map(Arc::new))
            .collect::<Vec<Arc<PrivateKey>>>();

        let server_public_key = match &self.server_public_key {
            Some(key) => PublicKey::from_openssh(key)
                .context("Failed to parse provided server public key")?,
            None => {
                // Read in ~/.ssh/known_hosts, find the first key that matches
                let known_hosts =
                    std::fs::read_to_string(dirs::home_dir().unwrap().join(".ssh/known_hosts"))
                        .unwrap_or_default();
                let mut key = None;
                for l in known_hosts.lines() {
                    if l.starts_with(&host) {
                        if let Some(key_part) = l.find(' ').map(|i| (&l[i..]).trim()) {
                            key = Some(PublicKey::from_openssh(key_part).with_context(|| {
                                format!("Failed to parse server public key in known_hosts file: {}", key_part)
                            })?);
                            break;
                        }
                    };
                }
                key.ok_or(anyhow::anyhow!(
                    "No server public key found for host {}",
                    host
                ))?
            }
        };
        Ok(Some(SshConfig {
            host,
            user,
            port,
            auth_keys,
            server_public_key,
        }))
    }
}

#[derive(Clone, Debug)]
pub struct SshConfig {
    pub host: String,
    pub user: String,
    pub port: u16,
    pub auth_keys: Vec<Arc<PrivateKey>>,
    pub server_public_key: PublicKey,
}

pub struct SshChild {
    // We keep the session alive
    _session: Handle<Client>,
    stdout: Option<Box<dyn AsyncBufRead + Unpin + Send>>,
    stderr: Option<Box<dyn AsyncBufRead + Unpin + Send>>,
}

impl Process for SshChild {
    fn stdout(&mut self) -> Option<Box<dyn AsyncBufRead + Unpin + Send>> {
        self.stdout.take()
    }

    fn stderr(&mut self) -> Option<Box<dyn AsyncBufRead + Unpin + Send>> {
        self.stderr.take()
    }
}

struct Client {
    server_public_key: PublicKey,
}

impl Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(&self.server_public_key == server_public_key)
    }
}

// Better ChannelStream
struct ByteStream {
    receiver: mpsc::Receiver<Vec<u8>>,
    current_chunk: Option<Vec<u8>>,
    position: usize,
}

impl ByteStream {
    fn new(receiver: mpsc::Receiver<Vec<u8>>) -> Self {
        Self {
            receiver,
            current_chunk: None,
            position: 0,
        }
    }
}

impl AsyncRead for ByteStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        loop {
            if let Some(chunk) = &self.current_chunk {
                if self.position < chunk.len() {
                    let len = std::cmp::min(buf.remaining(), chunk.len() - self.position);
                    buf.put_slice(&chunk[self.position..self.position + len]);
                    self.position += len;
                    return Poll::Ready(Ok(()));
                } else {
                    self.current_chunk = None;
                    self.position = 0;
                }
            }

            match self.receiver.poll_recv(cx) {
                Poll::Ready(Some(chunk)) => {
                    self.current_chunk = Some(chunk);
                    self.position = 0;
                }
                Poll::Ready(None) => return Poll::Ready(Ok(())), // EOF
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

pub async fn launch_on_remote(
    executable: PathBuf,
    args: Vec<String>,
    ssh_config: &SshConfig,
) -> Result<SshChild> {
    let hash = compute_binary_hash(&executable).await?;
    debug!("Worker binary hash: {}", hash);

    let config = russh::client::Config {
        inactivity_timeout: None,
        preferred: russh::Preferred {
            kex: std::borrow::Cow::Owned(vec![
                russh::kex::CURVE25519_PRE_RFC_8731,
                russh::kex::EXTENSION_SUPPORT_AS_CLIENT,
            ]),
            ..Default::default()
        },
        ..<_>::default()
    };
    let config = Arc::new(config);
    let sh = Client {
        server_public_key: ssh_config.server_public_key.clone(),
    };
    info!(
        "Connecting to {}:{} as {}",
        &ssh_config.host, ssh_config.port, &ssh_config.user
    );
    let mut session =
        russh::client::connect(config, (ssh_config.host.as_str(), ssh_config.port), sh).await?;
    authenticate(&mut session, &ssh_config).await?;
    // Upload the binary to the remote host, if it doesn't exist
    let remote_path = PathBuf::from(format!(".cache/slurm-webapp/worker-{}", hash));
    upload_file(&mut session, &executable, &remote_path).await?;
    // Launch the binary on the remote host
    let remote_args = args.join(" ");
    let launch_cmd = format!("{:?} {}", remote_path, remote_args);
    info!("Launching: {}", launch_cmd);
    let mut channel = session.channel_open_session().await?;
    channel.exec(true, launch_cmd).await?;

    let (stdout_tx, stdout_rx) = mpsc::channel(100);
    let (stderr_tx, stderr_rx) = mpsc::channel(100);

    // Spawn a task to pump bytes into the appropriate io stream
    tokio::spawn(async move {
        use russh::ChannelMsg;
        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { ref data } => {
                    let _ = stdout_tx.send(data.to_vec()).await;
                }
                ChannelMsg::ExtendedData { ref data, ext } => {
                    if ext == 1 {
                        // 1 is stderr
                        let _ = stderr_tx.send(data.to_vec()).await;
                    }
                }
                ChannelMsg::ExitStatus { exit_status } => {
                    debug!("Remote process exited with: {}", exit_status);
                    // We should probably close streams
                    break;
                }
                ChannelMsg::Eof => break,
                ChannelMsg::Close => break,
                _ => (),
            }
        }
    });

    Ok(SshChild {
        _session: session,
        stdout: Some(Box::new(BufReader::new(ByteStream::new(stdout_rx)))),
        stderr: Some(Box::new(BufReader::new(ByteStream::new(stderr_rx)))),
    })
}

async fn authenticate(session: &mut Handle<Client>, ssh_config: &SshConfig) -> Result<()> {
    use russh::client::AuthResult::*;
    // Try no authentication first
    let mut methods = match session.authenticate_none(&ssh_config.user).await? {
        Success => {
            warn!("No authentication required");
            return Ok(());
        }
        Failure {
            remaining_methods, ..
        } => remaining_methods,
    };
    if methods.contains(&MethodKind::PublicKey) {
        let hash_alg = session
            .best_supported_rsa_hash()
            .await
            .ok()
            .flatten()
            .flatten();
        for key in &ssh_config.auth_keys {
            if !methods.contains(&MethodKind::PublicKey) {
                break;
            }
            match session
                .authenticate_publickey(
                    &ssh_config.user,
                    PrivateKeyWithHashAlg::new(key.clone(), hash_alg),
                )
                .await?
            {
                Success => {
                    debug!("Authenticated using private key.");
                    return Ok(());
                }
                Failure {
                    partial_success,
                    remaining_methods,
                } => {
                    methods = remaining_methods;
                    if partial_success {
                        break;
                    }
                }
            }
        }
    }
    Err(anyhow::anyhow!("Authentication failed"))
}

async fn upload_file(
    session: &mut Handle<Client>,
    local_path: &Path,
    remote_path: &Path,
) -> Result<()> {
    let channel = session
        .channel_open_session()
        .await
        .context("Failed to open SSH channel")?;
    channel
        .request_subsystem(true, "sftp")
        .await
        .context("SFTP subsystem unavailable.")?;
    let sftp = SftpSession::new(channel.into_stream())
        .await
        .context("Failed to create SFTP session")?;
    // Check if file exists using SFTP
    // We can use metadata() or try to open it.
    let remote_path_str = remote_path.to_string_lossy();
    if sftp.try_exists(remote_path_str.to_string()).await? {
        info!("File exists on remote, skipping upload");
        return Ok(());
    }
    info!("Starting SFTP upload to {:?}", remote_path);
    let mut file = tokio::fs::File::open(local_path).await?;
    // Create all parents that do not exist
    let mut ancestors = remote_path.ancestors().collect::<Vec<_>>();
    ancestors.pop(); // Do not create the root directory.
    ancestors.reverse();
    ancestors.pop(); // Do not create the path itself
    for parent in ancestors {
        let parent_str = parent.to_string_lossy();
        debug!("Checking directory: {}", parent_str);
        if !sftp.try_exists(parent_str.to_string()).await? {
            debug!("Creating directory: {}", parent_str);
            sftp.create_dir(parent_str).await?;
        }
    }
    debug!("Creating remote file: {}", remote_path_str);
    // Create the file itself
    let mut remote_file = sftp.create(remote_path_str.to_string()).await?;
    let mut buffer = vec![0u8; 4 * 1024 * 1024]; // 4MB buffer
    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        remote_file.write_all(&buffer[..n]).await?;
    }
    remote_file.shutdown().await?;
    std::mem::drop(remote_file);
    let metadata = Metadata {
        permissions: Some(0o755),
        size: None,
        user: None,
        uid: None,
        group: None,
        gid: None,
        atime: None,
        mtime: None,
        ..Default::default()
    };
    sftp.set_metadata(remote_path_str.to_string(), metadata.clone())
        .await
        .with_context(|| format!("Failed to change file permissions {:?}", metadata))?;
    // Make the file executable
    Ok(())
}

async fn compute_binary_hash(path: &Path) -> Result<String> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];
    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let result = hasher.finalize();
    Ok(hex::encode(result))
}
