use super::table_model::TableModel;

#[derive(Debug)]
pub struct User {
    pub id: i64, // TODO: UserId
    pub name: String,
    pub email: String,
}

impl TableModel for User {
    fn table_name() -> &'static str {
        "users"
    }

    fn from_full_row(row: &rusqlite::Row) -> anyhow::Result<Self> {
        let result = Self {
            id: row.get("id")?,
            name: row.get("name")?,
            email: row.get("email")?,
        };
        Ok(result)
    }
}
