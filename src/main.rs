use dotenv::dotenv;
use shuttle_runtime::Service;
use std::time::Duration;
use tokio::time;
use tokio_cron_scheduler::{Job, JobScheduler};
use twapi_v2::{
    api::post_2_tweets::{self, Media},
    error::Error,
    oauth10a::OAuthAuthentication,
    upload::{self, check_processing, media_category::MediaCategory},
};

struct MyService {}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for MyService {
    async fn bind(self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| shuttle_runtime::Error::from(anyhow::Error::new(e)))?;

        let job = Job::new_async("40 9 * * FRI", |_uuid, _l| {
            Box::pin(async {
                match run_tweet_job().await {
                    Ok(_) => println!("Tweet job completed successfully"),
                    Err(e) => eprintln!("Tweet job failed: {:?}", e),
                }
            })
        })
        .map_err(|e| shuttle_runtime::Error::from(anyhow::Error::new(e)))?;

        scheduler
            .add(job)
            .await
            .map_err(|e| shuttle_runtime::Error::from(anyhow::Error::new(e)))?;
        scheduler
            .start()
            .await
            .map_err(|e| shuttle_runtime::Error::from(anyhow::Error::new(e)))?;

        // Keep the service running
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    }
}

#[shuttle_runtime::main]
async fn main() -> Result<MyService, shuttle_runtime::Error> {
    let service = MyService {};

    // Bind the service to a random port
    service
        .bind(std::net::SocketAddr::from(([0, 0, 0, 0], 0)))
        .await?;

    Ok(MyService {})
}

async fn run_tweet_job() -> anyhow::Result<()> {
    dotenv().ok();
    let auth = OAuthAuthentication::new(
        std::env::var("API_KEY").unwrap(),
        std::env::var("API_SECRET").unwrap(),
        std::env::var("ACCESS_TOKEN").unwrap(),
        std::env::var("ACCESS_SECRET").unwrap(),
    );
    let (response, _header) = upload::upload_media(
        &std::path::PathBuf::from("perstai.jpeg"),
        "img/jpeg",
        Some(MediaCategory::TweetImage),
        None,
        &auth,
    )
    .await?;
    tracing::info!(response =? response, "upload_media");
    let media_id = response.media_id_string.clone();
    check_processing(
        response,
        &auth,
        Some(|count, _response: &_, _header: &_| {
            if count > 100 {
                Err(Error::Upload("over counst".to_owned()))
            } else {
                Ok(())
            }
        }),
    )
    .await?;
    let body = post_2_tweets::Body {
        text: Some("perstai :D".to_string()),
        media: Some(Media {
            media_ids: vec![media_id],
            ..Default::default()
        }),
        ..Default::default()
    };
    let (response, _header) = post_2_tweets::Api::new(body).execute(&auth).await?;
    tracing::info!(response =? response, "post_2_tweets");
    Ok(())
}
