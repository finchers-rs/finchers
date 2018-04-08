#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate finchers;
#[macro_use]
extern crate serde;

mod application;
mod db;

use self::Response::*;
use self::db::*;
use finchers::prelude::*;
use finchers::runtime::Server;

#[derive(Debug, Serialize, HttpStatus)]
#[serde(untagged)]
enum Response {
    TheTodo(Todo),
    Todos(Vec<Todo>),
    #[status_code = "CREATED"]
    NewTodo(Todo),
    #[status_code = "NO_CONTENT"]
    Deleted,
}

fn main() {
    let app = application::new();

    let endpoint = {
        use finchers::endpoint::ok;
        use finchers::endpoint::prelude::*;
        use finchers::json::Json;

        let find_todo = get(param())
            .try_abort(app.with(|app, id| app.find_todo(id)))
            .map(TheTodo);

        let list_todos = get(ok(())).try_abort(app.with(|app, _| app.list_todos())).map(Todos);

        let add_todo = post(body())
            .try_abort(app.with(|app, Json(new_todo)| app.add_todo(new_todo)))
            .map(NewTodo);

        let patch_todo = patch(param().and(body()))
            .try_abort(app.with(|app, (id, Json(patch))| app.patch_todo(id, patch)))
            .map(TheTodo);

        let delete_todo = delete(param())
            .try_abort(app.with(|app, id| app.delete_todo(id)))
            .map(|_| Deleted);

        path("api/v1/todos")
            .right(choice![find_todo, list_todos, add_todo, patch_todo, delete_todo,])
            .map(Json::from)
    };

    let service = endpoint.into_service();
    Server::new(service).run();
}
