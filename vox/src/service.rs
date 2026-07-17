use std::{ffi::OsString, time::Duration};

use anyhow::Result;
use tokio::sync::oneshot;
use tracing::error;
use vox_shared::logger::setup_service_logger;
use windows_service::{
  define_windows_service,
  service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
    ServiceType,
  },
  service_control_handler::{self, ServiceControlHandlerResult},
  service_dispatcher,
};

const SERVICE_NAME: &str = "VoxDaemon";

define_windows_service!(ffi_service_main, service_main);

pub fn dispatch() -> Result<()> {
  service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
  Ok(())
}

fn service_main(_arguments: Vec<OsString>) {
  if let Err(error) = service_main_inner() {
    error!(error = ?error, "{SERVICE_NAME} failed to start");
  }
}

fn service_main_inner() -> Result<()> {
  setup_service_logger(false, "daemon");
  let (shutdown_tx, shutdown_rx) = oneshot::channel();
  let mut shutdown_tx = Some(shutdown_tx);
  let status_handle =
    service_control_handler::register(SERVICE_NAME, move |event| match event {
      ServiceControl::Stop | ServiceControl::Shutdown => {
        if let Some(tx) = shutdown_tx.take() {
          let _ = tx.send(());
        }
        ServiceControlHandlerResult::NoError
      }
      _ => ServiceControlHandlerResult::NotImplemented,
    })?;

  status_handle.set_service_status(status(ServiceState::StartPending))?;
  let result = tokio::runtime::Runtime::new()?.block_on(async {
    let (app, rx) = crate::initialize().await?;
    status_handle.set_service_status(status(ServiceState::Running))?;
    app.start_all(rx, Some(shutdown_rx)).await
  });
  status_handle.set_service_status(status(ServiceState::Stopped))?;
  result
}

fn status(state: ServiceState) -> ServiceStatus {
  ServiceStatus {
    service_type: ServiceType::OWN_PROCESS,
    current_state: state,
    controls_accepted: if state == ServiceState::Running {
      ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN
    } else {
      ServiceControlAccept::empty()
    },
    exit_code: ServiceExitCode::Win32(0),
    checkpoint: 0,
    wait_hint: Duration::default(),
    process_id: None,
  }
}
