#![feature(plugin)]
#![feature(custom_derive)]
#![plugin(rocket_codegen)]

extern crate chrono;
extern crate serde;
extern crate rocket;
extern crate rocket_contrib;
extern crate rusqlite;

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use chrono::Utc;
#[macro_use] extern crate serde_derive;
use rocket::response::Redirect;
use rocket::response::NamedFile;
use rocket::request::Form;
use rocket_contrib::Template;
use rusqlite::Connection;

#[derive(FromForm, Serialize)]
struct Post {
    id: Option<i32>,
    reply_id: Option<i32>,
    name: String,
    title: String,
    content: String,
    created_time: Option<String>,
}

#[derive(Serialize)]
struct PostCollection{
    topic: Post,
    reply: Vec<Post>,
}

#[derive(Serialize)]
struct IndexData{
    title: String,
    announcement: String,
    posts: Vec<PostCollection>,
}

#[get("/")]
fn index() -> Template {
    let database_url = "db/guestbook.db";
    let conn = Connection::open(database_url).unwrap();
    let mut stmt = conn.prepare("SELECT id, reply_id, name, title, content, created_time FROM post WHERE reply_id IS NULL ORDER BY id DESC").unwrap();
    let post_iter = stmt.query_map(&[], |row| {
        Post {
                    id: row.get(0),
              reply_id: row.get(1),
                  name: row.get(2),
                 title: row.get(3),
               content: row.get(4),
          created_time: row.get(5),
        }
    }).unwrap();
    
    let mut reply_stmt = conn.prepare("SELECT id, reply_id, name, title, content, created_time FROM post WHERE reply_id = :id").unwrap();
    let posts = post_iter.map(|post| post.unwrap()).map(|post| {
        let reply_iter = reply_stmt.query_map_named(&[(":id", &post.id)], |row| {
                            Post {
                                        id: row.get(0),
                                  reply_id: row.get(1),
                                      name: row.get(2),
                                     title: row.get(3),
                                   content: row.get(4),
                              created_time: row.get(5),
                            }
                         }).unwrap();
        let mut post_with_time = post;
        post_with_time.created_time = Some(post_with_time.created_time.unwrap().split('.').nth(0).unwrap().to_string());
        PostCollection {
            topic: post_with_time,
            reply: reply_iter.map(|reply| {
                let mut reply_with_time = reply.unwrap();
                reply_with_time.created_time = Some(reply_with_time.created_time.unwrap().split('.').nth(0).unwrap().to_string());
                reply_with_time
            }).collect(),
        }
    }).collect();

    let context = IndexData {
        title: "Rust 留言板".to_string(),
        announcement: "歡迎來到我的留言板。".to_string(),
        posts: posts,
    };

    Template::render("index", context)
}

#[get("/topic_form")]
fn topic_form() -> Template {
    let mut context = HashMap::new();
    context.insert("title", "新增留言");
    Template::render("post_form", context)
}

#[get("/reply_form/<reply_id>")]
fn reply_form(reply_id: String) -> Template {
    let mut context = HashMap::new();
    context.insert("title", "回覆留言".to_string());
    context.insert("reply_id", reply_id);
    Template::render("post_form", context)
}


#[post("/post", data="<post>")]
fn create_post(post: Form<Post>) -> Redirect {
    let database_url = "db/guestbook.db";
    let post_data = post.get();
    let conn = Connection::open(database_url).unwrap();
    conn.execute("INSERT INTO post (reply_id, name, title, content, created_time) VALUES (?1, ?2, ?3, ?4, ?5)",
                 &[&post_data.reply_id, &post_data.name, &post_data.title, &post_data.content, &Utc::now().naive_utc().to_string()]).unwrap();
    Redirect::to("/")
}

#[get("/static/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}

fn main() {
    rocket::ignite()
      .mount("/", routes![index, topic_form, reply_form, create_post, files])
      .attach(Template::fairing())
      .launch();
}
