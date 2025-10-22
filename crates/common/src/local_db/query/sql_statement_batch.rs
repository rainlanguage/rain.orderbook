use super::sql_statement::SqlStatement;
use thiserror::Error;

const BEGIN_TRANSACTION: &str = "BEGIN TRANSACTION";
const COMMIT: &str = "COMMIT";

#[derive(Clone, Debug, Default)]
pub struct SqlStatementBatch {
    statements: Vec<SqlStatement>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SqlBatchError {
    #[error("SQL statement batch is already wrapped in a transaction")]
    AlreadyTransaction,
}

impl SqlStatementBatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_statements(statements: Vec<SqlStatement>) -> Self {
        Self { statements }
    }

    pub fn add(&mut self, statement: SqlStatement) -> &mut Self {
        self.statements.push(statement);
        self
    }

    pub fn extend(&mut self, other: SqlStatementBatch) -> &mut Self {
        self.statements.extend(other.statements);
        self
    }

    pub fn into_transaction(mut self) -> Result<Self, SqlBatchError> {
        if self
            .statements
            .first()
            .is_some_and(|stmt| is_begin(stmt.sql()))
            || self
                .statements
                .last()
                .is_some_and(|stmt| is_commit(stmt.sql()))
        {
            return Err(SqlBatchError::AlreadyTransaction);
        }

        self.statements
            .insert(0, SqlStatement::new(BEGIN_TRANSACTION));
        self.statements.push(SqlStatement::new(COMMIT));
        Ok(self)
    }

    pub fn statements(&self) -> &[SqlStatement] {
        &self.statements
    }

    pub fn len(&self) -> usize {
        self.statements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }

    pub fn into_inner(self) -> Vec<SqlStatement> {
        self.statements
    }
}

impl From<Vec<SqlStatement>> for SqlStatementBatch {
    fn from(statements: Vec<SqlStatement>) -> Self {
        Self { statements }
    }
}

impl IntoIterator for SqlStatementBatch {
    type Item = SqlStatement;
    type IntoIter = std::vec::IntoIter<SqlStatement>;

    fn into_iter(self) -> Self::IntoIter {
        self.statements.into_iter()
    }
}

impl<'a> IntoIterator for &'a SqlStatementBatch {
    type Item = &'a SqlStatement;
    type IntoIter = std::slice::Iter<'a, SqlStatement>;

    fn into_iter(self) -> Self::IntoIter {
        self.statements.iter()
    }
}

impl<'a> IntoIterator for &'a mut SqlStatementBatch {
    type Item = &'a mut SqlStatement;
    type IntoIter = std::slice::IterMut<'a, SqlStatement>;

    fn into_iter(self) -> Self::IntoIter {
        self.statements.iter_mut()
    }
}

fn is_begin(sql: &str) -> bool {
    normalized(sql).eq_ignore_ascii_case(BEGIN_TRANSACTION)
}

fn is_commit(sql: &str) -> bool {
    normalized(sql).eq_ignore_ascii_case(COMMIT)
}

fn normalized(sql: &str) -> &str {
    let trimmed = sql.trim();
    trimmed
        .strip_suffix(';')
        .map(str::trim_end)
        .unwrap_or(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_batch() {
        let batch = SqlStatementBatch::new();
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn add_pushes_statement() {
        let mut batch = SqlStatementBatch::new();
        batch.add(SqlStatement::new("SELECT 1"));
        batch.add(SqlStatement::new("SELECT 2"));

        assert!(!batch.is_empty());
        assert_eq!(batch.len(), 2);
        let texts: Vec<_> = batch
            .statements()
            .iter()
            .map(|s| s.sql().to_owned())
            .collect();
        assert_eq!(texts, vec!["SELECT 1", "SELECT 2"]);
    }

    #[test]
    fn add_is_chainable() {
        let mut batch = SqlStatementBatch::new();
        batch
            .add(SqlStatement::new("SELECT 1"))
            .add(SqlStatement::new("SELECT 2"));

        let texts: Vec<_> = batch.into_iter().map(|s| s.sql().to_owned()).collect();
        assert_eq!(texts, vec!["SELECT 1", "SELECT 2"]);
    }

    #[test]
    fn extend_combines_batches() {
        let mut batch = SqlStatementBatch::from(vec![SqlStatement::new("SELECT 1")]);
        let other = SqlStatementBatch::from(vec![
            SqlStatement::new("SELECT 2"),
            SqlStatement::new("SELECT 3"),
        ]);

        batch.extend(other);

        let texts: Vec<_> = batch.into_iter().map(|s| s.sql().to_owned()).collect();
        assert_eq!(texts, vec!["SELECT 1", "SELECT 2", "SELECT 3"]);
    }

    #[test]
    fn extend_with_empty_batch_is_noop() {
        let mut batch = SqlStatementBatch::from(vec![SqlStatement::new("SELECT 1")]);
        batch.extend(SqlStatementBatch::new());
        assert_eq!(batch.len(), 1);
        assert_eq!(batch.statements()[0].sql(), "SELECT 1");
    }

    #[test]
    fn with_statements_and_from_match() {
        let vec = vec![SqlStatement::new("SELECT 1"), SqlStatement::new("SELECT 2")];
        let batch_from = SqlStatementBatch::from(vec.clone());
        let batch_with = SqlStatementBatch::with_statements(vec);

        assert_eq!(
            batch_from
                .statements()
                .iter()
                .map(|s| s.sql())
                .collect::<Vec<_>>(),
            batch_with
                .statements()
                .iter()
                .map(|s| s.sql())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn statements_slice_reflects_updates() {
        let mut batch = SqlStatementBatch::new();
        batch.add(SqlStatement::new("SELECT 1"));
        let slice = batch.statements();
        assert_eq!(slice.len(), 1);
        assert_eq!(slice[0].sql(), "SELECT 1");
    }

    #[test]
    fn into_inner_returns_vec() {
        let batch = SqlStatementBatch::from(vec![
            SqlStatement::new("SELECT 1"),
            SqlStatement::new("SELECT 2"),
        ]);

        let inner = batch.into_inner();
        let texts: Vec<_> = inner.into_iter().map(|s| s.sql().to_owned()).collect();
        assert_eq!(texts, vec!["SELECT 1", "SELECT 2"]);
    }

    #[test]
    fn iterates_by_reference() {
        let batch = SqlStatementBatch::from(vec![
            SqlStatement::new("SELECT 1"),
            SqlStatement::new("SELECT 2"),
        ]);

        let texts: Vec<_> = (&batch).into_iter().map(|s| s.sql().to_owned()).collect();
        assert_eq!(texts, vec!["SELECT 1", "SELECT 2"]);
    }

    #[test]
    fn iterates_mutably() {
        let mut batch = SqlStatementBatch::from(vec![
            SqlStatement::new("SELECT 1"),
            SqlStatement::new("SELECT 2"),
        ]);

        for stmt in (&mut batch).into_iter() {
            let new_sql = format!("{} -- comment", stmt.sql());
            *stmt = SqlStatement::new(new_sql);
        }

        let texts: Vec<_> = batch.into_iter().map(|s| s.sql().to_owned()).collect();
        assert_eq!(texts, vec!["SELECT 1 -- comment", "SELECT 2 -- comment"]);
    }

    #[test]
    fn into_transaction_wraps_statements() {
        let batch = SqlStatementBatch::from(vec![
            SqlStatement::new("INSERT INTO foo VALUES (?1)"),
            SqlStatement::new("UPDATE foo SET bar = ?1"),
        ]);

        let batch = batch.into_transaction().unwrap();
        let texts: Vec<_> = batch.into_iter().map(|s| s.sql().to_owned()).collect();
        assert_eq!(
            texts,
            vec![
                BEGIN_TRANSACTION,
                "INSERT INTO foo VALUES (?1)",
                "UPDATE foo SET bar = ?1",
                COMMIT
            ]
        );
    }

    #[test]
    fn into_transaction_wraps_empty_batch() {
        let batch = SqlStatementBatch::new().into_transaction().unwrap();
        let texts: Vec<_> = batch.into_iter().map(|s| s.sql().to_owned()).collect();
        assert_eq!(texts, vec![BEGIN_TRANSACTION, COMMIT]);
    }

    #[test]
    fn into_transaction_detects_existing_wrapper_by_begin() {
        let batch = SqlStatementBatch::from(vec![
            SqlStatement::new("BEGIN TRANSACTION"),
            SqlStatement::new("INSERT INTO foo VALUES (?1)"),
        ]);

        let err = batch.into_transaction().unwrap_err();
        assert_eq!(err, SqlBatchError::AlreadyTransaction);
    }

    #[test]
    fn into_transaction_detects_existing_wrapper_by_commit() {
        let batch = SqlStatementBatch::from(vec![
            SqlStatement::new("INSERT INTO foo VALUES (?1)"),
            SqlStatement::new("COMMIT"),
        ]);

        let err = batch.into_transaction().unwrap_err();
        assert_eq!(err, SqlBatchError::AlreadyTransaction);
    }

    #[test]
    fn into_transaction_detects_commit_with_whitespace_and_semicolon() {
        let batch = SqlStatementBatch::from(vec![
            SqlStatement::new("INSERT INTO foo VALUES (?1)"),
            SqlStatement::new("  commit ; "),
        ]);

        let err = batch.into_transaction().unwrap_err();
        assert_eq!(err, SqlBatchError::AlreadyTransaction);
    }

    #[test]
    fn into_transaction_recognizes_semicolon_suffix() {
        let batch = SqlStatementBatch::from(vec![SqlStatement::new("BEGIN TRANSACTION;")]);
        assert_eq!(
            batch.into_transaction().unwrap_err(),
            SqlBatchError::AlreadyTransaction
        );
    }

    #[test]
    fn into_transaction_detects_begin_with_whitespace_and_lowercase() {
        let batch = SqlStatementBatch::from(vec![
            SqlStatement::new(" begin transaction ; "),
            SqlStatement::new("INSERT INTO foo VALUES (?1)"),
        ]);

        let err = batch.into_transaction().unwrap_err();
        assert_eq!(err, SqlBatchError::AlreadyTransaction);
    }

    #[test]
    fn into_iter_consumes_batch() {
        let batch = SqlStatementBatch::from(vec![
            SqlStatement::new("SELECT 1"),
            SqlStatement::new("SELECT 2"),
        ]);

        let collected: Vec<_> = batch.into_iter().map(|s| s.sql().to_owned()).collect();
        assert_eq!(collected, vec!["SELECT 1", "SELECT 2"]);
    }
}
