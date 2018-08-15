#![feature(async_await, await_macro, futures_api, pin, integer_atomics)]

use failure::Fallible;

use finchers::endpoint::EndpointExt;
use finchers::endpoints::body;
use finchers::{route, routes};

use crate::db::ConnPool;

fn main() -> Fallible<()> {
    let pool = ConnPool::default();

    let find_todo = route!(@get / u64 /)
        .and(pool.clone())
        .and_then(crate::api::find_todo);

    let list_todos = route!(@get /)
        .and(pool.clone())
        .and_then(crate::api::list_todos);

    let add_todo = route!(@post /)
        .and(body::json())
        .and(pool.clone())
        .and_then(crate::api::create_todo);

    let patch_todo = route!(@patch / u64 /)
        .and(body::json())
        .and(pool.clone())
        .and_then(crate::api::patch_todo);

    let delete_todo = route!(@delete / u64 /)
        .and(pool.clone())
        .and_then(crate::api::delete_todo);

    let endpoint = route!(/ "api" / "v1" / "todos").and(routes![
        find_todo,
        list_todos,
        add_todo,
        patch_todo,
        delete_todo,
    ]);

    finchers::rt::launch(endpoint)?;
    Ok(())
}

mod api {
    use failure::format_err;
    use std::mem;

    use finchers::error::{internal_server_error, Error};
    use finchers::output::status::Created;
    use finchers::output::Json;
    use finchers::output::Responder;

    use crate::db::Conn;
    use crate::model::{NewTodo, PatchTodo, Todo};

    pub async fn find_todo(id: u64, conn: Conn) -> Result<impl Responder, Error> {
        let db = conn.read().map_err(internal_server_error)?;
        let found = db.todos.iter().find(|todo| todo.id == id).cloned();
        Ok(found.map(Json))
    }

    pub async fn list_todos(conn: Conn) -> Result<impl Responder, Error> {
        let db = conn.read().map_err(internal_server_error)?;
        Ok(Json(db.todos.clone()))
    }

    pub async fn create_todo(new_todo: NewTodo, mut conn: Conn) -> Result<impl Responder, Error> {
        let mut db = conn.write().map_err(internal_server_error)?;

        let new_todo = Todo {
            id: db.counter,
            text: new_todo.text,
            completed: new_todo.completed,
        };

        let new_id = match db.counter.checked_add(1) {
            Some(id) => id,
            None => return Err(internal_server_error(format_err!("overflow detected"))),
        };
        mem::replace(&mut db.counter, new_id);

        db.todos.push(new_todo.clone());

        Ok(Created(Json(new_todo)))
    }

    pub async fn patch_todo(
        id: u64,
        patch: PatchTodo,
        mut conn: Conn,
    ) -> Result<impl Responder, Error> {
        let mut db = conn.write().map_err(internal_server_error)?;

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
        let mut db = conn.write().map_err(internal_server_error)?;

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
    use futures::future::{ready, Ready};
    use std::mem::PinMut;
    use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

    use finchers::endpoint::Endpoint;
    use finchers::error::Error;
    use finchers::input::{Cursor, Input};

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

    impl Endpoint for ConnPool {
        type Output = (Conn,);
        type Future = Ready<Result<(Conn,), Error>>;

        fn apply<'c>(&self, _: PinMut<Input>, c: Cursor<'c>) -> Option<(Self::Future, Cursor<'c>)> {
            Some((
                ready(Ok((Conn {
                    db: self.db.clone(),
                },))),
                c,
            ))
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
