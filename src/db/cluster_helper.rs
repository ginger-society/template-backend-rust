use openssh::{KnownHosts, Session, Stdio};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    time::{timeout, Duration},
};

pub async fn create_execute_ssh_script(
    ssh_host: &str,
    ssh_user: &str,
    script_path: &str,
) -> Result<(), String> {
    // ✅ Establish SSH connection
    let session = match Session::connect(format!("{}@{}", ssh_user, ssh_host), KnownHosts::Accept).await {
        Ok(session) => session,
        Err(err) => return Err(format!("❌ SSH connection failed: {:?}", err)),
    };

    println!("✅ Connected to {}", ssh_host);

    // ✅ Run the shell script remotely with a 5-minute timeout
    let mut child = match session
        .command("bash")
        .arg(script_path)
        .stdin(Stdio::null()) // Ensure stdin is null
        .stdout(Stdio::piped()) // Capture stdout
        .stderr(Stdio::piped()) // Capture stderr
        .spawn()
        .await
    {
        Ok(child) => child,
        Err(err) => return Err(format!("❌ Failed to execute script: {:?}", err)),
    };

    let stdout = child.stdout().take().ok_or("❌ Failed to capture stdout")?;
    let stderr = child.stderr().take().ok_or("❌ Failed to capture stderr")?;

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let stdout_task = tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            println!("📜 [STDOUT] {}", line);
        }
    });

    let stderr_task = tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            eprintln!("⚠️  [STDERR] {}", line);
        }
    });

    // ✅ Apply a timeout for the script execution (5 minutes)
    match timeout(Duration::from_secs(300), async {
        stdout_task.await.ok();
        stderr_task.await.ok();
        child.wait().await.ok();
    })
    .await
    {
        Ok(_) => Ok(()),
        Err(_) => Err("⏳ SSH command timed out after 5 minutes".to_string()),
    }
}
