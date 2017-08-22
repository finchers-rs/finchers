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

mod todo {
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
    struct Todos {
        db: HashMap<u64, Todo>,
        counter: u64,
    }

    lazy_static! {
        static ref TODOS: RwLock<Todos> = RwLock::new(Todos::default());
    }

    pub fn get(id: u64) -> Option<Todo> {
        let todos = TODOS.read().unwrap();
        todos.db.get(&id).cloned()
    }

    pub fn set(id: u64, new_todo: Todo) {
        let mut todos = TODOS.write().unwrap();
        if let Some(todo) = todos.db.get_mut(&id) {
            *todo = new_todo;
        }
    }

    pub fn list() -> Vec<Todo> {
        let todos = TODOS.read().unwrap();
        todos.db.iter().map(|i| i.1.clone()).collect()
    }

    pub fn save(new_todo: NewTodo) -> Todo {
        let mut todos = TODOS.write().unwrap();
        todos.counter += 1;
        let todo = Todo {
            id: todos.counter,
            title: new_todo.title,
            completed: new_todo.completed.unwrap_or(false),
            order: new_todo.order.unwrap_or(0),
        };

        todos.db.insert(todo.id, todo.clone());
        todo
    }

    pub fn delete(id: u64) {
        let mut todos = TODOS.write().unwrap();
        todos.db.remove(&id);
    }

    pub fn clear() {
        let mut todos = TODOS.write().unwrap();
        todos.db.clear();
    }
}

fn main() {
    // GET /todos/:id
    let get_todo = get("todos".with(u64_)).map(|id| Json(todo::get(id)));

    // GET /todos
    let get_todos = get("todos").map(|()| Json(todo::list()));

    // DELETE /todos/:id
    let delete_todo = delete("todos".with(u64_)).map(|id| { todo::delete(id); });

    // DELETE /todos
    let delete_todos = delete("todos").map(|()| { todo::clear(); });

    // PUT /todos/:id
    let patch_todo = put("todos".with(u64_))
        .join(json_body::<todo::Todo>())
        .map(|(id, Json(new_todo))| { todo::set(id, new_todo); });

    // POST /todos
    let post_todo = post("todos")
        .with(json_body::<todo::NewTodo>())
        .map(|Json(new_todo)| {
            let new_todo = todo::save(new_todo);
            Created(Json(new_todo))
        });

    let endpoint = get_todo
        .or(get_todos)
        .or(delete_todo)
        .or(delete_todos)
        .or(patch_todo)
        .or(post_todo);

    finchers::server::run_http(endpoint, "127.0.0.1:3000");
}
