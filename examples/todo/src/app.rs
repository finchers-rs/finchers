use super::db::{self, NewTodo, PatchTodo, Todo, TodoRepository};
use finchers::error::HttpError;
use http::StatusCode;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "no entity")]
    NoEntity,

    #[fail(display = "{}", cause)]
    Database { cause: db::Error },
}

impl From<db::Error> for Error {
    fn from(cause: db::Error) -> Self {
        Error::Database { cause }
    }
}

impl HttpError for Error {
    fn status_code(&self) -> StatusCode {
        match *self {
            Error::NoEntity => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Application {
    todos: TodoRepository,
}

impl Application {
    pub fn find_todo(&self, id: u64) -> Result<Todo> {
        self.todos.find(id)?.ok_or_else(|| Error::NoEntity)
    }

    pub fn list_todos(&self) -> Result<Vec<Todo>> {
        Ok(self.todos.list()?)
    }

    pub fn add_todo(&self, new_todo: NewTodo) -> Result<Todo> {
        Ok(self.todos.add(new_todo)?)
    }

    pub fn patch_todo(&self, id: u64, patch: PatchTodo) -> Result<Todo> {
        self.todos.patch(id, patch)?.ok_or_else(|| Error::NoEntity)
    }

    pub fn delete_todo(&self, id: u64) -> Result<()> {
        self.todos.delete(id)?.ok_or_else(|| Error::NoEntity)
    }

    pub fn with<F, T, R>(&self, f: F) -> impl FnOnce(T) -> R + Send + Clone + 'static
    where
        F: FnOnce(Application, T) -> R + Send + Clone + 'static,
    {
        let app = self.clone();
        move |arg| f(app, arg)
    }
}

pub fn new() -> Application {
    let todos = TodoRepository::default();
    todos
        .add(NewTodo {
            title: "Read TRPL".to_string(),
        }).unwrap();
    todos
        .add(NewTodo {
            title: "Eat breakfast".to_string(),
        }).unwrap();

    Application { todos }
}
