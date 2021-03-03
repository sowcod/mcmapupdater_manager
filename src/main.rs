use tokio;
use shiplift::Docker;

mod manager;
mod config;

type CommonResult<T> = Result<T, Box<dyn std::error::Error>>;

async fn read_config(filename: &str) -> CommonResult<config::Config> {
    let file = std::fs::File::open(filename)?;
    let reader = std::io::BufReader::new(file);
    Ok(config::read(reader).await.unwrap())
}

/*
async fn test() -> CommonResult<()> {
    let client = reqwest::Client::new();
    let res1 = client.get("https://www.google.com/")
        //.header("Metadata-Flavor", "Google")
        .send()
        .await
        .or_else(|e| Err(e))?
        .text()
        .await
        .or_else(|e| Err(e))?;
    println!("{}", res1);
    Ok(())
}
*/

#[tokio::main]
async fn main() {
    /*
    match test().await {
        Err(e) => println!("test error {:?}", e),
        _ => (),
    }
    */
    println!("read config.yaml");
    let config = read_config("./config.yaml").await.unwrap();

    let docker = Docker::default();
    let manager = manager::Manager::new(config, &docker);
    manager.run_genmap_all().await.unwrap();
}