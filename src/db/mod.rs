pub mod entry;
pub mod topic;

pub(crate) trait Table {
    fn schema() -> &'static str;
    fn persist_changes(&self, conn: &mut rusqlite::Connection) -> rusqlite::Result<()>;
    fn delete(&self, conn: &rusqlite::Connection) -> rusqlite::Result<()>;
}

pub(crate) struct Database {
    schemas: Vec<&'static str>,
    pub connection: rusqlite::Connection,
}

impl Database {
    pub(crate) fn new(conn: rusqlite::Connection) -> Self {
        Self {
            schemas: Vec::new(),
            connection: conn,
        }
    }

    pub(crate) fn register_model<T: Table>(&mut self) {
        self.schemas.push(T::schema());
    }

    pub(crate) fn create_schemas(&self) -> rusqlite::Result<()> {
        self.connection.execute("PRAGMA foreign_keys = ON", ())?;
        self.schemas
            .iter()
            .map(|s| self.connection.execute(s, ()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }

    pub(crate) fn save<T: Table>(&mut self, t: &mut T) -> rusqlite::Result<()> {
        t.persist_changes(&mut self.connection)
    }
}
