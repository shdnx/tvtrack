pub trait TableModel: Sized {
    fn table_name() -> &'static str;
    fn from_full_row(row: &rusqlite::Row) -> anyhow::Result<Self>;
}
