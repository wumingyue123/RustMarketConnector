use std::sync::{Arc, Barrier};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::net::SocketAddr;
use paho_mqtt as mqtt;
use std::process;
use std::time::Duration;
use std::vec::Vec;
use paho_mqtt::Message;
use serde_json::{Result, Value};
use serde::{Deserialize, Serialize};
use super::influxwriter;
use std::collections::HashMap;
use influx_db_client::{Point, Points};
use influx_db_client as influx;
use crate::influxwriter::InfluxClient;

#[derive(Serialize, Deserialize, Debug)]
struct BitmexDataObject
{
    pub action: String,
    pub data: Vec<Box<BitmexTradeData>>,
    pub table: String,

}

#[derive(Serialize, Deserialize, Debug)]
enum BitmexDataOptions
{
    TradeData,
    QuoteData
}

#[derive(Serialize, Deserialize, Debug)]
struct BitmexTradeData
{
    pub size: f32,
    pub price: f32,
    pub symbol: String,
    pub foreignNotional: f32,
    pub grossValue: f32,
    pub homeNotional: f32,
    pub tickDirection: String,
    pub timestamp: String,
    pub trdMatchID: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct QuoteData
{
    pub foreign_notional: f32,
    pub gross_value: f32,
    pub home_notional: f32,
    pub tick_direction: String,
    pub timestamp: String,
    pub trd_match_id: String,
}

fn on_connect_success(cli: &mqtt::AsyncClient, _msgid: u16) {
    println!("Connection succeeded");
    let TOPICS = vec!["marketdata/deribitdata", "marketdata/bitmexdata"];
    // Subscribe to the desired topic(s).
    cli.subscribe_many(&TOPICS, &vec![0;TOPICS.len()]);
    println!("Subscribing to topics: {:?}", TOPICS);
    
    // TODO: This doesn't yet handle a failed subscription.
}

// Callback for a failed attempt to connect to the server.
// We simply sleep and then try again.
//
// Note that normally we don't want to do a blocking operation or sleep
// from  within a callback. But in this case, we know that the client is
// *not* conected, and thus not doing anything important. So we don't worry
// too much about stopping its callback thread.
fn on_connect_failure(cli: &mqtt::AsyncClient, _msgid: u16, rc: i32) {
    println!("Connection attempt failed with error code {}.\n", rc);
    thread::sleep(Duration::from_millis(2500));
    cli.reconnect_with_callbacks(on_connect_success, on_connect_failure);
}


pub struct BitmexClient
{
}

impl BitmexClient
{
    pub fn new() -> BitmexClient
    {
        BitmexClient{}
    }

    pub fn start(&mut self)
    {
        // Create a client and define connect options
        let create_options = mqtt::CreateOptionsBuilder::new()
        .server_uri("localhost:13000")
        .client_id("rust_consumer")
        .finalize();

        let mut cli = mqtt::AsyncClient::new(create_options)
        .unwrap_or_else(|err| 
        {
            println!("error creating client: {:?}", err);
            process::exit(1)
        });

        cli.set_connection_lost_callback(|cli: &mqtt::AsyncClient|
        {
            println!("Connection lost with mqtt server. Attempting to reconnect...", );
            thread::sleep(Duration::from_secs(2));
            cli.reconnect_with_callbacks(on_connect_success, on_connect_failure);
        });

        cli.set_message_callback(|_cli, msg|
        {
            BitmexClient::message_callback(msg);
        });

        let last_will = mqtt::Message::new("last_will", "rust subscriber lost connection", 1);

        let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .clean_session(false)
        .connect_timeout(Duration::from_secs(20))
        .will_message(last_will)
        .finalize();

        println!("Connecting to mqtt server", );
        cli.connect_with_callbacks(conn_opts, on_connect_success, on_connect_failure);

        loop{thread::sleep(Duration::from_secs(10))};

    }

    fn message_callback(msg: Option<Message>) -> Result<()>{
        if let Some(msg) = msg
        {
            let topic = msg.topic();
            let payload_str = msg.payload_str();
            if topic == "marketdata/bitmexdata"
            {
                let trade_data: BitmexDataObject = serde_json::from_str(&payload_str)?;
                let data_array = &trade_data.data;
                for _data in data_array.iter()
                    {
                        println!("{:?}", _data);
                        // pass to worker thread


                    }



            }
            else {
                //println!("{} - {}", topic, payload_str);
            }

        }
        Ok(())
    }
}
