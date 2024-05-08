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

#[proxy(
    interface = "org.zbus.MyGreeter",
    default_service = "org.zbus.MyGreeter",
    default_path = "/org/zbus/MyGreeter"
)]
trait GreeterClient {
    async fn greeter_name(&self) -> Result<String>;

    #[zbus(signal)]
    async fn notification(&self, name: &str) -> Result<()>;

    #[zbus(signal)]
    async fn host_metrics(&self, event: HostMetricsNotificationEvent) -> Result<()>;
}

#[tokio::main]
async fn main() -> Result<()> {
    let conn = Connection::session().await?;
    let manager = GreeterClientProxy::new(&conn).await?;

    let mut updated = manager.receive_host_metrics().await?;

    //  we need to check for update in loop

    while let Some(signal) = updated.next().await {
        let args = signal.args()?;

        println!("Matrix {:?}", args.event)
    }
    // No need to specify type of Result each time

    Ok(())
}
