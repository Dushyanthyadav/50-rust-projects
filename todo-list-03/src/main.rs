use std::io::Write;
use std::process::exit;
use std::{
    fmt::{self},
    io,
};

struct Task {
    id: u64,
    description: String,
    status: TaskStatus,
}

impl Task {
    fn new(id: u64, description: String, status: TaskStatus) -> Self {
        Self {
            id,
            description,
            status,
        }
    }
}
#[derive(PartialEq)]
enum TaskStatus {
    Pending,
    Completed,
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status: String;

        if self.status == TaskStatus::Pending {
            status = String::from("Pending");
        } else {
            status = String::from("Completed");
        }

        write!(
            f,
            "Id: {} Task: {}. Status: {}",
            self.id, self.description, status
        )
    }
}

struct TodoList {
    completed: Vec<Task>,
    pending: Vec<Task>,
}

impl TodoList {
    fn new() -> Self {
        Self {
            completed: vec![],
            pending: vec![],
        }
    }

    fn add(&mut self, task: Task) {
        if task.status == TaskStatus::Completed {
            self.completed.push(task);
        } else {
            self.pending.push(task);
        }
    }

    fn update_status(&mut self, search_id: u64, new_status: TaskStatus) {
        if new_status == TaskStatus::Pending {
            if let Some(pos) = self.completed.iter().position(|task| task.id == search_id) {
            let mut task = self.completed.remove(pos);
            task.status = TaskStatus::Pending;
            self.add(task);
            }
            
        } else {
            if let Some(pos) = self.pending.iter().position(|task| task.id == search_id) {
            let mut task = self.pending.remove(pos);
            task.status = TaskStatus::Completed;
            self.add(task);
        }
        }
    }

    fn delete(&mut self, delete_id: u64) {
        if let Some(pos) = self.completed.iter().position(|task| task.id == delete_id) {
            self.completed.remove(pos);
        }
        if let Some(pos) = self.pending.iter().position(|task| task.id == delete_id) {
            self.pending.remove(pos);
        }
    }

    fn show(&self) {
        println!("\n##########Pending Task############");
        for task in &self.pending {
            println!("{}", task);
        }
        println!("\n#########Compelted task###########");
        for task in &self.completed {
            println!("{}", task);
        }
    }
}

fn main() {
    println!("Terminal Todo list!!!");
    let mut id: u64 = 0;
    let mut ids: Vec<u64> = Vec::new();
    ids.push(id);
    let mut todo_list = TodoList::new();
    loop {
        let mut choice: String = String::new();
        println!("\n1. Add");
        println!("2. show");
        println!("3. Update status");
        println!("4. delete");
        println!("5. exit");

        print!("Enter the Choice: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut choice).unwrap();
        let choice: u8 = choice.trim().parse().unwrap();

        match choice {
            1 => {
                let mut task_description = String::new();
                print!("Enter the task: ");
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut task_description).unwrap();
                let task_description = String::from(task_description.trim());
                while ids.contains(&id) {
                    id = id + 1;
                }

                let new_task = Task::new(id, task_description, TaskStatus::Pending);
                ids.push(id);
                todo_list.add(new_task);
            }
            2 => {
                todo_list.show();
            }
            3 => {
                let mut id: String = String::new();
                print!("Enter the id: ");
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut id).unwrap();
                let id = id.trim().parse::<u64>().unwrap();
                println!("1. Completed\n2. Pending");
                print!("Enter the new status: ");
                io::stdout().flush().unwrap();
                let mut new_status = String::new();
                io::stdin().read_line(&mut new_status).unwrap();
                let new_status = new_status.trim().parse::<u8>().unwrap();
                if new_status == 1 {
                    todo_list.update_status(id, TaskStatus::Completed);
                } else {
                    todo_list.update_status(id, TaskStatus::Pending);
                }
            }
            4 => {
                let mut delete_id: String = String::new();
                print!("Enter the id: ");
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut delete_id).unwrap();
                let delete_id = delete_id.trim().parse::<u64>().unwrap();
                todo_list.delete(delete_id);
                let pos = ids.iter().position(|pos_id| *pos_id == delete_id).unwrap();
                ids.remove(pos);
            }
            5 => {
                exit(0);
            }
            _ => {
                continue;
            }
        }
    }
}
