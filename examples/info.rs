use std::error::Error;
use std::time::Duration;
use tracing::info;

use rmreco::tokio_client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let args: Vec<_> = std::env::args().collect();

    let (r, mut w) = tokio_client::connect(&args[1])?;
    let mut watch = r.watch_radar().await;
    let status = watch.get_game_robot_status().await;
    let send_id = status.robot_id as u16;
    info!("{:#?}", status);

    let mut itv = tokio::time::interval(Duration::from_millis(500));
    loop {
        for i in 0..50 {
            itv.tick().await;
            let data = vec![0x00, i];
            w.send_p2p(0x0200, send_id, 106, data.clone()).await.unwrap();
            info!("sent {:?}", &data);
            itv.tick().await;
            w.send_minimap_receipt((1 + (i % 7)) as u16, ((i % 28) as f32, (i % 15) as f32)).await.unwrap();
        }
    }
}
