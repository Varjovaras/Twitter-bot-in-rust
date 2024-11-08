use dotenv::dotenv;
use tokio_cron_scheduler::{Job, JobScheduler};
use twapi_v2::{
    api::post_2_tweets::{self, Media},
    error::Error,
    oauth10a::OAuthAuthentication,
    upload::{self, check_processing, media_category::MediaCategory},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let scheduler = JobScheduler::new().await?;
    println!("??");
    dbg!(chrono::Utc::now());

    // of any day in March and June that is a Friday of the year 2017.
    let tweet_job = Job::new_async("0 45 06 * * Fri *", |_uuid, _l| {
        println!(":D");
        Box::pin(async {
            match run_tweet_job().await {
                Ok(_) => println!(
                    "Tweet job completed successfully, at {:?}",
                    chrono::Utc::now()
                ),
                Err(e) => eprintln!("Tweet job failed: {:?}", e),
            }
        })
    })?;

    let time_job = Job::new_async("0 * * * * * *", |_uuid, _l| {
        Box::pin(async {
            println!("{:?}", chrono::Utc::now());
        })
    })?;
    // Add the job to the scheduler
    scheduler.add(tweet_job).await?;
    scheduler.add(time_job).await?;

    // Start the scheduler
    scheduler.start().await?;

    // Keep the main thread running
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(200)).await;
    }
}

async fn run_tweet_job() -> anyhow::Result<()> {
    dotenv().ok();
    println!(":DD");

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
