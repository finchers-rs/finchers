use finchers::Json;
use finchers::endpoint::ok;
use finchers::endpoint::prelude::*;

use app::Application;
use db::*;

use self::Response::*;

#[derive(Debug, Serialize, HttpStatus)]
#[serde(untagged)]
pub enum Response {
    TheTodo(Todo),
    Todos(Vec<Todo>),
    #[status_code = "CREATED"]
    NewTodo(Todo),
    #[status_code = "NO_CONTENT"]
    Deleted,
}

pub fn build_endpoint(app: &Application) -> impl Endpoint<Item = Json<Response>> + Send + Sync + 'static {
    let find_todo = get(param())
        .then(app.with(|app, id| app.find_todo(id)))
        .map(TheTodo);

    let list_todos = get(ok(()))
        .then(app.with(|app, _| app.list_todos()))
        .map(Todos);

    let add_todo = post(data())
        .then(app.with(|app, Json(new_todo)| app.add_todo(new_todo)))
        .map(NewTodo);

    let patch_todo = patch(param().and(data()))
        .then(app.with(|app, (id, Json(patch))| app.patch_todo(id, patch)))
        .map(TheTodo);

    let delete_todo = delete(param())
        .then(app.with(|app, id| app.delete_todo(id)))
        .map(|_| Deleted);

    path("api/v1/todos")
        .right(choice![find_todo, list_todos, add_todo, patch_todo, delete_todo,])
        .map(Json::from)
}
