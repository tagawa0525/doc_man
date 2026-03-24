use std::hash::{Hash, Hasher};

use crate::error::AppError;

/// 採番結果の構成要素。
/// 文書番号は DB 生成列が `frozen_kind_code || frozen_dept_code || '-' || doc_period || lpad(doc_seq, frozen_seq_digits, '0')` で組み立てる。
pub struct DocNumberParts {
    pub frozen_kind_code: String,
    pub doc_period: String,
    pub doc_seq: i32,
    pub frozen_seq_digits: i32,
}

/// 文書番号を採番する。
///
/// 同一 `(frozen_kind_code, frozen_dept_code, doc_period)` 内で次の `doc_seq` を決定し、
/// `DocNumberParts` を返す。
///
/// `tx` はトランザクション内で呼び出す必要がある。
/// `pg_advisory_xact_lock` でprefix単位の排他ロックを取得し、連番の一意性を保証する。
pub async fn assign_doc_number(
    tx: &mut sqlx::PgConnection,
    doc_kind_code: &str,
    dept_code: &str,
    seq_digits: i32,
    registered_at_jst: chrono::NaiveDateTime,
) -> Result<DocNumberParts, AppError> {
    if !(2..=3).contains(&seq_digits) {
        return Err(AppError::InvalidRequest(format!(
            "seq_digits must be 2 or 3, got {seq_digits}"
        )));
    }

    let doc_period = format!(
        "{:02}{:02}",
        registered_at_jst.format("%y"),
        registered_at_jst.format("%m"),
    );
    let prefix = format!("{doc_kind_code}{dept_code}-{doc_period}");

    // prefix をハッシュ化してアドバイザリロックのキーとする
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    prefix.hash(&mut hasher);
    let lock_key = hasher.finish() as i64;

    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(lock_key)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;

    let max_seq: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(doc_seq) FROM documents
         WHERE frozen_kind_code = $1
           AND frozen_dept_code = $2
           AND doc_period = $3",
    )
    .bind(doc_kind_code)
    .bind(dept_code)
    .bind(&doc_period)
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::Database)?;

    let next_seq = max_seq.unwrap_or(0) + 1;

    Ok(DocNumberParts {
        frozen_kind_code: doc_kind_code.to_string(),
        doc_period,
        doc_seq: next_seq,
        frozen_seq_digits: seq_digits,
    })
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

        assert_eq!(result.frozen_kind_code, "内");
        assert_eq!(result.doc_period, "2603");
        assert_eq!(result.doc_seq, 1);
        assert_eq!(result.frozen_seq_digits, 3);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn increments_existing_sequence(pool: PgPool) {
        let (kind_code, dept_code, seq_digits) = setup_test_data(&pool).await;

        // employee + project + doc_kind_id for document insertion
        let emp_id: uuid::Uuid = sqlx::query(
            "INSERT INTO employees (name, employee_code, role, position_id, is_active)
             VALUES ('Test', 'T001', 'admin', (SELECT id FROM positions WHERE name = '課長'), true) RETURNING id",
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

        // 既存文書を2件挿入（複合カラム方式）
        for i in 1..=2 {
            sqlx::query(
                "INSERT INTO documents (
                    frozen_kind_code, frozen_dept_code, doc_period, doc_seq, frozen_seq_digits,
                    title, author_id, doc_kind_id, project_id
                 )
                 VALUES ('内', '設計', '2603', $1, 3, 'test', $2, $3, $4)",
            )
            .bind(i)
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

        assert_eq!(result.doc_seq, 3);
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
        let march_parts =
            assign_doc_number(tx.as_mut(), &kind_code, &dept_code, seq_digits, march_dt)
                .await
                .unwrap();
        // 文書テーブルに保存するため依存データを作成
        let emp_id: uuid::Uuid = sqlx::query(
            "INSERT INTO employees (name, employee_code, role, position_id, is_active)
             VALUES ('Test', 'T001', 'admin', (SELECT id FROM positions WHERE name = '課長'), true) RETURNING id",
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
            "INSERT INTO documents (
                frozen_kind_code, frozen_dept_code, doc_period, doc_seq, frozen_seq_digits,
                title, author_id, doc_kind_id, project_id
             )
             VALUES ($1, $2, $3, $4, $5, 'test', $6, $7, $8)",
        )
        .bind(&march_parts.frozen_kind_code)
        .bind(&dept_code)
        .bind(&march_parts.doc_period)
        .bind(march_parts.doc_seq)
        .bind(march_parts.frozen_seq_digits)
        .bind(emp_id)
        .bind(dk_id)
        .bind(proj_id)
        .execute(tx.as_mut())
        .await
        .unwrap();
        tx.commit().await.unwrap();

        // 4月で採番 → 3月のデータがあっても1から始まる
        let april_dt = chrono::NaiveDate::from_ymd_opt(2026, 4, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let mut tx = pool.begin().await.unwrap();
        let result = assign_doc_number(tx.as_mut(), &kind_code, &dept_code, seq_digits, april_dt)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        assert_eq!(result.doc_period, "2604");
        assert_eq!(result.doc_seq, 1);
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

        assert_eq!(result.frozen_kind_code, "議");
        assert_eq!(result.doc_period, "2603");
        assert_eq!(result.doc_seq, 1);
        assert_eq!(result.frozen_seq_digits, 2);
    }
}
