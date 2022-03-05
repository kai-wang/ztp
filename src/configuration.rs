use serde::Deserialize;

#[derive(Debug, serde::Deserialize)]
pub struct Settings {
  pub database: DatabaseSettings,
  pub application_port: u16
}

#[derive(Debug, serde::Deserialize)]
pub struct DatabaseSettings {
  pub username: String,
  pub password: String,
  pub port: u16,
  pub host: String,
  pub database_name: String
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> { 
  // Initialise our configuration reader
  let mut settings = config::Config::default();
  // Add configuration values from a file named `configuration`.
  
  settings.merge(config::File::with_name("configuration"))?;
  // Try to convert the configuration values it read into
  // our Settings type, the below code works in config 0.11 but 0.12
  settings.try_into()
}

impl DatabaseSettings {
  pub fn connection_string(&self) -> String {
    format!(
      "postgres://{}:{}@{}:{}/{}",
      self.username, self.password, self.host, self.port, self.database_name
    )
  }
}