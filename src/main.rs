use btleplug::api::{
    Central, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use std::error::Error;
use std::time::Duration;
use uuid::{uuid, Uuid};
use futures::stream::StreamExt;

const SERVICE_UUID: Uuid         = uuid!("45611d13-4fc8-4e04-a88e-02bb24054e22");
const CHARACTERISTIC_UUID: Uuid  = uuid!("04aaf1b3-5724-4cda-8f61-775585393a46");

/*
#define    ESP_GATT_HEART_RATE_MEAS                 0x2A37 // hr measurement
#define    ESP_GATT_BODY_SENSOR_LOCATION            0x2A38
#define    ESP_GATT_HEART_RATE_CNTL_POINT           0x2A39

Characteristic { uuid: 00002a05-0000-1000-8000-00805f9b34fb, service_uuid: 00001801-0000-1000-8000-00805f9b34fb, properties: INDICATE }
Characteristic { uuid: 00002a37-0000-1000-8000-00805f9b34fb, service_uuid: 0000180d-0000-1000-8000-00805f9b34fb, properties: NOTIFY }
Characteristic { uuid: 00002a38-0000-1000-8000-00805f9b34fb, service_uuid: 0000180d-0000-1000-8000-00805f9b34fb, properties: READ }
Characteristic { uuid: 00002a39-0000-1000-8000-00805f9b34fb, service_uuid: 0000180d-0000-1000-8000-00805f9b34fb, properties: READ | WRITE }
*/


use tokio::time;

async fn find_light(central: &Adapter) -> Option<Peripheral> {
    for p in central.peripherals().await.unwrap() {
        if p.properties()
            .await
            .unwrap()
            .unwrap()
            .local_name
            .iter()
            .any(|name| name.contains("InkVT2"))
        {
            return Some(p);
        }
    }
    for p in central.peripherals().await.unwrap() {
        println!("{:?}", p.properties().await.unwrap().unwrap().local_name);
    }
    None
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await.unwrap();

    // get the first bluetooth adapter
    let central = manager
        .adapters()
        .await
        .expect("Unable to fetch adapter list.")
        .into_iter()
        .nth(0)
        .expect("Unable to find adapters.");

    // start scanning for devices
    central.start_scan(ScanFilter::default()).await?;
    // instead of waiting, you can use central.events() to get a stream which will
    // notify you of new devices, for an example of that see examples/event_driven_discovery.rs
    time::sleep(Duration::from_secs(2)).await;

    // find the device we're interested in
    let light = find_light(&central).await.expect("No lights found");

    light.connect().await?;
    light.discover_services().await?;
    let chars = light.characteristics();
    for c in chars {
        println!("{:?}", c);
    }

    let chars = light.characteristics();
    let cmd_char = chars
        .iter()
        .find(|c| c.uuid == CHARACTERISTIC_UUID)
        .expect("Unable to find characterics");

    //println!("$$ {:?} $$", light.read(&cmd_char).await);
    light.subscribe(&cmd_char).await.unwrap();


    let mut notification_stream =
        light.notifications().await?;
    while let Some(data) = notification_stream.next().await {
        println!(
            "Received data from [{:?}]: {:?}",
            data.uuid, data.value
        );
    }


    /* dance party
    let color_cmd = vec![0x56, 0x00, 0xF0, 0xAA];
    light
        .write(&cmd_char, &color_cmd, WriteType::WithoutResponse)
        .await?;
        */
    Ok(())
}
