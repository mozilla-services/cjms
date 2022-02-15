use std::collections::HashMap;
use std::env;

const REQUIRED_ENV_VARS: [&str; 3] = ["HOST", "PORT", "DATABASE_URL"];

#[derive(PartialEq, Eq, Debug)]
pub struct Env {
    pub host: String,
    pub port: String,
    pub database_url: String,
}

pub fn get_env() -> Env {
    let mut env = HashMap::new();
    for key in REQUIRED_ENV_VARS {
        match env::var_os(key) {
            Some(val) => {
                env.insert(
                    key,
                    val.into_string().unwrap()
                );
            },
            None => panic!("Aborting. `{}` is not defined in environment.", key),
        }
    }
    let frozen = Env {
        host: env["HOST"].to_string(),
        port: env["PORT"].to_string(),
        database_url: env["DATABASE_URL"].to_string(),
    };
    frozen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_get_env_missing_envvars() {
        let _ = get_env();
    }

    #[test]
    fn test_get_env_with_envvars_returns_struct() {
        env::set_var("HOST", "111.2.3.5");
        env::set_var("PORT", "2222");
        let actual = get_env();
        let expected = Env {
            host: "111.2.3.5".to_string(),
            port: "2222".to_string(),
        };
        assert_eq!(expected, actual);
    }
}
