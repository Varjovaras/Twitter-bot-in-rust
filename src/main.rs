use apalis::prelude::{Job, JobContext};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct Reminder(DateTime<Utc>);

impl From<DateTime<Utc>> for Reminder {
    fn from(t: DateTime<Utc>) -> Self {
        Reminder(t)
    }
}

// set up an identifier for apalis
impl Job for Reminder {
    const NAME: &'static str = "reminder::DailyReminder";
}

#[derive(Clone)]
struct CronjobData {
    message: String,
}

impl CronjobData {
    fn execute(&self, item: Reminder) {
        println!("{} from CronjobData::execute()!", &self.message);
    }
}

async fn say_hello_world(job: Reminder, ctx: JobContext) {
    println!("Hello world from send_reminder()!");
    // this lets you use variables stored in the CronjobData struct
    let svc = ctx.data_opt::<CronjobData>().unwrap();
    // this executes CronjobData::execute()
    svc.execute(job);
}

//https://www.shuttle.rs/blog/2024/01/24/writing-cronjobs-rust
//https://github.com/geofmureithi/apalis/blob/master/examples/async-std-runtime/src/main.rs
