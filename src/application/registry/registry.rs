use std::collections::HashMap;

use crate::core::providers::database_provider::DatabaseProvider;

pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn DatabaseProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn DatabaseProvider>) {
        self.providers.insert(provider.id().to_owned(), provider);
    }

    pub fn get(&self, id: &str) -> Option<&dyn DatabaseProvider> {
        self.providers.get(id).map(Box::as_ref)
    }
}
