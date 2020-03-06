use influx_db_client;
use influx_db_client::Client;

pub struct InfluxClient
{
    client:Client
}

impl InfluxClient
{
    pub fn new() -> InfluxClient
    {
        InfluxClient
            {
                client: influx_db_client::Client::default()
            }
    }

    pub fn start(&mut self)
    {
        // Connect to influxdb database
        let version = &self.client.get_version().expect("error getting version");
        println!("running influx version {:?}", version);
        println!("connected to influx database: {:?}", &self.client.ping());
    }
}
