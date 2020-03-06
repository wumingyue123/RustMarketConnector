pub mod client;
pub mod influxwriter;

fn main() {
    let mut client: client::BitmexClient = client::BitmexClient::new();

    client.start();
}
