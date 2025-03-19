use r2d2_redis::{r2d2::Pool, RedisConnectionManager};
use std::{env, ops::Deref, process::exit};

// Define a type alias for your Redis connection pool
type RedisPool = Pool<RedisConnectionManager>;

// Function to create and return a Redis connection pool
pub fn create_redis_pool() -> RedisPool {
    // Create a Redis connection manager

    let redis_url = env::var("REDIS_URI").expect("REDIS_URI must be set");

    let manager = RedisConnectionManager::new(redis_url).unwrap();

    // Create a pool with the manager
    match Pool::builder()
        .max_size(15) // Set the maximum number of connections in the pool
        .build(manager)
    {
        Ok(pool) => pool,
        Err(_) => {
            println!("Failed to connect to redis");
            exit(1);
        }
    }
}

// Rocket State to manage the Redis pool
#[derive(Debug)]
pub struct RedisPoolState(pub RedisPool);

// Implement Deref for RedisPoolState to access the pool easily
impl Deref for RedisPoolState {
    type Target = RedisPool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
