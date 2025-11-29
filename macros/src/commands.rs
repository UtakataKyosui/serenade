use structs::GuildCommand;


#[macro_export]
macro_rules! guild_command {
    ($name:expr, $description:expr) => {
        GuildCommand::new($name.to_string(), Some($description.to_string()))
    };
}
