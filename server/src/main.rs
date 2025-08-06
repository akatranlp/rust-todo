use actix_web::error::ErrorInternalServerError;
use actix_web::patch;
use actix_web::{
    App, HttpResponse, HttpServer, Responder, delete, error::ErrorNotFound, get, post, web,
};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Todo {
    id: Uuid,
    title: String,
    done: bool,
    description: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct UpdateTodoRequest {
    title: Option<String>,
    done: Option<bool>,
    description: Option<String>,
}

impl Todo {
    fn new(title: String, description: String) -> Self {
        return Todo {
            id: Uuid::now_v7(),
            title,
            done: false,
            description,
        };
    }
}

impl From<CreateTodoRequest> for Todo {
    fn from(value: CreateTodoRequest) -> Self {
        Self::new(value.title, value.description)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct CreateTodoRequest {
    title: String,
    description: String,
}

struct AppData {
    app_name: String,
    todos: Mutex<HashMap<uuid::Uuid, Todo>>,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            app_name: String::from("Streamlet Routing Server"),
            todos: Mutex::new(HashMap::new()),
        }
    }
}

#[get("/")]
async fn hello(data: web::Data<AppData>) -> String {
    let app_name = &data.app_name;
    format!("Hello {app_name}!")
}

#[post("/todos")]
async fn create_todo(
    todo: web::Json<CreateTodoRequest>,
    data: web::Data<AppData>,
) -> impl Responder {
    let mut todos = data.todos.lock().unwrap();
    let todo: Todo = todo.into_inner().into();
    let id = todo.id;
    todos.insert(id, todo);
    HttpResponse::Created()
        .insert_header(("Location", format!("/todos/{}", id)))
        .finish()
}

#[get("/todos")]
async fn list_todos(data: web::Data<AppData>) -> impl Responder {
    let todos = data.todos.lock().unwrap();
    let todos = todos.values().collect::<Vec<&Todo>>();
    HttpResponse::Ok().json(todos)
}

#[get("/todos/{id}")]
async fn get_todo(
    id: web::Path<Uuid>,
    data: web::Data<AppData>,
) -> actix_web::Result<impl Responder, actix_web::Error> {
    let todos = data.todos.lock().unwrap();
    let todo = todos
        .get(&id)
        .ok_or("todo not found")
        .map_err(|err| ErrorNotFound(err))?;
    Ok(HttpResponse::Ok().json(todo))
}

#[delete("/todos/{id}")]
async fn delete_todo(
    id: web::Path<Uuid>,
    data: web::Data<AppData>,
) -> actix_web::Result<impl Responder, actix_web::Error> {
    let mut todos = data.todos.lock().unwrap();
    todos
        .get(&id)
        .ok_or("todo not found")
        .map_err(|err| ErrorNotFound(err))?;
    let todo = todos.remove(&id);
    Ok(HttpResponse::Ok().json(todo))
}

#[patch("/todos/{id}")]
async fn patch_todo(
    id: web::Path<Uuid>,
    patch_todo: web::Json<UpdateTodoRequest>,
    data: web::Data<AppData>,
) -> actix_web::Result<impl Responder, actix_web::Error> {
    let mut todos = data.todos.lock().unwrap();
    let todo = todos
        .get_mut(&id)
        .ok_or("todo not found")
        .map_err(|err| ErrorNotFound(err))?;
    let patch_todo = patch_todo.into_inner();

    if let Some(title) = patch_todo.title {
        todo.title = title;
    }
    if let Some(description) = patch_todo.description {
        todo.description = description;
    }
    if let Some(done) = patch_todo.done {
        todo.done = done;
    }

    Ok(HttpResponse::Ok().json(todo))
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(format!("Echo: {}", req_body))
}

#[post("/solve")]
async fn solve() -> Result<impl Responder, actix_web::Error> {
    let data = routing::solve().map_err(|err| ErrorInternalServerError(err))?;
    Ok(HttpResponse::Ok().body(data))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Listening on port 8000....");

    let app_data = web::Data::new(AppData::default());

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(hello)
            .service(list_todos)
            .service(create_todo)
            .service(get_todo)
            .service(patch_todo)
            .service(delete_todo)
            .service(solve)
            .service(echo)
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
