#![feature(async_await, await_macro, futures_api)]

use finchers::endpoint::{lazy, EndpointExt};
use finchers::endpoints::body;
use finchers::{route, routes};

use crate::db::ConnPool;

fn main() {
    let pool = ConnPool::default();

    // Create an endpoint which establishes a connection to the DB.
    let conn = lazy(move |_| {
        let conn = pool.conn();
        async { Ok(conn) }
    });

    let find_todo = route!(@get / u64 /)
        .and(conn.clone())
        .and_then(crate::api::find_todo);

    let list_todos = route!(@get /)
        .and(conn.clone())
        .and_then(crate::api::list_todos);

    let add_todo = route!(@post /)
        .and(body::json())
        .and(conn.clone())
        .and_then(crate::api::create_todo);

    let patch_todo = route!(@patch / u64 /)
        .and(body::json())
        .and(conn.clone())
        .and_then(crate::api::patch_todo);

    let delete_todo = route!(@delete / u64 /)
        .and(conn.clone())
        .and_then(crate::api::delete_todo);

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
    use failure::format_err;

    use finchers::error::Error;
    use finchers::output::status::Created;
    use finchers::output::{Json, Responder};

    use crate::db::Conn;
    use crate::model::{NewTodo, PatchTodo, Todo};

    pub async fn find_todo(id: u64, conn: Conn) -> Result<impl Responder, Error> {
        let db = conn.read()?;
        let found = db.todos.iter().find(|todo| todo.id == id).cloned();
        Ok(found.map(Json))
    }

    pub async fn list_todos(conn: Conn) -> Result<impl Responder, Error> {
        let db = conn.read()?;
        Ok(Json(db.todos.clone()))
    }

    pub async fn create_todo(new_todo: NewTodo, mut conn: Conn) -> Result<impl Responder, Error> {
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

        Ok(Created(Json(new_todo)))
    }

    pub async fn patch_todo(
        id: u64,
        patch: PatchTodo,
        mut conn: Conn,
    ) -> Result<impl Responder, Error> {
        let mut db = conn.write()?;

        Ok(db.todos.iter_mut().find(|todo| todo.id == id).map(|todo| {
            if let Some(text) = patch.text {
                todo.text = text;
            }
            if let Some(completed) = patch.completed {
                todo.completed = completed;
            }

            Json(todo.clone())
        }))
    }

    pub async fn delete_todo(id: u64, mut conn: Conn) -> Result<impl Responder, Error> {
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
