use std::collections::HashMap;

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

#[derive(Debug, Default)]
pub struct TodoRepository {
    database: HashMap<u64, Todo>,
}

impl TodoRepository {
    pub fn add(&mut self, new_todo: NewTodo) -> Todo {
        let id = self.database
            .keys()
            .max_by_key(|x| *x)
            .map_or(0, |x| *x + 1);
        let todo = Todo {
            id,
            title: new_todo.title,
            completed: false,
        };
        self.database.insert(id, todo.clone());
        todo
    }

    pub fn find(&self, id: u64) -> Option<Todo> {
        self.database.get(&id).cloned()
    }

    pub fn list(&self) -> Vec<Todo> {
        self.database.values().cloned().collect()
    }

    pub fn patch(&mut self, id: u64, patch: PatchTodo) -> Option<Todo> {
        if let Some(todo) = self.database.get_mut(&id) {
            if let Some(title) = patch.title {
                todo.title = title;
            }
            if let Some(completed) = patch.completed {
                todo.completed = completed;
            }
            Some(todo.clone())
        } else {
            None
        }
    }

    pub fn delete(&mut self, id: u64) {
        self.database.remove(&id);
    }
}
