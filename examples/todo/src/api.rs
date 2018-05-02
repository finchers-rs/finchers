use finchers::Json;
use finchers::endpoint::just;
use finchers::endpoint::prelude::*;

use app::Application;
use db::*;

use self::Response::*;

#[derive(Debug, Serialize, HttpResponse)]
#[serde(untagged)]
pub enum Response {
    TheTodo(Todo),
    Todos(Vec<Todo>),
    #[status_code = "CREATED"]
    NewTodo(Todo),
    #[status_code = "NO_CONTENT"]
    Deleted,
}

pub fn build_endpoint(app: &Application) -> impl Endpoint<Output = Json<Response>> + 'static {
    let find_todo = get(param())
        .map_async(app.with(|app, id| app.find_todo(id)))
        .unwrap_ok()
        .map(TheTodo);

    let list_todos = get(just(()))
        .map_async(app.with(|app, _| app.list_todos()))
        .unwrap_ok()
        .map(Todos);

    let add_todo = post(data())
        .map_async(app.with(|app, Json(new_todo)| app.add_todo(new_todo)))
        .unwrap_ok()
        .map(NewTodo);

    let patch_todo = patch(param().and(data()))
        .map_async(app.with(|app, (id, Json(patch))| app.patch_todo(id, patch)))
        .unwrap_ok()
        .map(TheTodo);

    let delete_todo = delete(param())
        .map_async(app.with(|app, id| app.delete_todo(id)))
        .unwrap_ok()
        .map(|_| Deleted);

    path("api/v1/todos")
        .right(choice![find_todo, list_todos, add_todo, patch_todo, delete_todo,])
        .map(Json::from)
}
