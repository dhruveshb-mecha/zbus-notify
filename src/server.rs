use anyhow::Result;
use tokio::{
    signal,
    time::{self, Duration},
};
use zbus::{
    connection::Builder,
    interface,
    zvariant::{DeserializeDict, SerializeDict, Type},
    SignalContext,
};

#[derive(DeserializeDict, SerializeDict, Type, Debug, Clone, Copy)]
struct Greeter;

#[derive(DeserializeDict, SerializeDict, Type, Debug)]
// `Type` treats `HostMetricsNotificationEvents` is an alias for `a{sv}`.
#[zvariant(signature = "a{sv}")]
pub struct HostMetricsNotificationEvent {
    pub cpu_usage: f32,
    pub total_memory: u64,
    pub available_memory: u64,
}

#[interface(name = "org.zbus.MyGreeter")]
impl Greeter {
    async fn say_hello(&self, name: &str) -> String {
        println!("Received request from: {}", name);
        format!("Hello {}!", name)
    }

    #[zbus(signal)]
    async fn host_metrics(
        &self,
        ctxt: &SignalContext<'_>,
        event: HostMetricsNotificationEvent,
    ) -> Result<(), zbus::Error>;
}

async fn handle_tasks(greeter: &Greeter, conn: &zbus::Connection) -> Result<()> {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_cpu();
    let cpu_usage = sys.global_cpu_info().cpu_usage();
    sys.refresh_memory();
    let total_memory = sys.total_memory();
    let available_memory = sys.available_memory();
    let hots_matrix = HostMetricsNotificationEvent {
        cpu_usage,
        total_memory,
        available_memory,
    };

    // print host metrics
    println!("Host Matrix {:?}", hots_matrix);

    let ctxt = SignalContext::new(&conn, "/org/zbus/MyGreeter")?;
    greeter.host_metrics(&ctxt, hots_matrix).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Server is running");
    let greeter = Greeter;

    println!("creating connection");
    let conn = Builder::session()?
        .name("org.zbus.MyGreeter")?
        .serve_at("/org/zbus/MyGreeter", greeter)?
        .build()
        .await?;

    println!("Connection created");

    let mut interval = time::interval(Duration::from_secs(15));

    // Send a notification every 15 seconds
    loop {
        interval.tick().await;
        handle_tasks(&greeter, &conn).await?;
    }
}
