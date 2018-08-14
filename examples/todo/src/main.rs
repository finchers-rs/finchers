#![feature(rust_2018_preview)]
#![feature(futures_api, pin)]
#![feature(integer_atomics)]

extern crate failure;
extern crate finchers;
extern crate futures;
extern crate http;
extern crate serde;

use db::Database;

fn main() -> finchers::rt::LaunchResult<()> {
    let database = Database::default();
    let endpoint = endpoint(database);
    finchers::rt::launch(endpoint)
}

fn endpoint(db: Database) -> impl finchers::rt::AppEndpoint {
    use finchers::endpoint::EndpointExt;
    use finchers::endpoints::body;
    use finchers::output::responders::Json;
    use finchers::{route, routes};

    use futures::future::TryFutureExt;

    use util::Created;

    let find_todo = route!(@get / u64 /)
        .and(&db)
        .and_then(|id, conn| db::find_todo(conn, id).map_ok(|todo_opt| todo_opt.map(Json)));

    let list_todos = route!(@get /)
        .and(&db)
        .and_then(|conn| db::all_todos(conn).map_ok(Json));

    let add_todo = route!(@post /)
        .and(body::json())
        .and(&db)
        .and_then(|new_todo, conn| db::add_todo(conn, new_todo).map_ok(Json).map_ok(Created));

    let patch_todo =
        route!(@patch / u64 /)
            .and(body::json())
            .and(&db)
            .and_then(|id, patch, conn| {
                db::apply_patch(conn, id, patch).map_ok(|todo_opt| todo_opt.map(Json))
            });

    let delete_todo = route!(@delete / u64 /)
        .and(&db)
        .and_then(|id, conn| db::delete_todo(conn, id).map_ok(|_| ()));

    route!(/ "api" / "v1" / "todos").and(routes![
        find_todo,
        list_todos,
        add_todo,
        patch_todo,
        delete_todo,
    ])
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
    use failure::format_err;
    use futures::future::{ready, Future};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, RwLock};

    use finchers::endpoint::{self, IntoEndpoint};
    use finchers::error::{internal_server_error, Error};

    use model::{NewTodo, PatchTodo, Todo};

    #[derive(Debug, Default, Clone)]
    pub struct Database {
        inner: Arc<Inner>,
    }

    #[derive(Debug, Default)]
    struct Inner {
        todos: RwLock<Vec<Todo>>,
        counter: AtomicU64,
    }

    impl IntoEndpoint for &'_ Database {
        type Output = (Conn,);
        type Endpoint = endpoint::Ok<(Conn,)>;

        fn into_endpoint(self) -> Self::Endpoint {
            endpoint::ok((Conn {
                inner: self.inner.clone(),
            },))
        }
    }

    #[derive(Debug, Clone)]
    pub struct Conn {
        inner: Arc<Inner>,
    }

    pub fn find_todo(conn: Conn, id: u64) -> impl Future<Output = Result<Option<Todo>, Error>> {
        ready(
            conn.inner
                .todos
                .read()
                .map_err(|err| internal_server_error(format_err!("{}", err)))
                .and_then(|todos| {
                    let found = todos.iter().find(|todo| todo.id == id).cloned();
                    Ok(found)
                }),
        )
    }

    pub fn all_todos(conn: Conn) -> impl Future<Output = Result<Vec<Todo>, Error>> {
        ready(
            conn.inner
                .todos
                .read()
                .map_err(|err| internal_server_error(format_err!("{}", err)))
                .map(|todos| todos.clone()),
        )
    }

    pub fn add_todo(conn: Conn, new_todo: NewTodo) -> impl Future<Output = Result<Todo, Error>> {
        ready(
            conn.inner
                .todos
                .write()
                .map_err(|err| internal_server_error(format_err!("{}", err)))
                .and_then(|mut todos| {
                    let new_todo = Todo {
                        id: conn.inner.counter.fetch_add(1, Ordering::SeqCst),
                        text: new_todo.text,
                        completed: new_todo.completed,
                    };
                    todos.push(new_todo.clone());
                    Ok(new_todo)
                }),
        )
    }

    pub fn apply_patch(
        conn: Conn,
        id: u64,
        patch: PatchTodo,
    ) -> impl Future<Output = Result<Option<Todo>, Error>> {
        ready(
            conn.inner
                .todos
                .write()
                .map_err(|err| internal_server_error(format_err!("{}", err)))
                .map(|mut todos| {
                    todos.iter_mut().find(|todo| todo.id == id).map(|todo| {
                        if let Some(text) = patch.text {
                            todo.text = text;
                        }
                        if let Some(completed) = patch.completed {
                            todo.completed = completed;
                        }
                        todo.clone()
                    })
                }),
        )
    }

    pub fn delete_todo(conn: Conn, id: u64) -> impl Future<Output = Result<Option<()>, Error>> {
        ready(
            conn.inner
                .todos
                .write()
                .map_err(|err| internal_server_error(format_err!("{}", err)))
                .map(|mut todos| {
                    if let Some(pos) = todos.iter().position(|todo| todo.id == id) {
                        todos.remove(pos);
                        Some(())
                    } else {
                        None
                    }
                }),
        )
    }
}

mod util {
    use finchers::input::Input;
    use finchers::output::Responder;

    use http::{Response, StatusCode};
    use std::mem::PinMut;

    #[derive(Debug)]
    pub struct Created<T>(pub T);

    impl<T: Responder> Responder for Created<T> {
        type Body = T::Body;
        type Error = T::Error;

        fn respond(self, input: PinMut<Input>) -> Result<Response<Self::Body>, Self::Error> {
            let mut response = self.0.respond(input)?;
            *response.status_mut() = StatusCode::CREATED;
            Ok(response)
        }
    }
}
