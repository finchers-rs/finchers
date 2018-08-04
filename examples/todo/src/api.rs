use finchers::endpoint::just;
use finchers::endpoint::prelude::*;
use finchers::Json;

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
    let todo_id = param().unwrap_ok();
    let new_todo = body().unwrap_ok().map(Json::into_inner);
    let patch_todo = body().unwrap_ok().map(Json::into_inner);

    let find_todo = get(todo_id)
        .map_async(app.with(|app, id| app.find_todo(id).map(TheTodo)))
        .unwrap_ok();

    let list_todos = get(just(()))
        .map_async(app.with(|app, _| app.list_todos().map(Todos)))
        .unwrap_ok();

    let add_todo = post(new_todo)
        .map_async(app.with(|app, new_todo| app.add_todo(new_todo).map(NewTodo)))
        .unwrap_ok();

    let patch_todo = patch(todo_id.and(patch_todo))
        .map_async(app.with(|app, (id, patch)| app.patch_todo(id, patch).map(TheTodo)))
        .unwrap_ok();

    let delete_todo = delete(todo_id)
        .map_async(app.with(|app, id| app.delete_todo(id).and(Ok(Deleted))))
        .unwrap_ok();

    path("api/v1/todos")
        .right(choice![
            find_todo,
            list_todos,
            add_todo,
            patch_todo,
            delete_todo,
        ])
        .map(Json::from)
}
