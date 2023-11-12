pub enum Target {
    Default,
    Wasm,
}

impl Target {
    pub fn flags(&self) -> Vec<String> {
        match self {
            Target::Default => vec![],
            Target::Wasm => vec!["--target".to_owned(), "wasm32-unknown-unknown".to_owned()],
        }
    }
}
