use anyhow::{Context, Result};
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use russh::{
    client::{Handle, Handler},
    keys::{PrivateKey, PrivateKeyWithHashAlg, PublicKey},
    MethodKind,
};
use russh_config::Config as RusshConfig;
use russh_sftp::client::fs::Metadata;
use russh_sftp::client::SftpSession;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{
    AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter, Lines,
};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::io::{CopyToBytes, SinkWriter, StreamReader};
use tokio_util::sync::PollSender;

pub trait Process {
    fn stdin(&mut self) -> Option<Box<dyn AsyncWrite + Unpin + Send>>;
    fn stdout(&mut self) -> Option<Box<dyn AsyncBufRead + Unpin + Send>>;
    fn stderr(&mut self) -> Option<Box<dyn AsyncBufRead + Unpin + Send>>;
}

impl Process for Child {
    fn stdin(&mut self) -> Option<Box<dyn AsyncWrite + Unpin + Send>> {
        self.stdin.take().map(|x| {
            let x: Box<dyn AsyncWrite + Unpin + Send> = Box::new(BufWriter::new(x));
            x
        })
    }
    fn stdout(&mut self) -> Option<Box<dyn AsyncBufRead + Unpin + Send>> {
        self.stdout.take().map(|x| {
            let x: Box<dyn AsyncBufRead + Unpin + Send> = Box::new(BufReader::new(x));
            x
        })
    }
    fn stderr(&mut self) -> Option<Box<dyn AsyncBufRead + Unpin + Send>> {
        self.stderr.take().map(|x| {
            let x: Box<dyn AsyncBufRead + Unpin + Send> = Box::new(BufReader::new(x));
            x
        })
    }
}

impl Process for SshChild {
    fn stdin(&mut self) -> Option<Box<dyn AsyncWrite + Unpin + Send>> {
        self.stdin.take()
    }
    fn stdout(&mut self) -> Option<Box<dyn AsyncBufRead + Unpin + Send>> {
        self.stdout.take()
    }

    fn stderr(&mut self) -> Option<Box<dyn AsyncBufRead + Unpin + Send>> {
        self.stderr.take()
    }
}

pub enum LocalOrSshProcess {
    Local(Child),
    Ssh(SshChild),
}

impl Process for LocalOrSshProcess {
    fn stdin(&mut self) -> Option<Box<dyn AsyncWrite + Unpin + Send>> {
        match self {
            LocalOrSshProcess::Local(child) => child.stdin(),
            LocalOrSshProcess::Ssh(child) => child.stdin(),
        }
    }
    fn stdout(&mut self) -> Option<Box<dyn AsyncBufRead + Unpin + Send>> {
        match self {
            LocalOrSshProcess::Local(child) => child.stdout(),
            LocalOrSshProcess::Ssh(child) => child.stdout(),
        }
    }
    fn stderr(&mut self) -> Option<Box<dyn AsyncBufRead + Unpin + Send>> {
        match self {
            LocalOrSshProcess::Local(child) => child.stderr(),
            LocalOrSshProcess::Ssh(child) => child.stderr(),
        }
    }
}

#[derive(Clone, Debug, clap::Parser)]
pub struct SshOptions {
    #[arg(long = "ssh-host")]
    pub host: Option<String>,
    #[arg(long = "ssh-target-arch")]
    pub target_arch: Option<String>,
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
        let target_arch = match &self.target_arch {
            Some(target_arch) => target_arch.clone(),
            None => return Err(anyhow::anyhow!("No target arch provided")),
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
            target_arch,
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
    pub target_arch: String,
    pub user: String,
    pub port: u16,
    pub auth_keys: Vec<Arc<PrivateKey>>,
    pub server_public_key: PublicKey,
}

pub struct SshChild {
    // We keep the session alive
    _session: Handle<Client>,
    stdin: Option<Box<dyn AsyncWrite + Unpin + Send>>,
    stdout: Option<Box<dyn AsyncBufRead + Unpin + Send>>,
    stderr: Option<Box<dyn AsyncBufRead + Unpin + Send>>,
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

pub async fn launch_on_remote(
    executable: &Executable<'_>,
    args: Vec<String>,
    ssh_config: &SshConfig,
) -> Result<SshChild> {
    let hash = executable.hash().await?;
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
    upload_executable(&mut session, executable, &remote_path).await?;
    // Launch the binary on the remote host
    let remote_args = args.join(" ");
    let launch_cmd = format!("{:?} {}", remote_path, remote_args);
    info!("Launching: {}", launch_cmd);
    let mut channel = session.channel_open_session().await?;
    channel.exec(true, launch_cmd).await?;

    let (stdout_tx, stdout_rx) = mpsc::channel::<Bytes>(100);
    let (stderr_tx, stderr_rx) = mpsc::channel::<Bytes>(100);
    let (stdin_tx, mut stdin_rx) = mpsc::channel::<Bytes>(100);

    // Spawn a task to pump bytes into the appropriate io stream
    tokio::spawn(async move {
        use russh::ChannelMsg;
        // use select to handle stdin and channel messages
        loop {
            tokio::select! {
                buf = stdin_rx.recv() => match buf {
                    Some(buf) => {
                        debug!("Sending {} bytes to worker", buf.len());
                        match channel.data(&buf[..]).await {
                            Ok(_) => (),
                            Err(_) => break,
                        }
                    }
                    None => break,
                },
                msg = channel.wait() => match msg {
                    Some(msg) =>  match msg {
                        ChannelMsg::Data { ref data } => {
                            let _ = stdout_tx.send(data.to_vec().into()).await;
                        }
                        ChannelMsg::ExtendedData { ref data, ext } => {
                            if ext == 1 {
                                let _ = stderr_tx.send(data.to_vec().into()).await;
                            }
                        }
                        ChannelMsg::ExitStatus { exit_status } => {
                            debug!("Remote process exited with: {}", exit_status);
                            return;
                        }
                        ChannelMsg::Eof => {
                            debug!("Remote process sent EOF, shutting down channel handler.");
                            return;
                        }
                        ChannelMsg::Close => {
                            debug!("Remote process closed channel, shutting down channel handler.");
                            return;
                        }
                        _ => (),
                    },
                    None => {
                        debug!("Channel closed, shutting down channel handler.");
                        return;
                    },
                }
            }
        }
        // Read only from the channel, not from stdin (stdin has been dropped)
        loop {
            match channel.wait().await {
                Some(msg) => match msg {
                    ChannelMsg::Data { ref data } => {
                        let _ = stdout_tx.send(data.to_vec().into()).await;
                    }
                    ChannelMsg::ExtendedData { ref data, ext } => {
                        if ext == 1 {
                            let _ = stderr_tx.send(data.to_vec().into()).await;
                        }
                    }
                    ChannelMsg::ExitStatus { exit_status } => {
                        debug!("Remote process exited with: {}", exit_status);
                        return;
                    }
                    ChannelMsg::Eof => {
                        debug!("Remote process sent EOF, shutting down channel handler.");
                        return;
                    }
                    ChannelMsg::Close => {
                        debug!("Remote process closed channel, shutting down channel handler.");
                        return;
                    }
                    _ => (),
                },
                None => {
                    debug!("Channel closed, shutting down channel handler.");
                    return;
                }
            }
        }
    });
    let stdin_sink = PollSender::new(stdin_tx)
        .sink_map_err(|_| std::io::Error::from(std::io::ErrorKind::BrokenPipe));

    let stdin = BufWriter::new(SinkWriter::new(CopyToBytes::new(stdin_sink)));
    let stdout = BufReader::new(StreamReader::new(
        ReceiverStream::new(stdout_rx).map(|s| Ok::<_, std::io::Error>(s)),
    ));
    let stderr = BufReader::new(StreamReader::new(
        ReceiverStream::new(stderr_rx).map(|s| Ok::<_, std::io::Error>(s)),
    ));
    Ok(SshChild {
        _session: session,
        stdin: Some(Box::new(stdin)),
        stdout: Some(Box::new(stdout)),
        stderr: Some(Box::new(stderr)),
    })
}

async fn authenticate(session: &mut Handle<Client>, ssh_config: &SshConfig) -> Result<()> {
    use russh::client::AuthResult;
    // Try no authentication first
    let mut methods = match session.authenticate_none(&ssh_config.user).await? {
        AuthResult::Success => {
            warn!("No authentication required");
            return Ok(());
        }
        AuthResult::Failure {
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
                AuthResult::Success => {
                    debug!("Authenticated using private key.");
                    return Ok(());
                }
                AuthResult::Failure {
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

async fn upload_executable(
    session: &mut Handle<Client>,
    executable: &Executable<'_>,
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
    // Create all parents that do not exist
    let parent_path = remote_path.parent().unwrap().to_string_lossy();
    if !sftp.try_exists(parent_path).await? {
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
    }
    debug!("Creating remote file: {}", remote_path_str);
    // Create the file itself
    let mut remote_file = sftp.create(remote_path_str.to_string()).await?;
    match executable {
        Executable::Path(path) => {
            let mut file = tokio::fs::File::open(path).await?;
            debug!("Uploading local file: {}", path.display());
            tokio::io::copy(&mut file, &mut remote_file).await?;
        }
        Executable::Buffer(bytes) => {
            debug!("Uploading in-memory buffer");
            remote_file.write_all(&bytes[..]).await?;
        }
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

pub enum Executable<'a> {
    Path(PathBuf),
    Buffer(Cow<'a, [u8]>),
}

impl<'a> Executable<'a> {
    pub async fn local_path(&self) -> Result<PathBuf> {
        match self {
            Executable::Path(path) => Ok(path.clone()),
            Executable::Buffer(buffer) => {
                let hash = self.hash().await?;
                let path = std::env::home_dir()
                    .expect("Home directory not found")
                    .join(".cache/slurm-webapp/worker-{}".replace("{}", &hash));
                if path.exists() {
                    return Ok(path);
                } else {
                    tokio::fs::create_dir_all(path.parent().unwrap()).await?;
                    let mut file = tokio::fs::File::create(&path).await?;
                    file.write_all(&buffer[..]).await?;
                    file.set_permissions(Permissions::from_mode(0o755)).await?;
                    Ok(path)
                }
            }
        }
    }
    pub async fn hash(&self) -> Result<String> {
        let mut hasher = Sha256::new();
        match self {
            Executable::Path(path) => {
                let file = tokio::fs::File::open(path).await?;
                let mut reader = tokio::io::BufReader::new(file);
                loop {
                    let len = {
                        let chunk = reader.fill_buf().await?;
                        if chunk.is_empty() {
                            break;
                        }
                        hasher.update(chunk);
                        chunk.len()
                    };
                    reader.consume(len);
                }
            }
            Executable::Buffer(bytes) => {
                hasher.update(bytes);
            }
        }
        Ok(hex::encode(hasher.finalize()))
    }
}

pub trait Target {
    #[allow(async_fn_in_trait)]
    async fn lookup<'s>(&'s self, arch: &str) -> Result<Executable<'s>>;
}

pub struct CargoTarget {
    pub package: String,
    pub binary: String,
    pub cwd: PathBuf,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
#[serde(tag = "reason")]
enum BuildOutput {
    #[serde(rename = "build-finished")]
    BuildFinished,
    #[serde(rename = "compiler-artifact")]
    CompilerArtifact {
        package_id: String,
        target: serde_json::Value,
        profile: serde_json::Value,
        filenames: Vec<String>,
        executable: Option<String>,
    },
    #[serde(other)]
    Other,
}

impl Target for CargoTarget {
    async fn lookup(&self, arch: &str) -> Result<Executable<'_>> {
        let mut command = Command::new("cargo");
        command
            .arg("build")
            .arg("--package")
            .arg(&self.package)
            .arg("--bin")
            .arg(&self.binary)
            .arg("--target")
            .arg(arch)
            .arg("--release")
            .arg("--message-format=json")
            .current_dir(&self.cwd)
            .stdout(Stdio::piped());

        info!("Executing cargo build command");

        let mut child = command
            .spawn()
            .context("Failed to spawn cargo build process")?;
        let stdout = child.stdout.take().context("Failed to open stdout")?;
        let mut reader = BufReader::new(stdout).lines();

        let mut last_artifact = None;
        while let Some(line) = reader.next_line().await? {
            let output = serde_json::from_str::<BuildOutput>(&line)?;
            match output {
                BuildOutput::CompilerArtifact { .. } => last_artifact = Some(output),
                BuildOutput::BuildFinished => break,
                _ => (),
            }
        }
        if let Some(BuildOutput::CompilerArtifact { executable, .. }) = last_artifact {
            return Ok(Executable::Path(PathBuf::from(
                executable.context("No executable found")?,
            )));
        } else {
            return Err(anyhow::anyhow!("No executable foundin build output"));
        }
    }
}

pub struct Worker {
    // Keep so that we don't drop the child process
    #[allow(unused)]
    child: LocalOrSshProcess,
    stdout: Lines<Box<dyn AsyncBufRead + Send + Unpin>>,
    stderr: Lines<Box<dyn AsyncBufRead + Send + Unpin>>,
    // the parse buffer
    buffer: Option<String>,
}

const TARGET: &str = env!("TARGET");

impl Worker {
    pub async fn launch<T: Target, I: Serialize>(
        target: T,
        args: impl IntoIterator<Item = String>,
        // If None, launch on localhost, otherwise
        // will try to launch on the remote host.
        ssh_config: Option<&SshConfig>,
        input: Option<I>,
    ) -> Result<Worker> {
        let mut child = match ssh_config {
            Some(ssh_config) => {
                let executable = target.lookup(ssh_config.target_arch.as_str()).await?;
                LocalOrSshProcess::Ssh(
                    launch_on_remote(&executable, args.into_iter().collect(), ssh_config).await?,
                )
            }
            None => {
                let executable = target.lookup(TARGET).await?;
                let path = executable.local_path().await?;
                println!("Found executable: {:#?}", path);
                let mut command = Command::new(path);
                command.args(args);
                let child = command.spawn()?;
                LocalOrSshProcess::Local(child)
            }
        };
        let stdout = child.stdout().context("Failed to open stdout")?.lines();
        let stderr = child.stderr().context("Failed to open stderr")?.lines();
        let mut stdin = child.stdin().context("Failed to open stdin")?;
        if let Some(input) = input {
            stdin.write_all(&serde_json::to_vec(&input)?).await?;
            stdin.write(&[b'\n']).await?;
            stdin.flush().await?;
        }
        Ok(Worker {
            child,
            stdout,
            stderr,
            buffer: None,
        })
    }
    pub async fn next<'de, O: Deserialize<'de>>(&'de mut self) -> Option<O> {
        let line = loop {
            let line = tokio::select! {
                line = self.stdout.next_line() => match line {
                    Ok(Some(line)) => Some(line),
                    Ok(None) | Err(_) => return None,
                },
                line = self.stderr.next_line() => match line {
                    Ok(Some(line)) => {
                        info!("Worker: {}", line);
                        continue;
                    },
                    Ok(None) | Err(_) => return None,
                },
            };
            if let Some(line) = line {
                break line;
            }
        };
        self.buffer = Some(line);
        if let Some(line) = &self.buffer {
            match serde_json::from_str(line) {
                Ok(value) => Some(value),
                Err(_) => {
                    error!("Failed to parse line: {}", line);
                    None
                }
            }
        } else {
            panic!("Should be unreachable!")
        }
    }
}
