// TODO: Put a .map_err(...) to a HTTPResponse for every `unwrap()` call?
pub mod services {
    use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
    use log::info;
    use r2d2_sqlite::SqliteConnectionManager;
    use serde::{Deserialize, Serialize};
    use std::env;
    use std::str;

    // TODO: Implement format and/or display trait
    #[derive(Deserialize, Serialize)]
    struct LinkPayload {
        slug: String,
        url: String,
    }

    #[get("/{slug}")]
    async fn show(
        req: HttpRequest,
        pool: web::Data<r2d2::Pool<SqliteConnectionManager>>,
    ) -> impl Responder {
        let slug = req.match_info().get("slug").unwrap();
        let conn = pool.get().unwrap();

        // TODO: Use a ORM like Desiel??
        // TODO: Add a test for SQL injection?
        let mut stmt = conn
            .prepare("SELECT slug, url FROM urls WHERE slug=:slug LIMIT 1;")
            .unwrap();

        let mut url_iter = stmt
            .query_map(&[(":slug", slug)], |row| {
                Ok(LinkPayload {
                    slug: row.get(0)?,
                    url: row.get(1)?,
                })
            })
            .unwrap();

        match url_iter.next() {
            Some(results) => match results {
                Ok(payload) => HttpResponse::PermanentRedirect()
                    .insert_header(("Location", payload.url.to_string()))
                    .body(payload.url),
                Err(error) => {
                    println!("Error occured: {}", error);
                    HttpResponse::InternalServerError().body("Internal Server Error!")
                }
            },
            None => HttpResponse::NotFound().body("Not found"),
        }
    }

    #[post("/{slug}")]
    async fn create(
        pool: web::Data<r2d2::Pool<SqliteConnectionManager>>,
        req: HttpRequest,
        bytes: web::Bytes,
    ) -> impl Responder {
        // TODO: We should probably just read the key once on start up and keep in memory _but_ I
        // can't be asked in figuring out how `app_data` works right now.
        let secret_key = env::var("SECRET_KEY").unwrap();

        // Maybe this can be all summed up into a `authentication_required` macro.
        let auth_header = match req.headers().get("Authentication") {
            Some(header) => header.to_str().unwrap(),
            None => return HttpResponse::Forbidden().body("Forbidden"),
        };

        if auth_header.trim() != secret_key {
            return HttpResponse::Unauthorized().body("Not authorized");
        }

        let slug = req.match_info().get("slug").unwrap();
        let url = str::from_utf8(&bytes)
            .map_err(|_| HttpResponse::BadRequest().finish())
            .unwrap();
        let conn = pool.get().unwrap();

        let results = conn.execute("INSERT INTO urls (slug, url) VALUES (?1, ?2)", (slug, url));

        match results {
            Ok(_) => HttpResponse::Ok().body(format!("Created a redirect to {} for {}", url, slug)),
            Err(error) => match error {
                rusqlite::Error::SqliteFailure(error, _) => match error.code {
                    rusqlite::ErrorCode::ConstraintViolation => HttpResponse::Conflict()
                        .body(format!("There's already a link for {}", slug)),
                    _ => HttpResponse::InternalServerError().body("Error"),
                },
                _ => HttpResponse::InternalServerError().body("Error"),
            },
        }
    }

    pub fn scaffold_database(pool: r2d2::Pool<SqliteConnectionManager>) {
        let conn = pool.get().unwrap();
        let _ = conn.execute(
            "create table if not exists urls (
                 id integer primary key,
                 slug text not null unique,
                 url text not null
             )",
            (),
        );
    }
}
