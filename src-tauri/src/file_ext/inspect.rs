use std::{
    error::Error,
    mem,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

use itertools::Itertools;
use tauri::{AppHandle, Emitter, Listener, WebviewUrl, WebviewWindow};
static ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct Inspect {
    app: AppHandle,
    pub state: Arc<Mutex<InspectState>>,
}

impl Inspect {
    pub fn new(app: AppHandle) -> tauri::Result<Inspect> {
        // TODO: if we know the path on creation, we can pass it as an arg in the url, rather than emitting events later
        let label = format!("extract-{}", ID.fetch_add(1, Ordering::Relaxed));

        let window = WebviewWindow::builder(&app, &label, WebviewUrl::default())
            .title("解压")
            .inner_size(1170.0, 850.0)
            .resizable(false)
            .fullscreen(false)
            // .disable_drag_drop_handler()
            .build()?;

        let window_label = label.clone();

        let inspect = Inspect {
            app: app.clone(),
            state: Arc::new(Mutex::new(InspectState {
                label,
                ready: false,
                path: Vec::new(),
                errors: Vec::new(),
            })),
        };
        let state = inspect.state.clone();
        window.listen("ready", move |event| {
            let mut state = state.lock().unwrap();
            // For some reason, the "ready" event is received from all windows, even though we
            // explicitly listen on a specific window. I suspect this is a Tauri bug, but for now
            // this is the workaround.
            match serde_json::from_str::<String>(event.payload()) {
                Ok(payload) => {
                    if payload == window_label {
                        if let Err(err) = state.ready(&app) {
                            state.error(&app, err);
                        }

                        app.unlisten(event.id());
                    }
                }
                Err(err) => state.error(&app, err),
            }
        });

        Ok(inspect)
    }

    #[allow(dead_code)]
    pub fn app_handle(&self) -> AppHandle {
        self.app.clone()
    }

    #[allow(dead_code)]
    pub fn send(&self, path: Vec<PathBuf>) -> tauri::Result<()> {
        let mut state = self.state.lock().unwrap();
        state.send(&self.app, path)
    }

    #[allow(dead_code)]
    pub fn error_string(&self, error: String) {
        let mut state = self.state.lock().unwrap();
        state.error_string(&self.app, error)
    }

    #[allow(dead_code)]
    pub fn error<T: Error>(&self, err: T) {
        self.error_string(err.to_string());
    }
}

#[derive(Debug)]
pub struct InspectState {
    pub label: String,
    ready: bool,
    path: Vec<PathBuf>,
    // Errors passed to the client to be displayed (or logged)
    errors: Vec<String>,
}

impl InspectState {
    #[allow(dead_code)]
    pub fn send(&mut self, app: &AppHandle, paths: Vec<PathBuf>) -> tauri::Result<()> {
        match self.ready {
            true => {
                app.emit_to(&self.label, "inspect", paths)?;
            }
            false => {
                self.path = paths;
            }
        }

        Ok(())
    }

    pub fn ready(&mut self, app: &AppHandle) -> tauri::Result<()> {
        self.ready = true;

        if !self.path.is_empty() {
            app.emit_to(&self.label, "in-extract", self.path.clone())?;

            // No longer using self.errors, so clear out the allocation
            for error in mem::take(&mut self.errors) {
                self.error_string(app, error);
            }
        }

        Ok(())
    }

    pub fn error_string(&mut self, app: &AppHandle, error: String) {
        match self.ready {
            true => {
                // TODO: if this errors, save to log file, ignore result for now
                let _ = app.emit_to(&self.label, "error", error);
            }
            false => {
                self.errors.push(error);
            }
        }
    }

    pub fn error<T: Error>(&mut self, app: &AppHandle, err: T) {
        self.error_string(app, err.to_string());
    }
}
