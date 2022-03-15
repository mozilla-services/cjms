pub mod appconfig;
pub mod bigquery;
pub mod cjms;
pub mod controllers;
pub mod models;
pub mod settings;

#[cfg(test)]
pub mod utils {
    use fake::{Fake, StringFaker};

    pub fn random_ascii_string() -> String {
        const ASCII: &str =
			"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"#$%&\'()*+,-./:;<=>?@";
        let f = StringFaker::with(Vec::from(ASCII), 8..90);
        f.fake()
    }

    pub fn random_simple_ascii_string() -> String {
        const ASCII: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ._-";
        let f = StringFaker::with(Vec::from(ASCII), 8..90);
        f.fake()
    }
}
