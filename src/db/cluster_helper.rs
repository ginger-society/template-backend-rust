use diesel::{r2d2::PooledConnection, PgConnection};
use diesel::prelude::*;
use openssh::{KnownHosts, Session, Stdio};
use r2d2_redis::redis::Commands;
use r2d2_redis::RedisConnectionManager;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    time::{timeout, Duration},
};

use crate::models::schema::Compute_Unit;

pub async fn create_execute_ssh_script(
    ssh_host: &str,
    ssh_user: &str,
    script_path: &str,
    cluster_name: &str,
    cpus: i64,
    memory: &str,
    disk_size: &str,
) -> Result<(), String> {
    // ✅ Establish SSH connection
    let session = match Session::connect(format!("{}@{}", ssh_user, ssh_host), KnownHosts::Accept).await {
        Ok(session) => session,
        Err(err) => return Err(format!("❌ SSH connection failed: {:?}", err)),
    };

    println!("✅ Connected to {}", ssh_host);

    // ✅ Format the command with the arguments safely
    let command = format!(
        "bash {} -n '{}' -c {} -m '{}' -d '{}'",
        script_path, cluster_name, cpus, memory, disk_size
    );

    println!("{:?}" , command);

    // ✅ Run the shell script remotely with a 5-minute timeout
    let mut child = match session
        .command("sh")
        .arg("-c") // Allows passing multiple arguments as a single string
        .arg(&command)
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
        match child.wait().await {
            Ok(exit_status) => {
                if !exit_status.success() {
                    return Err(format!("❌ Script exited with status: {:?}", exit_status));
                }
            }
            Err(err) => return Err(format!("❌ Failed to get script exit status: {:?}", err)),
        }
        Ok(())
    })
    .await
    {
        Ok(result) => result,
        Err(_) => Err("⏳ SSH command timed out after 5 minutes".to_string()),
    }
}


/// **Fetch the first available compute unit that is not locked in Redis**
pub fn get_available_compute_unit(
    conn: &mut PgConnection,
    cache_conn: &mut PooledConnection<RedisConnectionManager>,
    region: &str,
) -> Result<Option<Compute_Unit>, diesel::result::Error> {
    use crate::models::schema::schema::compute_unit::dsl::*;

    let mut excluded_ids = vec![];

    loop {
        // Query the first available compute unit in the region
        let compute_unit_result = compute_unit
            .filter(available.eq(true))
            .filter(region_code.eq(region))
            .filter(id.ne_all(&excluded_ids)) // Exclude locked units
            .first::<Compute_Unit>(conn)
            .optional()?;

        if let Some(unit) = compute_unit_result {
            let lock_key = format!("LOCK_{}", unit.id);

            // Check if it's locked in Redis
            let is_locked: bool = cache_conn.get(&lock_key).unwrap_or(false);

            if !is_locked {
                // Lock the compute unit in Redis for 1 hour
                let set_result: Result<(), _> = cache_conn.set_ex(&lock_key, true, 3600);
            
                match set_result {
                    Ok(_) => {
                        println!("✅ Successfully added lock in the cache: {:?} , {:?}", lock_key, unit);
                        return Ok(Some(unit));
                    }
                    Err(err) => {
                        println!("❌ Failed to add lock in the cache: {:?}, Error: {:?}", lock_key, err);
                        return Err(diesel::result::Error::RollbackTransaction); // Or handle error appropriately
                    }
                }
            }

            // If locked, add to exclusion list and retry
            excluded_ids.push(unit.id);
        } else {
            return Ok(None); // No available compute units found
        }
    }
}

/// **Release the lock on a compute unit after use**
pub fn release_compute_unit_lock(cache_conn: &mut PooledConnection<RedisConnectionManager> , compute_unit_id: i64) {
    let lock_key = format!("LOCK_{}", compute_unit_id);
    let _: () = cache_conn.del(&lock_key).unwrap_or(());
}