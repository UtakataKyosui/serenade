extern crate proc_macro;

use structs::GuildCommand;

#[macro_export]
macro_rules! get_env {
    ($env_name:expr) => {
        std::env::var($env_name)
            .expect(&format!("Environment variable {} not found", $env_name))
        
    };
}

#[macro_export]
macro_rules! guild_command {
    ($name:expr, $description:expr) => {
        GuildCommand::new($name.to_string(), Some($description.to_string()))
    };
}
