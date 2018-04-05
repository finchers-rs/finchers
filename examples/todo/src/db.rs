use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    Poisoned,
}

impl<T> From<::std::sync::PoisonError<T>> for Error {
    fn from(_: ::std::sync::PoisonError<T>) -> Self {
        Error::Poisoned
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Poisoned => f.write_str("failed to acquire a lock"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "server error"
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub completed: bool,
}

#[derive(Debug, Deserialize)]
pub struct NewTodo {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct PatchTodo {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

#[derive(Debug, Default, Clone)]
pub struct TodoRepository {
    database: Arc<RwLock<HashMap<u64, Todo>>>,
}

impl TodoRepository {
    pub fn add(&self, new_todo: NewTodo) -> Result<Todo> {
        let mut database = self.database.write()?;
        let id = database.keys().max_by_key(|x| *x).map_or(0, |x| *x + 1);
        let todo = Todo {
            id,
            title: new_todo.title,
            completed: false,
        };
        database.insert(id, todo.clone());
        Ok(todo)
    }

    pub fn find(&self, id: u64) -> Result<Option<Todo>> {
        let database = self.database.read()?;
        Ok(database.get(&id).cloned())
    }

    pub fn list(&self) -> Result<Vec<Todo>> {
        let database = self.database.read()?;
        Ok(database.values().cloned().collect())
    }

    pub fn patch(&self, id: u64, patch: PatchTodo) -> Result<Option<Todo>> {
        let mut database = self.database.write()?;
        if let Some(todo) = database.get_mut(&id) {
            if let Some(title) = patch.title {
                todo.title = title;
            }
            if let Some(completed) = patch.completed {
                todo.completed = completed;
            }
            Ok(Some(todo.clone()))
        } else {
            Ok(None)
        }
    }

    pub fn delete(&self, id: u64) -> Result<Option<()>> {
        let mut database = self.database.write()?;
        Ok(database.remove(&id).map(|_| ()))
    }
}
