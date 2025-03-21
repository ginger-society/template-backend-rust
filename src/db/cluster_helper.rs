use std::sync::Arc;

use diesel::{r2d2::PooledConnection, PgConnection};
use diesel::prelude::*;
use openssh::{KnownHosts, Session, Stdio};
use r2d2_redis::redis::Commands;
use r2d2_redis::RedisConnectionManager;
use tokio::sync::Mutex;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    time::{timeout, Duration},
};

use crate::models::schema::Compute_Unit;

use super::rabbitmq::DbPool;

pub async fn create_execute_ssh_script(
    ssh_host: &str,
    ssh_user: &str,
    script_path: &str,
    cluster_name: &str,
    cpus: i64,
    memory: &str,
    disk_size: &str,
) -> Result<String, String> {
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

    // ✅ Convert to async reader and read lines properly
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let kubeconfig_lines = Arc::new(Mutex::new(Vec::new()));
    let kubeconfig_lines_stdout = Arc::clone(&kubeconfig_lines);

    let stdout_task = tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            println!("📜 [STDOUT] {}", line);
            if line.contains("[KUBECONFIG]") {
                let mut kubeconfig = kubeconfig_lines_stdout.lock().await;
                kubeconfig.push(line.replace("[KUBECONFIG] ", ""));
            }
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
    }?;

    let kubeconfig_lines = kubeconfig_lines.lock().await;
    if !kubeconfig_lines.is_empty() {
        let kubeconfig = kubeconfig_lines.join("\n");
        Ok(kubeconfig)
    } else {
        Err("⚠️  No kubeconfig data captured.".to_string())
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
            if is_locked {println!("CU is locked , ID: {:?}" , unit.id)}
            if !is_locked {
                // Lock the compute unit in Redis for 1 hour
                let set_result: Result<(), _> = cache_conn.set_ex(&lock_key, true, 240); // its a 4 minute timeout
            
                match set_result {
                    Ok(_) => {
                        println!("✅ Successfully added lock in the cache: {:?} ", lock_key);
                        return Ok(Some(unit));
                    }
                    Err(err) => {
                        println!("❌ Failed to add lock in the cache: {:?}, Error: {:?}", lock_key, err);
                        return Err(diesel::result::Error::RollbackTransaction);
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

pub async fn update_cluster_state(
    db_pool: &DbPool,
    cluster_id: &str,
    new_state: &str,
) -> Result<(), diesel::result::Error> {
    let mut conn = db_pool.get().expect("Failed to get DB connection");
    use crate::models::schema::schema::cluster::dsl::*;

    diesel::update(cluster.filter(identifier.eq(cluster_id)))
        .set(state.eq(new_state))
        .execute(&mut conn)?;

    println!("✅ Cluster '{}' state updated to '{}'", cluster_id, new_state);
    Ok(())
}

pub async fn update_cluster_kubeconfig(
    db_pool: &DbPool,
    cluster_id: &str,
    config: String,
) -> Result<(), diesel::result::Error> {
    let mut conn = db_pool.get().expect("Failed to get DB connection");
    use crate::models::schema::schema::cluster::dsl::*;

    diesel::update(cluster.filter(identifier.eq(cluster_id)))
        .set(kubeconfig.eq(config))
        .execute(&mut conn)?;

    println!("✅ Cluster '{}' kubeconfig updated", cluster_id);
    Ok(())
}




pub async fn delete_cluster(
    db_pool: &DbPool,
    cluster_uid: &str,
) -> Result<Option<Compute_Unit>, String> {
    use crate::models::schema::schema::cluster::dsl as cluster_dsl;
    use crate::models::schema::schema::compute_unit::dsl::*;

    let mut conn = db_pool.get().expect("Failed to get DB connection");

    // Fetch the cluster details
    let cluster_details: Option<(String, Option<String>)> = cluster_dsl::cluster
        .filter(cluster_dsl::identifier.eq(cluster_uid))
        .select((cluster_dsl::identifier, cluster_dsl::parent_server_fqdn))
        .first::<(String, Option<String>)>(&mut conn)
        .optional()
        .map_err(|err| format!("❌ Database error: {:?}", err))?;

    if let Some((cluster_id, parent_server_fqdn)) = cluster_details {
        let parent_server_fqdn = "dc0102.rackmint.com";
        
        // Fetch Compute Unit details
        let compute_unit_details: Compute_Unit = compute_unit
            .filter(fqdn.eq(parent_server_fqdn.clone()))
            .first::<Compute_Unit>(&mut conn)
            .map_err(|err| format!("❌ Failed to fetch compute unit: {:?}", err))?;
        
        let ssh_user = "dc0102";
        let delete_script_path = "/home/dc0102/Documents/rackmint-infra-as-code/delete-cluster.sh";

        // Establish SSH connection
        let session = Session::connect(format!("{}@{}", ssh_user, parent_server_fqdn), KnownHosts::Accept)
            .await
            .map_err(|err| format!("❌ SSH connection failed: {:?}", err))?;

        println!("✅ Connected to {} for cluster deletion", parent_server_fqdn);

        // Execute deletion script
        let command = format!("bash {} '{}'", delete_script_path, cluster_uid);
        let mut child = session
            .command("sh")
            .arg("-c")
            .arg(&command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .await
            .map_err(|err| format!("❌ Failed to execute delete script: {:?}", err))?;

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

        // Wait for the process to complete
        let exit_status = child.wait().await.map_err(|err| format!("❌ SSH command error: {:?}", err))?;

        stdout_task.await.ok();
        stderr_task.await.ok();

        if exit_status.success() {
            return Ok(Some(compute_unit_details));
        } else {
            return Err(format!("❌ Cluster deletion script failed with exit status: {:?}", exit_status));
        }
    }

    println!("⚠️ Cluster '{}' not found.", cluster_uid);
    Ok(None)
}
