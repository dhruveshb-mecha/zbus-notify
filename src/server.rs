use anyhow::Result;
use tokio::{
    signal,
    task::JoinHandle,
    time::{self, Duration},
};
use zbus::{
    blocking::connection,
    connection::Builder,
    interface,
    zvariant::{DeserializeDict, SerializeDict, Type},
    Connection, SignalContext,
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

    // #[zbus(signal)]
    // async fn notification(
    //     &self,
    //     ctxt: &SignalContext<'_>,
    //     event: String,
    // ) -> Result<(), zbus::Error>;

    // pub async fn send_notification_stream(&self) -> Result<(), zbus::fdo::Error> {
    //     let ctxt = SignalContext::new(&Connection::session().await?, "org/zbus/MyGreeter1")?;
    //     self.notification(&ctxt, "Hello from the server!".to_string())
    //         .await?;
    //     println!("Notification sent!");
    //     Ok(())
    // }

    // async fn get_host_metrics(&self) -> Result<(), zbus::fdo::Error> {
    //     let mut sys = sysinfo::System::new_all();
    //     sys.refresh_cpu();

    //     let cpu_usage = sys.global_cpu_info().cpu_usage();

    //     sys.refresh_memory();
    //     let total_memory = sys.total_memory();
    //     let available_memory = sys.available_memory();

    //     let hots_matrix = HostMetricsNotificationEvent {
    //         cpu_usage: cpu_usage,
    //         total_memory: total_memory,
    //         available_memory: available_memory,
    //     };

    //     // println! host_metrics;
    //     println!("Host Matrix: {:?}", hots_matrix);

    //     Ok(())
    // }

    #[zbus(signal)]
    async fn host_metrics(
        &self,
        ctxt: &SignalContext<'_>,
        event: HostMetricsNotificationEvent,
    ) -> Result<(), zbus::Error>;

    pub async fn get_host_metrics(&self) -> Result<(), zbus::fdo::Error> {
        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;
            let mut sys = sysinfo::System::new_all();
            sys.refresh_cpu();

            let cpu_usage = sys.global_cpu_info().cpu_usage();

            sys.refresh_memory();
            let total_memory = sys.total_memory();
            let available_memory = sys.available_memory();

            let hots_matrix = HostMetricsNotificationEvent {
                cpu_usage: cpu_usage,
                total_memory: total_memory,
                available_memory: available_memory,
            };

            //  print host matrics
            println!("Host Matrix {:?}", hots_matrix);
            let ctxt = SignalContext::new(&Connection::session().await?, "org/zbus/MyGreeter")?;
            self.host_metrics(&ctxt, hots_matrix).await?;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let greeter = Greeter;

    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    let conn = Builder::session()?
        .name("org.zbus.MyGreeter")?
        .serve_at("/org/zbus/MyGreeter", greeter)?
        .build()
        .await?;

    // // send host metrics to the client

    // Wait for Ctrl+C signal
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let mut interval = time::interval(Duration::from_secs(15));

    // Send a notification every 15 seconds
    loop {
        interval.tick().await;

        let mut sys = sysinfo::System::new_all();
        sys.refresh_cpu();

        let cpu_usage = sys.global_cpu_info().cpu_usage();

        sys.refresh_memory();
        let total_memory = sys.total_memory();
        let available_memory = sys.available_memory();

        let hots_matrix = HostMetricsNotificationEvent {
            cpu_usage: cpu_usage,
            total_memory: total_memory,
            available_memory: available_memory,
        };

        //  print host matrics
        println!("Host Matrix {:?}", hots_matrix);

        let ctxt = SignalContext::new(&conn, "/org/zbus/MyGreeter")?;
        greeter.host_metrics(&ctxt, hots_matrix).await?;
    }

    // // Run the server indefinitely
    // tokio::select! {
    //     _ = ctrl_c => {
    //         println!("Received Ctrl+C signal, shutting down...");
    //     }
    // }

    // let greeter_handle = tokio::spawn(async move {
    //     if let Err(e) = greeter.get_host_metrics().await {
    //         println!("Error in notification stream: {}", e);
    //     }
    // });

    // handles.push(greeter_handle);

    // for handle in handles {
    //     handle.await?;
    // }

    Ok(())
}
