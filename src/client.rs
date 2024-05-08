use anyhow::Result;
use zbus::{
    proxy,
    zvariant::{DeserializeDict, SerializeDict, Type},
    Connection,
};

use tokio_stream::StreamExt;

#[derive(DeserializeDict, SerializeDict, Debug, Clone, PartialEq, Type)]
// `Type` treats `BatteryInfoResponse` is an alias for `a{sv}`.
#[zvariant(signature = "a{sv}")]
pub struct HostMetricsNotificationEvent {
    pub cpu_usage: f32,
    pub total_memory: u64,
    pub available_memory: u64,
}

#[derive(DeserializeDict, SerializeDict, Type, Debug, Clone, PartialEq)]
// `Type` treats `NotificationEvent` is an alias for `a{sv}`.
#[zvariant(signature = "a{sv}")]
pub struct PowerNotificationEvent {
    pub status: String,
    pub percentage: f32,
}

#[derive(DeserializeDict, SerializeDict, Type, Debug, Clone)]
/// A wireless notification event.
#[zvariant(signature = "a{sv}")]
pub struct WirelessNotificationEvent {
    pub signal_strength: String,
    pub is_connected: bool,
    pub is_enabled: bool,
    pub frequency: String,
    pub ssid: String,
}

#[derive(DeserializeDict, SerializeDict, Type, Debug)]
// `Type` treats `BluetoothNotificationEvent` is an alias for `a{sv}`.
#[zvariant(signature = "a{sv}")]
pub struct BluetoothNotificationEvent {
    pub is_connected: bool,
    pub is_enabled: bool,
}

#[proxy(
    interface = "org.mechanix.services.Power",
    default_service = "org.mechanix.services.Power",
    default_path = "/org/mechanix/services/Power"
)]
trait GreeterClient {
    async fn greeter_name(&self) -> Result<String>;

    // #[zbus(signal)]
    // async fn notification(&self, name: &str) -> Result<()>;

    #[zbus(signal)]
    async fn notification(&self, event: PowerNotificationEvent) -> Result<()>;
}

#[tokio::main]
async fn main() -> Result<()> {
    let conn = Connection::system().await?;
    let manager = GreeterClientProxy::new(&conn).await?;

    let mut updated = manager.receive_notification().await?;

    //  we need to check for update in loop

    while let Some(signal) = updated.next().await {
        let args = signal.args()?;

        println!("Bluetooth {:?}", args.event)
    }
    // No need to specify type of Result each time

    Ok(())
}
