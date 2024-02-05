pub mod services {
    use actix_web::{error, get, post, web, HttpRequest, HttpResponse, Responder};
    use serde::{Deserialize, Serialize};

    extern crate rusqlite;

    use r2d2_sqlite::SqliteConnectionManager;

    #[derive(Deserialize, Serialize)]
    struct LinkPayload {
        slug: String,
        url: String,
    }

    #[get("/{slug}")]
    async fn show(
        pool: web::Data<r2d2::Pool<SqliteConnectionManager>>,
        req: HttpRequest,
    ) -> impl Responder {
        let slug: String = req.match_info().get("slug").unwrap().parse().unwrap();
        let conn = pool.get().map_err(error::ErrorInternalServerError).unwrap();

        let mut stmt = conn
            .prepare("SELECT slug, url FROM urls WHERE slug=:slug LIMIT 1;")
            .map_err(error::ErrorInternalServerError)
            .unwrap();
        let mut url_iter = stmt
            .query_map(&[(":slug", slug.to_string().as_str())], |row| {
                Ok(LinkPayload {
                    slug: row.get(0)?,
                    url: row.get(1)?,
                })
            })
            .map_err(error::ErrorInternalServerError)
            .unwrap();

        match url_iter.next() {
            Some(results) => match results {
                Ok(url) => HttpResponse::PermanentRedirect()
                    .insert_header(("Location", url.url.to_string()))
                    .json(url),
                Err(error) => HttpResponse::InternalServerError().json(error.to_string()),
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
        let slug: String = req.match_info().get("slug").unwrap().parse().unwrap();
        let url = String::from_utf8(bytes.to_vec())
            .map_err(|_| HttpResponse::BadRequest().finish())
            .unwrap();
        let conn = pool.get().unwrap();

        let results = conn.execute(
            "INSERT INTO urls (slug, url) VALUES (?1, ?2)",
            // TODO: I don't know why I gotta clone these here.
            (slug.clone(), url.clone()),
        );

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
