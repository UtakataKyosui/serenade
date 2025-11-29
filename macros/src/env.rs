use std;

#[macro_export]
macro_rules! get_env {
    ($env_name:expr) => {
        std::env::var($env_name)
            .expect(&format!("Environment variable {} not found", $env_name))
    };
}