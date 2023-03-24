use postgres::{Client, Error, NoTls, Row};
use uuid::Uuid;

struct Task {
    id: Uuid,
    title: String,
    completed: bool,
}

impl From<Row> for Task {
    fn from(value: Row) -> Self {
        Self {
            id: value.get(0),
            title: value.get(1),
            completed: (value.get(2)),
        }
    }
}

struct TaskClient {
    client: Client,
}

impl TaskClient {
    fn create_table(&mut self) -> Result<(), Error> {
        self.client.batch_execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
                title TEXT NOT NULL,
                completed BOOLEAN NOT NULL
            )",
        )
    }

    fn add_task(&mut self, title: &str) -> Result<u64, Error> {
        self.client.execute(
            "INSERT INTO tasks (title, completed) VALUES ($1, $2)",
            &[&title, &false],
        )
    }

    fn set_task_completed(&mut self, id: &Uuid) -> Result<u64, Error> {
        self.client
            .execute("UPDATE tasks SET completed = true WHERE id = $1", &[&id])
    }

    fn delete_task(&mut self, id: &Uuid) -> Result<u64, Error> {
        self.client
            .execute("DELETE FROM tasks WHERE id = $1", &[&id])
    }

    fn get_task_by_id(&mut self, id: &Uuid) -> Task {
        let row = self
            .client
            .query_one("SELECT * FROM tasks WHERE id = $1", &[&id]);

        match row {
            Ok(result) => Task::from(result),
            Err(error) => panic!("{:?}", error),
        }
    }

    fn get_all_tasks(&mut self) -> Vec<Task> {
        let result = self.client.query("SELECT * FROM tasks", &[]);

        let tasks = match result {
            Ok(rows) => rows.into_iter().map(|row| Task::from(row)).collect(),
            Err(error) => panic!("{:?}", error),
        };

        return tasks;
    }
}

fn init_client(url: &str) -> Client {
    Client::connect(url, NoTls).expect("Failed to create client")
}

#[cfg(test)]
mod test {
    use crate::database;
    use dotenv::dotenv;
    use std::env;

    fn setup_tasks_client() -> database::TaskClient {
        dotenv().ok();

        let database_url =
            env::var("DATABASE_URL").expect("A database url doesn't exist in enviroment");

        let mut client = database::TaskClient {
            client: database::init_client(&database_url),
        };

        client.create_table().expect("Failed to create table");

        return client;
    }

    #[test]
    fn it_returns_all_tasks() {
        let mut client = setup_tasks_client();
        let tasks = client.get_all_tasks();
        assert_ne!(tasks.len(), 0)
    }

    #[test]
    fn it_creates_a_task() {
        let mut client = setup_tasks_client();

        let rows_updated = match client.add_task("TEST_TASK") {
            Ok(rows) => rows,
            Err(error) => panic!("Error: {:?}", error),
        };

        assert_ne!(rows_updated, 0)
    }

    #[test]
    fn it_returns_a_task() {
        let mut client = setup_tasks_client();

        let all_tasks = client.get_all_tasks();

        let task_id = &all_tasks[0].id;

        let task = client.get_task_by_id(task_id);

        assert_ne!(task.id.to_string(), "");
    }

    #[test]
    fn it_deletes_a_task() {
        let mut client = setup_tasks_client();

        let all_tasks = client.get_all_tasks();

        let task = &all_tasks[0];

        let rows_updated = match client.delete_task(&task.id) {
            Ok(rows) => rows,
            Err(error) => panic!("{:?}", error),
        };

        assert_ne!(rows_updated, 0);
    }

    #[test]
    fn it_updates_task_status() {
        let mut client = setup_tasks_client();

        let all_tasks = client.get_all_tasks();

        let task = &all_tasks[0];

        let rows_updated = match client.set_task_completed(&task.id) {
            Ok(rows) => rows,
            Err(error) => panic!("{:?}", error),
        };

        assert_ne!(rows_updated, 0)
    }
}
