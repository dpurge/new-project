use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub app_name: String,
    pub host: String,
    pub port: u16,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            app_name: get_env("APP_NAME", "{{ cookiecutter.project_name }}"),
            host: get_env("APP_HOST", "0.0.0.0"),
            port: get_env("APP_PORT", "{{ cookiecutter.http_port }}")
                .parse()
                .expect("APP_PORT must be a valid u16"),
            database_url: get_env(
                "DATABASE_URL",
                "postgres://{{ cookiecutter.postgres_user }}:{{ cookiecutter.postgres_password }}@localhost:5432/{{ cookiecutter.postgres_db }}",
            ),
        }
    }
}

fn get_env(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_owned())
}
