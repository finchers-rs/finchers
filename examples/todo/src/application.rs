use super::db::{self, NewTodo, PatchTodo, Todo, TodoRepository};
use finchers::http::StatusCode;
use finchers::response::HttpStatus;

error_chain! {
    foreign_links {
        Database(db::Error);
    }
}

impl HttpStatus for Error {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(Debug, Clone)]
pub struct Application {
    todos: TodoRepository,
}

impl Application {
    pub fn find_todo(&self, id: u64) -> Result<Option<Todo>> {
        Ok(self.todos.find(id)?)
    }

    pub fn list_todos(&self) -> Result<Vec<Todo>> {
        Ok(self.todos.list()?)
    }

    pub fn add_todo(&self, new_todo: NewTodo) -> Result<Todo> {
        Ok(self.todos.add(new_todo)?)
    }

    pub fn patch_todo(&self, id: u64, patch: PatchTodo) -> Result<Option<Todo>> {
        Ok(self.todos.patch(id, patch)?)
    }

    pub fn delete_todo(&self, id: u64) -> Result<Option<()>> {
        Ok(self.todos.delete(id)?)
    }
}

pub fn new() -> Application {
    let todos = TodoRepository::default();
    todos
        .add(NewTodo {
            title: "Read TRPL".to_string(),
        })
        .unwrap();
    todos
        .add(NewTodo {
            title: "Eat breakfast".to_string(),
        })
        .unwrap();

    Application { todos }
}
