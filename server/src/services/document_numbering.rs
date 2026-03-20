use std::hash::{Hash, Hasher};

use crate::error::AppError;

/// 文書番号を採番する。
///
/// フォーマット: `{doc_kind_code}{dept_code}-{YYMM}{seq}`
/// 例: `内設計-2603001`
///
/// `tx` はトランザクション内で呼び出す必要がある。
/// `pg_advisory_xact_lock` でprefix単位の排他ロックを取得し、連番の一意性を保証する。
pub async fn assign_doc_number(
    tx: &mut sqlx::PgConnection,
    doc_kind_code: &str,
    dept_code: &str,
    seq_digits: i32,
    registered_at_jst: chrono::NaiveDateTime,
) -> Result<String, AppError> {
    if !(2..=3).contains(&seq_digits) {
        return Err(AppError::InvalidRequest(format!(
            "seq_digits must be 2 or 3, got {seq_digits}"
        )));
    }

    let yymm = format!(
        "{:02}{:02}",
        registered_at_jst.format("%y"),
        registered_at_jst.format("%m"),
    );
    let prefix = format!("{doc_kind_code}{dept_code}-{yymm}");

    // prefix をハッシュ化してアドバイザリロックのキーとする
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    prefix.hash(&mut hasher);
    let lock_key = hasher.finish() as i64;

    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(lock_key)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;

    let last_doc: Option<String> = sqlx::query_scalar(
        "SELECT doc_number FROM documents
         WHERE doc_number LIKE $1
         ORDER BY doc_number DESC
         LIMIT 1",
    )
    .bind(format!("{prefix}%"))
    .fetch_optional(&mut *tx)
    .await
    .map_err(AppError::Database)?;

    let next_seq = match last_doc {
        Some(ref last) => {
            let seq_str = &last[prefix.len()..];
            let current: i32 = seq_str
                .parse()
                .map_err(|_| AppError::Internal("invalid seq in doc_number".to_string()))?;
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
    use sqlx::Row;

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
                "INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, project_id)
                 VALUES ($1, 'test', $2, $3, '設計', $4)",
            )
            .bind(format!("内設計-2603{i:03}"))
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

        // まず3月の文書を採番して保存
        let march_dt = chrono::NaiveDate::from_ymd_opt(2026, 3, 15)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let mut tx = pool.begin().await.unwrap();
        let march_num =
            assign_doc_number(tx.as_mut(), &kind_code, &dept_code, seq_digits, march_dt)
                .await
                .unwrap();
        // 文書テーブルに保存するため依存データを作成
        let emp_id: uuid::Uuid = sqlx::query(
            "INSERT INTO employees (name, employee_code, role, is_active)
             VALUES ('Test', 'T001', 'admin', true) RETURNING id",
        )
        .fetch_one(tx.as_mut())
        .await
        .unwrap()
        .get("id");
        let dept_id: uuid::Uuid =
            sqlx::query_scalar("SELECT id FROM departments WHERE code = '設計'")
                .fetch_one(tx.as_mut())
                .await
                .unwrap();
        let disc_id: uuid::Uuid = sqlx::query(
            "INSERT INTO disciplines (code, name, department_id) VALUES ('MECH', '機械', $1) RETURNING id",
        )
        .bind(dept_id)
        .fetch_one(tx.as_mut())
        .await
        .unwrap()
        .get("id");
        let proj_id: uuid::Uuid = sqlx::query(
            "INSERT INTO projects (name, discipline_id) VALUES ('テスト', $1) RETURNING id",
        )
        .bind(disc_id)
        .fetch_one(tx.as_mut())
        .await
        .unwrap()
        .get("id");
        let dk_id: uuid::Uuid =
            sqlx::query_scalar("SELECT id FROM document_kinds WHERE code = '内'")
                .fetch_one(tx.as_mut())
                .await
                .unwrap();
        sqlx::query(
            "INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, project_id)
             VALUES ($1, 'test', $2, $3, '設計', $4)",
        )
        .bind(&march_num)
        .bind(emp_id)
        .bind(dk_id)
        .bind(proj_id)
        .execute(tx.as_mut())
        .await
        .unwrap();
        tx.commit().await.unwrap();

        // 4月で採番 → 3月のデータがあっても001から始まる
        let april_dt = chrono::NaiveDate::from_ymd_opt(2026, 4, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let mut tx = pool.begin().await.unwrap();
        let result = assign_doc_number(tx.as_mut(), &kind_code, &dept_code, seq_digits, april_dt)
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
