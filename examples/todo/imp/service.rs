use std::sync::RwLock;

#[derive(Debug, Clone, Serialize)]
pub struct Todo {
    id: u64,
    title: String,
    completed: bool,
}

#[derive(Debug, Deserialize)]
pub struct NewTodo {
    title: String,
}

#[derive(Debug, Deserialize)]
pub struct PatchTodo {
    title: Option<String>,
    completed: Option<bool>,
}

error_chain!{}

#[derive(Default)]
struct TodoRepository {
    todos: Vec<Todo>,
    counter: u64,
}

lazy_static! {
    static ref REPOSITORY: RwLock<TodoRepository> = Default::default();
}

pub fn find_todo(id: u64) -> Result<Option<Todo>> {
    let repo = REPOSITORY.read().map_err(|e| e.to_string())?;
    Ok(repo.todos.iter().find(|todo| todo.id == id).cloned())
}

pub fn get_todos() -> Result<Vec<Todo>> {
    let repo = REPOSITORY.read().map_err(|e| e.to_string())?;
    Ok(repo.todos.clone())
}

pub fn add_new_todo(entry: NewTodo) -> Result<Todo> {
    let mut repo = REPOSITORY.write().map_err(|e| e.to_string())?;
    let todo = Todo {
        id: repo.counter,
        title: entry.title,
        completed: false,
    };
    repo.todos.push(todo.clone());
    repo.counter += 1;
    Ok(todo)
}

pub fn update_todo(id: u64, patch: PatchTodo) -> Result<Option<Todo>> {
    let mut repo = REPOSITORY.write().map_err(|e| e.to_string())?;
    if let Some(todo) = repo.todos.iter_mut().find(|todo| todo.id == id) {
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

pub fn remove_todo(id: u64) -> Result<()> {
    let mut repo = REPOSITORY.write().map_err(|e| e.to_string())?;
    if let Some(pos) = repo.todos.iter().position(|todo| todo.id == id) {
        repo.todos.remove(pos);
    }
    Ok(())
}
