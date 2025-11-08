use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", content = "v")]
pub enum SqlValue {
    Text(String),
    I64(i64),
    U64(u64),
    Null,
}

impl From<&str> for SqlValue {
    fn from(s: &str) -> Self {
        SqlValue::Text(s.to_owned())
    }
}
impl From<String> for SqlValue {
    fn from(s: String) -> Self {
        SqlValue::Text(s)
    }
}
impl From<i64> for SqlValue {
    fn from(i: i64) -> Self {
        SqlValue::I64(i)
    }
}
impl From<u64> for SqlValue {
    fn from(i: u64) -> Self {
        SqlValue::U64(i)
    }
}

#[derive(Clone, Debug)]
pub struct SqlStatement {
    pub(crate) sql: String,
    pub(crate) params: Vec<SqlValue>,
}

impl SqlStatement {
    pub fn new(sql: impl Into<String>) -> Self {
        Self {
            sql: sql.into(),
            params: vec![],
        }
    }

    pub fn new_with_params<I, T>(sql: impl Into<String>, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<SqlValue>,
    {
        let mut statement = Self::new(sql);
        for value in values {
            statement.push(value);
        }
        statement
    }

    pub fn sql(&self) -> &str {
        &self.sql
    }

    pub fn params(&self) -> &[SqlValue] {
        &self.params
    }

    /// Push a value and return its placeholder string ("?N"). Use to
    /// satisfy fixed placeholders embedded in the template (e.g., ?1).
    pub fn push(&mut self, v: impl Into<SqlValue>) -> String {
        self.params.push(v.into());
        format!("?{}", self.params.len())
    }

    /// Replace all occurrences of `marker` in the SQL template with the
    /// provided raw text. This uses `String::replace`, which substitutes
    /// every occurrence globally.
    ///
    /// Safety:
    /// - `with` must be trusted/static SQL and not derived from user input.
    ///   This helper performs a verbatim splice (no parameterization), so
    ///   injecting untrusted text can lead to SQL injection.
    /// - `with` must not introduce parameter placeholders like `?1`, `?2`, …
    ///   as this would desynchronize placeholder numbering relative to values
    ///   pushed via [`push`].
    ///
    /// Returns an error if the marker is not present.
    pub fn replace(&mut self, marker: &str, with: &str) -> Result<&mut Self, SqlBuildError> {
        if !self.sql.contains(marker) {
            return Err(SqlBuildError::missing_marker(marker));
        }
        self.sql = self.sql.replace(marker, with);
        Ok(self)
    }

    /// Bind a single-parameter clause: inject `clause_body` at `clause_marker`
    /// and substitute the `{param}` token with a new placeholder. If `v` is
    /// `None`, removes the entire clause.
    pub fn bind_param_clause<T: Into<SqlValue>>(
        &mut self,
        clause_marker: &str,
        clause_body: &str,
        v: Option<T>,
    ) -> Result<&mut Self, SqlBuildError> {
        if !self.sql.contains(clause_marker) {
            return Err(SqlBuildError::missing_marker(clause_marker));
        }
        const PARAM_TOKEN: &str = "{param}";
        if !clause_body.contains(PARAM_TOKEN) {
            return Err(SqlBuildError::missing_marker(PARAM_TOKEN));
        }
        if let Some(v) = v {
            let ph = self.push(v);
            let body = clause_body.replace(PARAM_TOKEN, &ph);
            self.sql = self.sql.replace(clause_marker, &body);
        } else {
            self.sql = self.sql.replace(clause_marker, "");
        }
        Ok(self)
    }

    /// Bind a list clause: inject `clause_body` at `clause_marker` and
    /// substitute the `{list}` token with joined placeholders. If list is empty,
    /// remove the clause entirely.
    pub fn bind_list_clause<T: Into<SqlValue>>(
        &mut self,
        clause_marker: &str,
        clause_body: &str,
        it: impl IntoIterator<Item = T>,
    ) -> Result<&mut Self, SqlBuildError> {
        if !self.sql.contains(clause_marker) {
            return Err(SqlBuildError::missing_marker(clause_marker));
        }
        let mut list = String::new();
        let mut first = true;
        let mut count = 0usize;
        for v in it {
            count += 1;
            let ph = self.push(v);
            if !first {
                list.push_str(", ");
            } else {
                first = false;
            }
            list.push_str(&ph);
        }
        if count == 0 {
            self.sql = self.sql.replace(clause_marker, "");
            return Ok(self);
        }
        const LIST_TOKEN: &str = "{list}";
        if !clause_body.contains(LIST_TOKEN) {
            return Err(SqlBuildError::missing_marker(LIST_TOKEN));
        }
        let body = clause_body.replace(LIST_TOKEN, &list);
        self.sql = self.sql.replace(clause_marker, &body);
        Ok(self)
    }

    /// JS parameter conversion for the WASM SDK call.
    pub fn as_js_params(&self) -> Vec<SqlValue> {
        self.params.clone()
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SqlBuildError {
    #[error("SQL template marker not found: {marker}")]
    MissingMarker { marker: String },
    #[error("{message}")]
    Generic { message: String },
}

impl SqlBuildError {
    pub fn missing_marker(marker: impl Into<String>) -> Self {
        SqlBuildError::MissingMarker {
            marker: marker.into(),
        }
    }

    pub fn new(message: impl Into<String>) -> Self {
        SqlBuildError::Generic {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sql_value_from_conversions() {
        assert_eq!(SqlValue::from("abc"), SqlValue::Text("abc".to_owned()));
        assert_eq!(
            SqlValue::from(String::from("def")),
            SqlValue::Text("def".to_owned())
        );
        assert_eq!(SqlValue::from(42i64), SqlValue::I64(42));
        assert_eq!(SqlValue::from(7u64), SqlValue::U64(7));
        assert_eq!(SqlValue::from(7u64), SqlValue::U64(7));
    }

    #[test]
    fn sql_value_serde_shape_and_roundtrip() {
        let v_text = SqlValue::Text("hi".to_owned());
        let v_i64 = SqlValue::I64(5);
        let v_u64 = SqlValue::U64(123);
        let v_null = SqlValue::Null;

        let j_text = serde_json::to_value(&v_text).unwrap();
        let j_i64 = serde_json::to_value(&v_i64).unwrap();
        let j_u64 = serde_json::to_value(&v_u64).unwrap();
        let j_null = serde_json::to_value(&v_null).unwrap();

        assert_eq!(j_text, json!({"t":"Text","v":"hi"}));
        assert_eq!(j_i64, json!({"t":"I64","v":5}));
        assert_eq!(j_u64, json!({"t":"U64","v":123}));
        assert_eq!(j_null, json!({"t":"Null"}));

        let rt_text: SqlValue = serde_json::from_value(j_text).unwrap();
        let rt_i64: SqlValue = serde_json::from_value(j_i64).unwrap();
        let rt_u64: SqlValue = serde_json::from_value(j_u64).unwrap();
        let rt_null: SqlValue = serde_json::from_value(j_null).unwrap();
        assert_eq!(rt_text, v_text);
        assert_eq!(rt_i64, v_i64);
        assert_eq!(rt_u64, v_u64);
        assert_eq!(rt_null, v_null);
    }

    #[test]
    fn sql_value_vec_serialization() {
        let arr = vec![
            SqlValue::Text("a".to_owned()),
            SqlValue::I64(1),
            SqlValue::U64(2),
            SqlValue::Null,
        ];
        let j = serde_json::to_value(&arr).unwrap();
        assert_eq!(
            j,
            json!([
                {"t":"Text","v":"a"},
                {"t":"I64","v":1},
                {"t":"U64","v":2},
                {"t":"Null"}
            ])
        );
    }

    #[test]
    fn statement_new_and_push() {
        let mut s = SqlStatement::new("SELECT 1");
        assert_eq!(s.sql, "SELECT 1");
        assert!(s.params.is_empty());

        let p1 = s.push(10i64);
        assert_eq!(p1, "?1");
        let p2 = s.push("abc");
        assert_eq!(p2, "?2");
        assert_eq!(
            s.params,
            vec![SqlValue::I64(10), SqlValue::Text("abc".to_owned())]
        );
    }

    #[test]
    fn statement_new_with_params_populates_params() {
        let s = SqlStatement::new_with_params(
            "SELECT ?1, ?2, ?3",
            vec![
                SqlValue::I64(10),
                SqlValue::Text("twenty".to_owned()),
                SqlValue::Null,
            ],
        );
        assert_eq!(s.sql, "SELECT ?1, ?2, ?3");
        assert_eq!(
            s.params,
            vec![
                SqlValue::I64(10),
                SqlValue::Text("twenty".to_owned()),
                SqlValue::Null,
            ]
        );
    }

    #[test]
    fn statement_new_with_params_preserves_placeholder_sequence() {
        let mut s =
            SqlStatement::new_with_params("SELECT * FROM t WHERE a = ?1 AND b = ?2", [1i64, 2i64]);
        let ph = s.push("third");
        assert_eq!(ph, "?3");
        assert_eq!(
            s.params,
            vec![
                SqlValue::I64(1),
                SqlValue::I64(2),
                SqlValue::Text("third".to_owned())
            ]
        );
    }

    #[test]
    fn push_accepts_u64_values() {
        let mut s = SqlStatement::new("SELECT ?1");
        let ph = s.push(123u64);
        assert_eq!(ph, "?1");
        assert_eq!(s.params, vec![SqlValue::U64(123)]);
    }

    #[test]
    fn replace_success_replaces_all_and_keeps_params() {
        let mut s = SqlStatement::new("A /*X*/ B /*X*/");
        s.push(1i64);
        s.push("two");
        let before = s.params.clone();
        s.replace("/*X*/", "Y").unwrap();
        assert_eq!(s.sql, "A Y B Y");
        assert_eq!(s.params, before);
    }

    #[test]
    fn replace_missing_marker_error() {
        let mut s = SqlStatement::new("SELECT 1");
        let err = s.replace("/*NOPE*/", "X").unwrap_err();
        assert_eq!(
            err,
            SqlBuildError::MissingMarker {
                marker: "/*NOPE*/".to_owned()
            }
        );
        assert_eq!(err.to_string(), "SQL template marker not found: /*NOPE*/");
    }

    #[test]
    fn bind_param_clause_some_injects_and_pushes() {
        let mut s = SqlStatement::new("SELECT * FROM t WHERE 1=1 /*M*/");
        s.bind_param_clause("/*M*/", " AND a = {param}", Some(99i64))
            .unwrap();
        assert_eq!(s.sql, "SELECT * FROM t WHERE 1=1  AND a = ?1");
        assert_eq!(s.params, vec![SqlValue::I64(99)]);
    }

    #[test]
    fn bind_param_clause_accepts_u64_value() {
        let mut s = SqlStatement::new("SELECT * FROM t WHERE 1=1 /*M*/");
        s.bind_param_clause("/*M*/", " AND a = {param}", Some(42u64))
            .unwrap();
        assert_eq!(s.sql, "SELECT * FROM t WHERE 1=1  AND a = ?1");
        assert_eq!(s.params, vec![SqlValue::U64(42)]);
    }

    #[test]
    fn bind_param_clause_none_removes_marker_no_push() {
        let mut s = SqlStatement::new("SELECT * FROM t WHERE 1=1 /*M*/");
        s.push(7i64);
        s.bind_param_clause("/*M*/", " AND a = {param}", Option::<i64>::None)
            .unwrap();
        assert_eq!(s.sql, "SELECT * FROM t WHERE 1=1 ");
        assert_eq!(s.params, vec![SqlValue::I64(7)]);
    }

    #[test]
    fn bind_param_clause_missing_marker_errors() {
        let mut s = SqlStatement::new("SELECT 1");
        let err = s
            .bind_param_clause("/*M*/", " AND a = {param}", Some(1i64))
            .unwrap_err();
        assert_eq!(
            err,
            SqlBuildError::MissingMarker {
                marker: "/*M*/".to_owned()
            }
        );
    }

    #[test]
    fn bind_param_clause_missing_param_token_errors_even_if_none() {
        let mut s = SqlStatement::new("SELECT * FROM t WHERE 1=1 /*M*/");
        // Missing {param} in the clause body should error regardless of Some/None
        let err = s
            .bind_param_clause("/*M*/", " AND a = ?1", Option::<i64>::None)
            .unwrap_err();
        assert_eq!(
            err,
            SqlBuildError::MissingMarker {
                marker: "{param}".to_owned()
            }
        );
    }

    #[test]
    fn bind_param_clause_numbering_continuity() {
        let mut s = SqlStatement::new("SELECT * FROM t WHERE chain_id = ?1 /*M*/");
        // Fixed param for ?1
        let p = s.push(100i64);
        assert_eq!(p, "?1");
        // Now dynamic param should start at ?2
        s.bind_param_clause("/*M*/", " AND owner = {param}", Some("alice"))
            .unwrap();
        assert_eq!(s.sql, "SELECT * FROM t WHERE chain_id = ?1  AND owner = ?2");
        assert_eq!(
            s.params,
            vec![SqlValue::I64(100), SqlValue::Text("alice".to_owned())]
        );
    }

    #[test]
    fn bind_list_clause_non_empty_joins_and_pushes() {
        let mut s = SqlStatement::new("SELECT * FROM t WHERE 1=1 /*LIST*/");
        s.bind_list_clause("/*LIST*/", " AND a IN ({list})", vec![1i64, 2, 3])
            .unwrap();
        assert_eq!(s.sql, "SELECT * FROM t WHERE 1=1  AND a IN (?1, ?2, ?3)");
        assert_eq!(
            s.params,
            vec![SqlValue::I64(1), SqlValue::I64(2), SqlValue::I64(3)]
        );
    }

    #[test]
    fn bind_list_clause_accepts_u64_values() {
        let mut s = SqlStatement::new("SELECT * FROM t WHERE 1=1 /*LIST*/");
        s.bind_list_clause("/*LIST*/", " AND a IN ({list})", vec![10u64, 11, 12])
            .unwrap();
        assert_eq!(s.sql, "SELECT * FROM t WHERE 1=1  AND a IN (?1, ?2, ?3)");
        assert_eq!(
            s.params,
            vec![SqlValue::U64(10), SqlValue::U64(11), SqlValue::U64(12)]
        );
    }

    #[test]
    fn bind_list_clause_empty_removes_marker() {
        let mut s = SqlStatement::new("SELECT * FROM t WHERE 1=1 /*LIST*/");
        s.push("x");
        s.bind_list_clause::<i64>("/*LIST*/", " AND a IN ({list})", std::iter::empty())
            .unwrap();
        assert_eq!(s.sql, "SELECT * FROM t WHERE 1=1 ");
        assert_eq!(s.params, vec![SqlValue::Text("x".to_owned())]);
    }

    #[test]
    fn bind_list_clause_missing_marker_errors() {
        let mut s = SqlStatement::new("SELECT 1");
        let err = s
            .bind_list_clause("/*LIST*/", " AND a IN ({list})", vec![1i64])
            .unwrap_err();
        assert_eq!(
            err,
            SqlBuildError::MissingMarker {
                marker: "/*LIST*/".to_owned()
            }
        );
    }

    #[test]
    fn bind_list_clause_missing_list_token_errors_when_non_empty() {
        let mut s = SqlStatement::new("SELECT * FROM t WHERE 1=1 /*LIST*/");
        let err = s
            .bind_list_clause("/*LIST*/", " AND a IN (?)", vec![1i64, 2])
            .unwrap_err();
        assert_eq!(
            err,
            SqlBuildError::MissingMarker {
                marker: "{list}".to_owned()
            }
        );
    }

    #[test]
    fn bind_list_clause_missing_list_token_ok_when_empty() {
        // For empty lists, the clause is removed before validating the token
        let mut s = SqlStatement::new("SELECT * FROM t WHERE 1=1 /*LIST*/");
        s.bind_list_clause::<i64>("/*LIST*/", " AND a IN (?)", std::iter::empty())
            .unwrap();
        assert_eq!(s.sql, "SELECT * FROM t WHERE 1=1 ");
        assert!(s.params.is_empty());
    }

    #[test]
    fn bind_list_clause_numbering_continuity() {
        let mut s = SqlStatement::new("SELECT * FROM t WHERE owner = ?1 /*LIST*/");
        let p = s.push("alice");
        assert_eq!(p, "?1");
        s.bind_list_clause("/*LIST*/", " AND id IN ({list})", vec![10i64, 11])
            .unwrap();
        assert_eq!(
            s.sql,
            "SELECT * FROM t WHERE owner = ?1  AND id IN (?2, ?3)"
        );
        assert_eq!(
            s.params,
            vec![
                SqlValue::Text("alice".to_owned()),
                SqlValue::I64(10),
                SqlValue::I64(11)
            ]
        );
    }

    #[test]
    fn combined_build_with_replace_param_and_list() {
        let mut s = SqlStatement::new("SELECT * FROM t WHERE chain_id = ?1 /*W*/ /*L*/ /*TAIL*/");
        // Fixed param for ?1
        s.push(1i64);
        // Optional filter
        s.bind_param_clause("/*W*/", " AND owner = {param}", Some("bob"))
            .unwrap();
        // List filter
        s.bind_list_clause("/*L*/", " AND id IN ({list})", vec![5i64, 6])
            .unwrap();
        // Simple splice
        s.replace("/*TAIL*/", " ORDER BY id DESC LIMIT 10").unwrap();

        assert_eq!(
            s.sql,
            "SELECT * FROM t WHERE chain_id = ?1  AND owner = ?2  AND id IN (?3, ?4)  ORDER BY id DESC LIMIT 10"
        );
        assert_eq!(
            s.params,
            vec![
                SqlValue::I64(1),
                SqlValue::Text("bob".to_owned()),
                SqlValue::I64(5),
                SqlValue::I64(6),
            ]
        );
    }

    #[test]
    fn as_js_params_returns_clone() {
        let mut s = SqlStatement::new("SELECT 1 WHERE a=?1");
        s.push(7i64);
        let js_params = s.as_js_params();
        assert_eq!(js_params, s.params);
    }

    #[test]
    fn error_type_display_message() {
        let e = SqlBuildError::missing_marker("MARK");
        assert_eq!(e.to_string(), "SQL template marker not found: MARK");
    }

    // Property tests (native-only)
    #[cfg(not(target_family = "wasm"))]
    mod prop {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn bind_list_clause_length_and_params(vals in proptest::collection::vec(0i64..1000, 0..20)) {
                let mut s = SqlStatement::new("SELECT 1 /*M*/");
                s.bind_list_clause("/*M*/", " WHERE id IN ({list})", vals.clone()).unwrap();

                // Params length matches input length and values preserved in order
                prop_assert_eq!(s.params.len(), vals.len());
                let expected: Vec<SqlValue> = vals.into_iter().map(SqlValue::I64).collect();
                let expected_len = expected.len();
                prop_assert_eq!(s.params, expected);

                // Placeholder count equals number of params
                let ph_count = s.sql.matches('?').count();
                prop_assert_eq!(ph_count, expected_len);
            }

            #[test]
            fn bind_list_clause_length_and_params_u64(vals in proptest::collection::vec(0u64..1_000, 0..20)) {
                let mut s = SqlStatement::new("SELECT 1 /*M*/");
                s.bind_list_clause("/*M*/", " WHERE id IN ({list})", vals.clone()).unwrap();

                prop_assert_eq!(s.params.len(), vals.len());
                let expected: Vec<SqlValue> = vals.into_iter().map(SqlValue::U64).collect();
                let expected_len = expected.len();
                prop_assert_eq!(s.params, expected);

                let ph_count = s.sql.matches('?').count();
                prop_assert_eq!(ph_count, expected_len);
            }
        }
    }
}
