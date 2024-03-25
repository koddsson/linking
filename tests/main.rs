#[cfg(test)]
mod tests {
    use actix_web::{http::header::ContentType, http::StatusCode, test, web, App};
    use linking::services::{create, show, scaffold_database};
    use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;

    fn before_each() -> Pool<SqliteConnectionManager> {
        let manager = SqliteConnectionManager::memory();
        let pool = r2d2::Pool::builder()
            .max_size(1)
            .build(manager)
            .expect("Couldn't open up a sqlite db in memory");

        scaffold_database(pool.clone());

        pool
    }
    
    #[actix_web::test]
    async fn test_show_root_not_found_plaintext() {
        let pool = before_each();
    
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(show),
        )
        .await;
        let req = test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .uri("/")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(
            resp.status() == StatusCode::NOT_FOUND,
            "Expected {} to be a 404 status",
            resp.status()
        );
    }

    #[actix_web::test]
    async fn test_show_slug_not_found_plaintext() {
        let pool = before_each();
    
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(show),
        )
        .await;
        let req = test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .uri("/foo")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(
            resp.status() == StatusCode::NOT_FOUND,
            "Expected {} to be a 404 status",
            resp.status()
        );
    }

    #[actix_web::test]
    async fn test_show_success_plaintext() {
        let pool = before_each();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(create),
        )
        .await;
        let req = test::TestRequest::post()
            .insert_header(ContentType::plaintext())
            .uri("/foo")
            .set_payload("http://koddsson.com")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(
            resp.status().is_success(), 
            "Expected {} to be a successful response code",
            resp.status()
        );

        let req = test::TestRequest::get()
            .insert_header(ContentType::plaintext())
            .uri("/foo")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(
            resp.status().is_success(), 
            "Expected {} to be a successful response code",
            resp.status()
        );
    }
    }
