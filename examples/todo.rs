extern crate finchers;
#[macro_use]
extern crate lazy_static;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use finchers::{Endpoint, Json};
use finchers::endpoint::{json_body, u64_};
use finchers::endpoint::method::{delete, get, post, put};
use finchers::response::Created;

use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Todo {
    id: u64,
    title: String,
    completed: bool,
    order: usize,
}

#[derive(Deserialize)]
pub struct NewTodo {
    title: String,
    completed: Option<bool>,
    order: Option<usize>,
}

#[derive(Default)]
pub struct Todos {
    db: HashMap<u64, Todo>,
    counter: u64,
}

impl Todos {
    pub fn get(&self, id: u64) -> Option<Todo> {
        self.db.get(&id).cloned()
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Todo> {
        self.db.get_mut(&id)
    }

    pub fn list(&self) -> Vec<Todo> {
        self.db.iter().map(|i| i.1.clone()).collect()
    }

    pub fn save(&mut self, new_todo: NewTodo) -> Todo {
        self.counter += 1;
        let todo = Todo {
            id: self.counter,
            title: new_todo.title,
            completed: new_todo.completed.unwrap_or(false),
            order: new_todo.order.unwrap_or(0),
        };

        self.db.insert(todo.id, todo.clone());
        todo
    }

    pub fn delete(&mut self, id: u64) {
        self.db.remove(&id);
    }

    pub fn clear(&mut self) {
        self.db.clear();
    }
}

fn main() {
    lazy_static! {
        pub static ref TODOS: RwLock<Todos> = RwLock::new(Todos::default());
    }

    // GET /todos/:id
    let get_todo = get("todos".with(u64_)).map(|id| {
        let todos = TODOS.read().unwrap();
        Json(todos.get(id))
    });

    // GET /todos
    let get_todos = get("todos").map(|()| {
        let todos = TODOS.read().unwrap();
        Json(todos.list())
    });

    // DELETE /todos/:id
    let delete_todo = delete("todos".with(u64_)).map(|id| {
        let mut todos = TODOS.write().unwrap();
        todos.delete(id);
    });

    // DELETE /todos
    let delete_todos = delete("todos").map(|()| {
        let mut todos = TODOS.write().unwrap();
        todos.clear();
    });

    // PUT /todos/:id
    let patch_todo = put("todos".with(u64_))
        .join(json_body::<Todo>())
        .map(|(id, Json(new_todo))| {
            let mut todos = TODOS.write().unwrap();
            if let Some(todo) = todos.get_mut(id) {
                *todo = new_todo;
            }
        });

    // POST /todos
    let post_todo = post("todos")
        .with(json_body::<NewTodo>())
        .map(|Json(new_todo)| {
            let mut todos = TODOS.write().unwrap();
            Created(Json(todos.save(new_todo)))
        });

    let endpoint = get_todo
        .or(get_todos)
        .or(delete_todo)
        .or(delete_todos)
        .or(patch_todo)
        .or(post_todo);

    finchers::server::run_http(endpoint, "127.0.0.1:3000");
}
