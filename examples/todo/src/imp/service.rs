use std::fmt;
use std::error::Error;
use std::sync::{Arc, RwLock};
use http::StatusCode;
use finchers::core::HttpResponse;
use super::model::{NewTodo, PatchTodo, Todo, TodoRepository};

#[derive(Debug)]
pub enum ServiceError {
    Poisoned,
}

impl<T> From<::std::sync::PoisonError<T>> for ServiceError {
    fn from(_: ::std::sync::PoisonError<T>) -> Self {
        ServiceError::Poisoned
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ServiceError::Poisoned => f.write_str("failed to acquire a lock"),
        }
    }
}

impl Error for ServiceError {
    fn description(&self) -> &str {
        "server error"
    }
}

impl HttpResponse for ServiceError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub type Result<T> = ::std::result::Result<T, ServiceError>;

#[derive(Debug, Clone)]
pub struct Service {
    todos: Arc<RwLock<TodoRepository>>,
}

impl Service {
    pub fn find_todo(&self) -> impl Fn(u64) -> Result<Option<Todo>> + 'static {
        let todos = self.todos.clone();
        move |id| -> Result<Option<Todo>> {
            let todos = todos.read()?;
            Ok(todos.find(id))
        }
    }

    pub fn list_todos(&self) -> impl Fn(()) -> Result<Vec<Todo>> + 'static {
        let todos = self.todos.clone();
        move |()| -> Result<Vec<Todo>> {
            let todos = todos.read()?;
            Ok(todos.list())
        }
    }

    pub fn add_todo(&self) -> impl Fn(NewTodo) -> Result<Todo> + 'static {
        let todos = self.todos.clone();
        move |new_todo| {
            let mut todos = todos.write()?;
            Ok(todos.add(new_todo))
        }
    }

    pub fn apply_patch(&self) -> impl Fn((u64, PatchTodo)) -> Result<Option<Todo>> + 'static {
        let todos = self.todos.clone();
        move |(id, patch)| {
            let mut todos = todos.write()?;
            Ok(todos.patch(id, patch))
        }
    }

    pub fn delete_todo(&self) -> impl Fn(u64) -> Result<()> + 'static {
        let todos = self.todos.clone();
        move |id| {
            let mut todos = todos.write()?;
            todos.delete(id);
            Ok(())
        }
    }
}

pub fn new_service() -> Service {
    let mut todos = TodoRepository::default();
    todos.add(NewTodo {
        title: "Read TRPL".to_string(),
    });
    todos.add(NewTodo {
        title: "Eat breakfast".to_string(),
    });

    Service {
        todos: Arc::new(RwLock::new(todos)),
    }
}
