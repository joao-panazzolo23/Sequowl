use crate::core::models::{
    connection_parameters::ConnectionParameters, database_connection::DatabaseConnection,
    database_type::DatabaseType,
};

pub trait DatabaseProvider {
    fn id(&self) -> &'static DatabaseType;

    fn connect(&self, config: &ConnectionParameters)
    -> anyhow::Result<Box<dyn DatabaseConnection>>;
}
