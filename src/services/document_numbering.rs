use crate::error::AppError;

/// 文書番号を採番する。
///
/// フォーマット: `{doc_kind_code}{dept_code}-{YYMM}{seq}`
/// 例: `内設計-2603001`
///
/// `tx` はトランザクション内で呼び出す必要がある。
/// `SELECT ... FOR UPDATE` で排他ロックを取得し、連番の一意性を保証する。
pub async fn assign_doc_number(
    tx: &mut sqlx::PgConnection,
    doc_kind_code: &str,
    dept_code: &str,
    seq_digits: i32,
    registered_at_jst: chrono::NaiveDateTime,
) -> Result<String, AppError> {
    let yymm = format!(
        "{:02}{:02}",
        registered_at_jst.format("%y"),
        registered_at_jst.format("%m"),
    );
    let prefix = format!("{}{}-{}", doc_kind_code, dept_code, yymm);

    let last_doc: Option<String> = sqlx::query_scalar(
        "SELECT doc_number FROM documents
         WHERE doc_number LIKE $1
         ORDER BY doc_number DESC
         LIMIT 1
         FOR UPDATE",
    )
    .bind(format!("{}%", prefix))
    .fetch_optional(&mut *tx)
    .await
    .map_err(AppError::Database)?;

    let next_seq = match last_doc {
        Some(ref last) => {
            let seq_str = &last[prefix.len()..];
            let current: i32 = seq_str
                .parse()
                .map_err(|_| AppError::Internal(format!("invalid seq in doc_number: {}", last)))?;
            current + 1
        }
        None => 1,
    };

    let doc_number = format!(
        "{}{:0>width$}",
        prefix,
        next_seq,
        width = seq_digits as usize
    );

    Ok(doc_number)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    async fn setup_test_data(pool: &PgPool) -> (String, String, i32) {
        // department
        sqlx::query(
            "INSERT INTO departments (code, name, effective_from) VALUES ('設計', '設計部', '2020-01-01')",
        )
        .execute(pool)
        .await
        .unwrap();

        // doc_kind
        sqlx::query("INSERT INTO document_kinds (code, name, seq_digits) VALUES ('内', '社内', 3)")
            .execute(pool)
            .await
            .unwrap();

        ("内".to_string(), "設計".to_string(), 3)
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn assigns_first_number(pool: PgPool) {
        let (kind_code, dept_code, seq_digits) = setup_test_data(&pool).await;
        let dt = chrono::NaiveDate::from_ymd_opt(2026, 3, 15)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();

        let mut tx = pool.begin().await.unwrap();
        let result = assign_doc_number(tx.as_mut(), &kind_code, &dept_code, seq_digits, dt)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        assert_eq!(result, "内設計-2603001");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn increments_existing_sequence(pool: PgPool) {
        let (kind_code, dept_code, seq_digits) = setup_test_data(&pool).await;

        // employee + project + doc_kind_id for document insertion
        use sqlx::Row;
        let emp_id: uuid::Uuid = sqlx::query(
            "INSERT INTO employees (name, employee_code, role, is_active)
             VALUES ('Test', 'T001', 'admin', true) RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .unwrap()
        .get("id");

        let dept_id: uuid::Uuid =
            sqlx::query_scalar("SELECT id FROM departments WHERE code = '設計'")
                .fetch_one(&pool)
                .await
                .unwrap();

        let disc_id: uuid::Uuid = sqlx::query(
            "INSERT INTO disciplines (code, name, department_id)
             VALUES ('MECH', '機械', $1) RETURNING id",
        )
        .bind(dept_id)
        .fetch_one(&pool)
        .await
        .unwrap()
        .get("id");

        let proj_id: uuid::Uuid = sqlx::query(
            "INSERT INTO projects (name, discipline_id) VALUES ('テスト', $1) RETURNING id",
        )
        .bind(disc_id)
        .fetch_one(&pool)
        .await
        .unwrap()
        .get("id");

        let dk_id: uuid::Uuid =
            sqlx::query_scalar("SELECT id FROM document_kinds WHERE code = '内'")
                .fetch_one(&pool)
                .await
                .unwrap();

        // 既存文書を2件挿入
        for i in 1..=2 {
            sqlx::query(
                "INSERT INTO documents (doc_number, title, file_path, author_id, doc_kind_id, frozen_dept_code, project_id)
                 VALUES ($1, 'test', '/path', $2, $3, '設計', $4)",
            )
            .bind(format!("内設計-2603{:03}", i))
            .bind(emp_id)
            .bind(dk_id)
            .bind(proj_id)
            .execute(&pool)
            .await
            .unwrap();
        }

        let dt = chrono::NaiveDate::from_ymd_opt(2026, 3, 15)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();

        let mut tx = pool.begin().await.unwrap();
        let result = assign_doc_number(tx.as_mut(), &kind_code, &dept_code, seq_digits, dt)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        assert_eq!(result, "内設計-2603003");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn different_month_starts_at_one(pool: PgPool) {
        let (kind_code, dept_code, seq_digits) = setup_test_data(&pool).await;

        // 4月で採番（3月のデータがあっても関係ない）
        let dt = chrono::NaiveDate::from_ymd_opt(2026, 4, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();

        let mut tx = pool.begin().await.unwrap();
        let result = assign_doc_number(tx.as_mut(), &kind_code, &dept_code, seq_digits, dt)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        assert_eq!(result, "内設計-2604001");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn two_digit_seq(pool: PgPool) {
        // 議事録は2桁連番
        sqlx::query(
            "INSERT INTO departments (code, name, effective_from) VALUES ('保全', '保全部', '2020-01-01')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO document_kinds (code, name, seq_digits) VALUES ('議', '議事録', 2)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let dt = chrono::NaiveDate::from_ymd_opt(2026, 3, 15)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();

        let mut tx = pool.begin().await.unwrap();
        let result = assign_doc_number(tx.as_mut(), "議", "保全", 2, dt)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        assert_eq!(result, "議保全-260301");
    }
}
