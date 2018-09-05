#![feature(async_await, await_macro, futures_api)]

use finchers::endpoint::{unit, EndpointExt};
use finchers::endpoints::body;
use finchers::{output, route, routes};

use crate::db::ConnPool;

fn main() {
    // Create an endpoint which establishes a connection to the DB.
    let pool = ConnPool::default();
    let conn = unit().map(move || pool.conn());

    let find_todo = route!(@get / u64 /)
        .and(conn.clone())
        .and_then(async move |id, conn| {
            await!(crate::api::find_todo(id, conn))?.ok_or_else(crate::util::not_found)
        }).map(output::Json);

    let list_todos = route!(@get /)
        .and(conn.clone())
        .and_then(async move |conn| Ok(await!(crate::api::list_todos(conn))?))
        .map(output::Json);

    let add_todo = route!(@post /)
        .and(body::json())
        .and(conn.clone())
        .and_then(async move |new_todo, conn| Ok(await!(crate::api::create_todo(new_todo, conn))?))
        .map(output::Json);

    let patch_todo = route!(@patch / u64 /)
        .and(body::json())
        .and(conn.clone())
        .and_then(async move |id, patch, conn| {
            await!(crate::api::patch_todo(id, patch, conn))?.ok_or_else(crate::util::not_found)
        }).map(output::Json);

    let delete_todo = route!(@delete / u64 /)
        .and(conn.clone())
        .and_then(async move |id, conn| {
            await!(crate::api::delete_todo(id, conn))?.ok_or_else(crate::util::not_found)
        });

    let endpoint = route!(/ "api" / "v1" / "todos").and(routes![
        find_todo,
        list_todos,
        add_todo,
        patch_todo,
        delete_todo,
    ]);

    finchers::launch(endpoint).start("127.0.0.1:4000")
}

mod api {
    use failure::{format_err, Fallible};

    use crate::db::Conn;
    use crate::model::{NewTodo, PatchTodo, Todo};

    pub async fn find_todo(id: u64, conn: Conn) -> Fallible<Option<Todo>> {
        let db = conn.read()?;
        let found = db.todos.iter().find(|todo| todo.id == id).cloned();
        Ok(found)
    }

    pub async fn list_todos(conn: Conn) -> Fallible<Vec<Todo>> {
        let db = conn.read()?;
        Ok(db.todos.clone())
    }

    pub async fn create_todo(new_todo: NewTodo, mut conn: Conn) -> Fallible<Todo> {
        let mut db = conn.write()?;

        let new_todo = Todo {
            id: db.counter,
            text: new_todo.text,
            completed: new_todo.completed,
        };

        db.counter = db
            .counter
            .checked_add(1)
            .ok_or_else(|| format_err!("overflow detected"))?;

        db.todos.push(new_todo.clone());

        Ok(new_todo)
    }

    pub async fn patch_todo(id: u64, patch: PatchTodo, mut conn: Conn) -> Fallible<Option<Todo>> {
        let mut db = conn.write()?;

        Ok(db.todos.iter_mut().find(|todo| todo.id == id).map(|todo| {
            if let Some(text) = patch.text {
                todo.text = text;
            }
            if let Some(completed) = patch.completed {
                todo.completed = completed;
            }

            todo.clone()
        }))
    }

    pub async fn delete_todo(id: u64, mut conn: Conn) -> Fallible<Option<()>> {
        let mut db = conn.write()?;

        if let Some(pos) = db.todos.iter().position(|todo| todo.id == id) {
            db.todos.remove(pos);
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }
}

mod model {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Todo {
        pub id: u64,
        pub text: String,
        pub completed: bool,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct NewTodo {
        pub text: String,
        #[serde(default)]
        pub completed: bool,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct PatchTodo {
        pub text: Option<String>,
        pub completed: Option<bool>,
    }
}

mod db {
    use failure::{format_err, Fallible};
    use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

    use crate::model::Todo;

    #[derive(Debug, Default)]
    pub struct Database {
        pub todos: Vec<Todo>,
        pub counter: u64,
    }

    #[derive(Debug, Default, Clone)]
    pub struct ConnPool {
        db: Arc<RwLock<Database>>,
    }

    impl ConnPool {
        pub fn conn(&self) -> Conn {
            Conn {
                db: self.db.clone(),
            }
        }
    }

    #[derive(Debug)]
    pub struct Conn {
        db: Arc<RwLock<Database>>,
    }

    impl Conn {
        pub fn read(&self) -> Fallible<RwLockReadGuard<Database>> {
            self.db.read().map_err(|e| format_err!("{}", e))
        }

        pub fn write(&mut self) -> Fallible<RwLockWriteGuard<Database>> {
            self.db.write().map_err(|e| format_err!("{}", e))
        }
    }
}

mod util {
    use finchers::error::{err_msg, Error};
    use http::StatusCode;

    pub fn not_found() -> Error {
        err_msg(StatusCode::NOT_FOUND, "not found")
    }
}
