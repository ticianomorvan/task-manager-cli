mod database;

use dotenv::dotenv;
use std::env;

fn main() {
    dotenv().ok();

    println!("Running database from URL: {:#?}", env::var("DATABASE_URL"));
}
