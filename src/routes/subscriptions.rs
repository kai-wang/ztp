use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;
use tracing::Instrument;

#[derive(serde::Deserialize)]
pub struct FormData {
  email: String,
  name: String
}

pub async fn subscribe(
  form: web::Form<FormData>,
  pool: web::Data<PgPool>
) -> HttpResponse {

  // Generate a random uuid
  let request_id = Uuid::new_v4();

  // Spans, like logs, have an associated level
  // `info_span` creates a span at the info-level
  let request_span = tracing::info_span!(
    "Adding a new sbuscriber.",
    %request_id,
    subscriber_email = %form.email,
    subscriber_name = %form.name
  );

  let _request_span_guard = request_span.enter();

  // We do ot call `enter` on query_span!
  // `.instrument` taks care of it at the right moments
  let query_span = tracing::info_span!(
    "Saving new subscriber details in the database"
  );

  match sqlx::query!(
    r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
    Uuid::new_v4(),
    form.email,
    form.name,
    Utc::now()
  )
  .execute(pool.get_ref())
  .instrument(query_span)
  .await {
    Ok(_) => {
      HttpResponse::Ok().finish()
    },
    Err(e) => {
      tracing::error!("Failed to execute query: {:?}", e);
      HttpResponse::InternalServerError().finish()
    }
  }
}