use super::SqlStatement;
use serde::{Deserialize, Serialize};

pub const INTEGRITY_CHECK_SQL: &str = "PRAGMA quick_check";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrityCheckRow {
    pub quick_check: String,
}

pub fn integrity_check_stmt() -> SqlStatement {
    SqlStatement::new(INTEGRITY_CHECK_SQL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integrity_check_stmt_is_static_and_param_free() {
        let stmt = integrity_check_stmt();
        assert_eq!(stmt.sql(), INTEGRITY_CHECK_SQL);
        assert!(stmt.params().is_empty());
    }

    #[test]
    fn integrity_check_row_serde_ok() {
        let row = IntegrityCheckRow {
            quick_check: "ok".to_string(),
        };
        let j = serde_json::to_value(&row).expect("serialize");
        let rt: IntegrityCheckRow = serde_json::from_value(j).expect("deserialize");
        assert_eq!(rt, row);
    }

    #[test]
    fn integrity_check_row_serde_error() {
        let row = IntegrityCheckRow {
            quick_check: "database disk image is malformed".to_string(),
        };
        let j = serde_json::to_value(&row).expect("serialize");
        let rt: IntegrityCheckRow = serde_json::from_value(j).expect("deserialize");
        assert_eq!(rt, row);
    }
}
