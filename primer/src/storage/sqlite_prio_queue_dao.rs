use crate::queue::dao::I64PrioQueueStorage;
use rusqlite::{Connection, Error, OptionalExtension};
use uuid::Uuid;

pub struct SqlitePrioQueueDao {
    connection: Connection,
    table: String,
}

impl SqlitePrioQueueDao {
    pub fn new(path: &'static str) -> Self {
        let connection = Connection::open(path).expect(
            format!("should be able to open connection to sqlite database at '{path}'").as_str(),
        );
        let table = format!(
            "primer_prio_queue_{}",
            Uuid::new_v4().to_string().replace("-", "")
        );

        connection
            .execute(
                format!(
                    // language=sqlite
                    r#"
                        CREATE TEMPORARY TABLE {table} (
                            id INTEGER PRIMARY KEY AUTOINCREMENT,
                            priority INTEGER NOT NULL,
                            value INTEGER NOT NULL
                        );

                        CREATE INDEX {table}_priority_index ON {table} (priority);
                    "#,
                    table = table
                )
                .as_str(),
                (),
            )
            .expect(format!("should be able to create table '{table}'", table = table).as_str());

        Self { connection, table }
    }
}

impl I64PrioQueueStorage for SqlitePrioQueueDao {
    fn insert(&mut self, items: &Vec<(i64, i64)>) -> Result<(), Error> {
        let tx = self.connection.transaction()?;
        let mut stmt = tx.prepare(
            format!(
                // language=sqlite
                "INSERT INTO {table} (priority, value) VALUES (?, ?)",
                table = self.table
            )
            .as_str(),
        )?;

        for &item in items {
            stmt.execute(item)?;
        }
        drop(stmt);

        tx.commit()
    }

    fn retrieve(&mut self, count: usize) -> Result<Vec<(i64, i64)>, Error> {
        let mut data = self
            .connection
            .prepare(
                format!(
                    // language=sqlite
                    r#"
                        DELETE
                            FROM
                                {table}
                            WHERE
                                id IN (
                                      SELECT
                                          id
                                          FROM
                                              {table}
                                          ORDER BY
                                              priority
                                          LIMIT {count}
                                      )
                            RETURNING priority, value
                    "#,
                    table = self.table,
                    count = count,
                )
                .as_str(),
            )?
            .query_map([], |row| {
                Ok((row.get::<usize, i64>(0)?, row.get::<usize, i64>(1)?))
            })?
            .collect::<Result<Vec<(i64, i64)>, Error>>()?;

        data.sort_by_key(|(priority, _)| *priority);

        Ok(data)
    }

    fn lowest_priority(&self) -> Option<i64> {
        self.connection
            .query_row(
                format!(
                    // language=sqlite
                    r#"
                        SELECT
                            priority
                            FROM
                                {table}
                            ORDER BY
                                priority
                            LIMIT 1
                    "#,
                    table = self.table
                )
                .as_str(),
                (),
                |row| -> Result<i64, Error> { row.get(0) },
            )
            .optional()
            .unwrap()
    }

    fn len(&self) -> usize {
        self.connection
            .query_row(
                format!(
                    // language=sqlite
                    "SELECT COUNT(*) FROM {table}",
                    table = self.table
                )
                .as_str(),
                (),
                |row| -> Result<usize, Error> { row.get(0) },
            )
            .unwrap()
    }

    fn is_empty(&self) -> bool {
        self.connection
            .query_row(
                format!(
                    // language=sqlite
                    r#"
                        SELECT
                            EXISTS (
                                   SELECT *
                                       FROM
                                           {table}
                                   )
                    "#,
                    table = self.table
                )
                .as_str(),
                (),
                |row| -> Result<usize, Error> { row.get(0) },
            )
            .unwrap()
            == 0
    }

    fn is_not_empty(&self) -> bool {
        !self.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_starts_empty() {
        let dao = get_dao();
        assert!(dao.is_empty());
        assert!(!dao.is_not_empty());
        assert_eq!(dao.len(), 0);
    }

    #[test]
    fn retrieve_from_empty() {
        let mut dao = get_dao();

        let data = dao.retrieve(1).expect("dao should retrieve empty data");
        assert_eq!(data, vec![]);
        assert!(dao.is_empty());
        assert!(!dao.is_not_empty());
        assert_eq!(dao.len(), 0);
    }

    #[test]
    fn insert_and_retrieve_one() {
        let mut dao = get_dao();

        dao.insert(&vec![(100, 200)])
            .expect("dao should accept inserts");

        assert!(!dao.is_empty());
        assert!(dao.is_not_empty());
        assert_eq!(dao.len(), 1);

        let data = dao.retrieve(1).expect("dao should retrieve data");
        assert_eq!(data, vec![(100, 200)]);
        assert!(dao.is_empty());
        assert!(!dao.is_not_empty());
        assert_eq!(dao.len(), 0);
    }

    #[test]
    fn insert_and_retrieve_multiple() {
        let mut dao = get_dao();

        dao.insert(&vec![
            (900, 900),
            (100, 200),
            (500, 600),
            (300, 400),
            (700, 800),
        ])
        .expect("dao should accept inserts");

        assert!(!dao.is_empty());
        assert!(dao.is_not_empty());
        assert_eq!(dao.len(), 5);

        let data = dao.retrieve(5).expect("dao should retrieve data");
        // Additionally ensure that data was returned in ascending key order
        assert_eq!(
            data,
            vec![(100, 200), (300, 400), (500, 600), (700, 800), (900, 900)]
        );
        assert!(dao.is_empty());
        assert!(!dao.is_not_empty());
    }

    #[test]
    fn insert_and_retrieve_duplicate_priority() {
        let mut dao = get_dao();

        dao.insert(&vec![(100, 400), (100, 200)])
            .expect("dao should accept inserts");

        assert!(!dao.is_empty());
        assert!(dao.is_not_empty());
        assert_eq!(dao.len(), 2);

        let mut data = dao.retrieve(2).expect("dao should retrieve data");
        data.sort_by_key(|(_, value)| *value);

        assert_eq!(data, vec![(100, 200), (100, 400)]);
        assert!(dao.is_empty());
        assert!(!dao.is_not_empty());
    }

    #[test]
    fn insert_and_retrieve_multiple_one_by_one() {
        let mut dao = get_dao();

        // Insert one by one
        for i in 0..5 {
            dao.insert(&vec![vec![
                (900, 900),
                (100, 200),
                (500, 600),
                (300, 400),
                (700, 800),
            ]
            .get(i)
            .map(|x| *x)
            .unwrap()])
                .expect("dao should accept inserts");

            assert!(!dao.is_empty());
            assert!(dao.is_not_empty());
            assert_eq!(dao.len(), i + 1);
        }

        // Retrieve one by one
        for i in 0..5 {
            let data = dao.retrieve(1).expect("dao should retrieve data");
            assert_eq!(data.len(), 1);
            assert_eq!(dao.len(), 4 - i);
            // Additionally ensure that data was returned in ascending key order
            assert_eq!(
                data.first().unwrap(),
                vec![(100, 200), (300, 400), (500, 600), (700, 800), (900, 900)]
                    .get(i)
                    .unwrap()
            );
            if i < 4 {
                assert!(!dao.is_empty());
                assert!(dao.is_not_empty());
            } else {
                assert!(dao.is_empty());
                assert!(!dao.is_not_empty());
            }
        }
    }

    #[test]
    fn retrieve_more_than_len() {
        let mut dao = get_dao();

        dao.insert(&vec![
            (900, 900),
            (100, 200),
            (500, 600),
            (300, 400),
            (700, 800),
        ])
        .expect("dao should accept inserts");

        assert!(!dao.is_empty());
        assert!(dao.is_not_empty());
        assert_eq!(dao.len(), 5);

        let data = dao
            .retrieve(dao.len() + 10)
            .expect("dao should retrieve data");
        // Additionally ensure that data was returned in ascending key order
        assert_eq!(
            data,
            vec![(100, 200), (300, 400), (500, 600), (700, 800), (900, 900)]
        );
        assert!(dao.is_empty());
        assert!(!dao.is_not_empty());
    }

    fn get_dao() -> SqlitePrioQueueDao {
        SqlitePrioQueueDao::new(":memory:")
    }
}
