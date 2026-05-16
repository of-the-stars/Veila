use anyhow::{Context, Result, anyhow, bail};
use smithay_client_toolkit::reexports::{
    client::{
        Connection, Dispatch, Proxy, QueueHandle,
        globals::{GlobalListContents, registry_queue_init},
        protocol::{wl_registry, wl_seat},
    },
    protocols::ext::idle_notify::v1::client::{ext_idle_notification_v1, ext_idle_notifier_v1},
};
use veila_common::ipc::LatencyReportMode;

use super::daemon::lock_running_daemon;

const DEFAULT_LOCK_AFTER_SECONDS: u64 = 300;

pub(super) async fn run_idle_monitor(
    daemon_socket_path: &std::path::Path,
    lock_after_seconds: Option<u64>,
) -> Result<()> {
    let lock_after_seconds = lock_after_seconds.unwrap_or(DEFAULT_LOCK_AFTER_SECONDS);
    let timeout_ms = timeout_millis(lock_after_seconds)?;
    let connection =
        Connection::connect_to_env().context("failed to connect to Wayland display")?;
    let (globals, mut event_queue) =
        registry_queue_init(&connection).context("failed to enumerate Wayland globals")?;
    let queue_handle = event_queue.handle();

    let mut app = IdleApp::default();
    let notifier = bind_idle_notifier(&globals, &queue_handle)?;
    let seat = bind_first_seat(&globals, &queue_handle)?;
    let _notification = notifier.get_idle_notification(timeout_ms, &seat, &queue_handle, ());
    connection
        .flush()
        .context("failed to flush idle notification request")?;

    println!("idle_monitor=true");
    println!("lock_after_seconds={lock_after_seconds}");

    loop {
        event_queue
            .blocking_dispatch(&mut app)
            .context("idle Wayland event dispatch failed")?;

        if app.take_idled() {
            match lock_running_daemon(
                daemon_socket_path,
                false,
                false,
                LatencyReportMode::Disabled,
            )
            .await
            {
                Ok(_) => {
                    println!("lock_requested=true");
                }
                Err(error) => {
                    tracing::warn!("failed to request idle lock: {error:#}");
                }
            }
        }
    }
}

fn timeout_millis(seconds: u64) -> Result<u32> {
    let millis = seconds
        .checked_mul(1_000)
        .ok_or_else(|| anyhow!("--lock-after is too large"))?;
    u32::try_from(millis).map_err(|_| anyhow!("--lock-after is too large"))
}

fn bind_idle_notifier(
    globals: &smithay_client_toolkit::reexports::client::globals::GlobalList,
    queue_handle: &QueueHandle<IdleApp>,
) -> Result<ext_idle_notifier_v1::ExtIdleNotifierV1> {
    let advertised = globals.contents().with_list(|globals| {
        globals.iter().any(|global| {
            global.interface == ext_idle_notifier_v1::ExtIdleNotifierV1::interface().name
        })
    });
    if !advertised {
        bail!("compositor does not support ext-idle-notify-v1");
    }

    globals
        .bind(queue_handle, 1..=2, ())
        .context("failed to bind ext-idle-notify-v1")
}

fn bind_first_seat(
    globals: &smithay_client_toolkit::reexports::client::globals::GlobalList,
    queue_handle: &QueueHandle<IdleApp>,
) -> Result<wl_seat::WlSeat> {
    let advertised = globals.contents().with_list(|globals| {
        globals
            .iter()
            .any(|global| global.interface == wl_seat::WlSeat::interface().name)
    });
    if !advertised {
        bail!("compositor did not advertise a wl_seat");
    }

    globals
        .bind(queue_handle, 1..=9, ())
        .context("failed to bind wl_seat")
}

#[derive(Default)]
struct IdleApp {
    idled: bool,
}

impl IdleApp {
    fn take_idled(&mut self) -> bool {
        std::mem::take(&mut self.idled)
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for IdleApp {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: <wl_registry::WlRegistry as Proxy>::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ext_idle_notifier_v1::ExtIdleNotifierV1, ()> for IdleApp {
    fn event(
        _: &mut Self,
        _: &ext_idle_notifier_v1::ExtIdleNotifierV1,
        _: ext_idle_notifier_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ext_idle_notification_v1::ExtIdleNotificationV1, ()> for IdleApp {
    fn event(
        state: &mut Self,
        _: &ext_idle_notification_v1::ExtIdleNotificationV1,
        event: ext_idle_notification_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            ext_idle_notification_v1::Event::Idled => {
                state.idled = true;
            }
            ext_idle_notification_v1::Event::Resumed => {
                state.idled = false;
            }
            _ => {}
        }
    }
}

smithay_client_toolkit::reexports::client::delegate_noop!(IdleApp: ignore wl_seat::WlSeat);

#[cfg(test)]
mod tests {
    use super::timeout_millis;

    #[test]
    fn timeout_millis_converts_seconds() {
        assert_eq!(timeout_millis(300).expect("timeout"), 300_000);
    }

    #[test]
    fn timeout_millis_rejects_overflow() {
        let error = timeout_millis(u64::MAX).expect_err("overflow should fail");
        assert!(error.to_string().contains("too large"));
    }
}
