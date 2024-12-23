use sqlx::postgres::PgDatabaseError;
pub use sqlx::postgres::PgSeverity;
use sqlx::Executor;
use sqlx::PgPool;
use text_size::TextRange;
use text_size::TextSize;

pub struct TypecheckerParams<'a> {
    pub conn: &'a PgPool,
    pub sql: &'a str,
    pub enriched_ast: Option<&'a pg_syntax::AST>,
    pub ast: &'a pg_query_ext::NodeEnum,
}

#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub code: String,
    pub severity: PgSeverity,
    pub position: Option<usize>,
    pub range: Option<TextRange>,
    pub table: Option<String>,
    pub column: Option<String>,
    pub data_type: Option<String>,
    pub constraint: Option<String>,
}

pub async fn check_sql<'a>(params: TypecheckerParams<'a>) -> Vec<TypeError> {
    let mut errs = vec![];

    // prpeared statements work only for select, insert, update, delete, and cte
    if match params.ast {
        pg_query_ext::NodeEnum::SelectStmt(_) => false,
        pg_query_ext::NodeEnum::InsertStmt(_) => false,
        pg_query_ext::NodeEnum::UpdateStmt(_) => false,
        pg_query_ext::NodeEnum::DeleteStmt(_) => false,
        pg_query_ext::NodeEnum::CommonTableExpr(_) => false,
        _ => true,
    } {
        return errs;
    }

    let res = params.conn.prepare(params.sql).await;

    if res.is_err() {
        if let sqlx::Error::Database(err) = res.as_ref().unwrap_err() {
            let pg_err = err.downcast_ref::<PgDatabaseError>();

            let position = match pg_err.position() {
                Some(sqlx::postgres::PgErrorPosition::Original(pos)) => Some(pos - 1),
                _ => None,
            };

            let range = match params.enriched_ast {
                Some(ast) => {
                    if position.is_none() {
                        None
                    } else {
                        ast.covering_node(TextRange::empty(
                            TextSize::try_from(position.unwrap()).unwrap(),
                        ))
                        .map(|node| node.range())
                    }
                }
                None => None,
            };

            errs.push(TypeError {
                message: pg_err.message().to_string(),
                code: pg_err.code().to_string(),
                severity: pg_err.severity(),
                position,
                range,
                table: pg_err.table().map(|s| s.to_string()),
                column: pg_err.column().map(|s| s.to_string()),
                data_type: pg_err.data_type().map(|s| s.to_string()),
                constraint: pg_err.constraint().map(|s| s.to_string()),
            });
        }
    }

    errs
}

#[cfg(test)]
mod tests {
    use async_std::task::block_on;
    use pg_test_utils::test_database::get_new_test_db;

    use crate::{check_sql, TypecheckerParams};

    #[test]
    fn test_basic_type() {
        let input = "select id, unknown from contact;";

        let test_db = block_on(get_new_test_db());

        let root = pg_query_ext::parse(input).unwrap();
        let ast = pg_syntax::parse_syntax(input, &root).ast;

        let errs = block_on(check_sql(TypecheckerParams {
            conn: &test_db,
            sql: input,
            ast: &root,
            enriched_ast: Some(&ast),
        }));

        assert_eq!(errs.len(), 1);

        let e = &errs[0];

        assert_eq!(&input[e.range.unwrap()], "contact");
    }
}
