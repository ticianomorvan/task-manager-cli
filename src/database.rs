use postgres::{Client, Error, NoTls, Row};

struct Task {
    id: String,
    name: String,
    completed: bool,
}

impl From<Row> for Task {
    fn from(value: Row) -> Self {
        Self {
            id: value.get(0),
            name: value.get(1),
            completed: (value.get(2)),
        }
    }
}

fn init_client(url: &str) -> Client {
    Client::connect(url, NoTls).expect("Failed to create client")
}

fn create_table(client: &mut Client) -> Result<(), Error> {
    client.batch_execute(
        "
        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            completed BOOLEAN NOT NULL
        )
    ",
    )
}

fn add_task(client: &mut Client, task: Task) -> Result<u64, Error> {
    client.execute(
        "INSERT INTO tasks (id, name, completed) VALUES ($1, $2, $3)",
        &[&task.id.to_string(), &task.name, &task.completed],
    )
}

fn update_task(client: &mut Client, id: String, status: bool) -> Result<u64, Error> {
    client.execute(
        "UPDATE tasks SET completed = $1 WHERE id = $2",
        &[&status, &id],
    )
}

fn delete_task(client: &mut Client, id: String) -> Result<u64, Error> {
    client.execute("DELETE FROM tasks WHERE id = $1", &[&id])
}

fn get_task_by_id(client: &mut Client, id: String) -> Task {
    let row = client
        .query_one("SELECT * FROM tasks WHERE id = $1", &[&id])
        .expect("Failed to find task");

    let task = Task {
        id: row.get(0),
        name: row.get(1),
        completed: row.get(2),
    };

    return task;
}

fn get_all_tasks(client: &mut Client) -> Vec<Task> {
    let tasks: Vec<Row> = client
        .query("SELECT * FROM tasks", &[])
        .expect("Failed to get all tasks");

    let tasks: Vec<Task> = tasks.into_iter().map(|row| Task::from(row)).collect();

    return tasks;
}

#[cfg(test)]
mod test {
    use crate::database;
    use dotenv::dotenv;
    use postgres::Client;
    use std::env;
    use uuid::Uuid;

    fn setup_client() -> Client {
        dotenv().ok();

        let database_url =
            env::var("DATABASE_URL").expect("A database url doesn't exist in enviroment");

        database::init_client(database_url.as_str())
    }

    #[test]
    fn it_returns_all_tasks() {
        let mut client = setup_client();
        let tasks = database::get_all_tasks(&mut client);
        assert_ne!(tasks.len(), 0)
    }

    #[test]
    fn it_creates_a_task() {
        let mut client = setup_client();

        let task = database::Task {
            id: Uuid::new_v4().to_string(),
            name: "TEST_TASK".to_string(),
            completed: false,
        };

        let rows_updated = match database::add_task(&mut client, task) {
            Ok(rows) => rows,
            Err(error) => panic!("Error: {:?}", error),
        };

        assert_ne!(rows_updated, 0)
    }

    #[test]
    fn it_returns_a_task() {
        let mut client = setup_client();

        let all_tasks = database::get_all_tasks(&mut client);

        let task_id = &all_tasks[0].id;

        let task = database::get_task_by_id(&mut client, task_id.to_string());

        assert_ne!(task.id, "");
    }

    #[test]
    fn it_deletes_a_task() {
        let mut client = setup_client();

        let all_tasks = database::get_all_tasks(&mut client);

        let task = &all_tasks[0];

        let delete_result = database::delete_task(&mut client, task.id.to_string())
            .expect("Failed to delete the task");

        assert_ne!(delete_result, 0);
    }

    #[test]
    fn it_updates_task_status() {
        let mut client = setup_client();

        let all_tasks = database::get_all_tasks(&mut client);

        let task = &all_tasks[0];

        let result = database::update_task(&mut client, task.id.to_string(), true)
            .expect("Failed to update task's status");

        assert_ne!(result, 0)
    }
}
