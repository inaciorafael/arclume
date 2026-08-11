use std::time::Instant;

use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::ShortcutState;

mod actions;
mod apps;
mod clipboard;
mod dictionary;
mod history;
mod index;
mod plugins;
mod search;
mod shortcut;

pub use plugins::run_plugin_host;

use std::sync::Arc;

use actions::ActionCatalog;
use apps::AppCatalog;
use clipboard::ClipboardHistory;
use dictionary::{DictionaryEntry, DictionaryProvider};
use history::HistoryStore;
use index::FileIndex;
use plugins::PluginManager;
use search::{SearchDiagnostics, SearchResponse, search_applications};
use shortcut::{ShortcutSettings, get_global_shortcut, set_global_shortcut};

#[tauri::command]
async fn search(
    query_id: u64,
    query: String,
    catalog: tauri::State<'_, AppCatalog>,
    file_index: tauri::State<'_, Arc<FileIndex>>,
    actions: tauri::State<'_, ActionCatalog>,
    history: tauri::State<'_, HistoryStore>,
    plugins: tauri::State<'_, PluginManager>,
) -> Result<SearchResponse, String> {
    let started_at = Instant::now();
    let provider_started_at = Instant::now();
    let applications = catalog.snapshot();
    let catalog_snapshot_micros = provider_started_at.elapsed().as_micros();
    let provider_started_at = Instant::now();
    let files = file_index.search(&query, 40);
    let file_provider_micros = provider_started_at.elapsed().as_micros();
    let provider_started_at = Instant::now();
    let action_items = actions.evaluate(&query);
    let action_provider_micros = provider_started_at.elapsed().as_micros();
    let provider_started_at = Instant::now();
    let boosts = history.boosts(&query);
    let recent_items = if query.trim().is_empty() {
        history.recent_items()
    } else {
        Vec::new()
    };
    let history_provider_micros = provider_started_at.elapsed().as_micros();
    let provider_started_at = Instant::now();
    let plugin_items = plugins.search(query_id, &query);
    let plugin_provider_micros = provider_started_at.elapsed().as_micros();
    let response = search_applications(
        query_id,
        &query,
        &applications,
        &files,
        &action_items,
        &boosts,
        &recent_items,
        &plugin_items,
        12,
    );
    Ok(response.with_diagnostics(
        started_at.elapsed().as_micros(),
        SearchDiagnostics {
            catalog_snapshot_micros,
            file_provider_micros,
            action_provider_micros,
            history_provider_micros,
            plugin_provider_micros,
            ranking_micros: 0,
        },
    ))
}

#[tauri::command]
#[expect(
    clippy::too_many_arguments,
    reason = "Tauri injects managed state while the remaining fields are the stable IPC command contract"
)]
async fn execute_result(
    id: String,
    catalog: tauri::State<'_, AppCatalog>,
    file_index: tauri::State<'_, Arc<FileIndex>>,
    actions: tauri::State<'_, ActionCatalog>,
    app: tauri::AppHandle,
    confirmed: bool,
    query: String,
    title: String,
    kind: String,
    history: tauri::State<'_, HistoryStore>,
    plugins: tauri::State<'_, PluginManager>,
    dictionary: tauri::State<'_, DictionaryProvider>,
) -> Result<Option<DictionaryEntry>, String> {
    if let Some(word) = id.strip_prefix("dictionary:") {
        let entry = dictionary.lookup(word).await?;
        history.record(&query, &id, &title, &kind)?;
        return Ok(Some(entry));
    }
    if id == "history:clear" {
        if !confirmed {
            return Err("confirmation required".into());
        }
        history.clear()?;
        return Ok(None);
    }
    let result = if id.starts_with("plugin:") {
        plugins.execute(&id)
    } else if id.starts_with("action:") {
        actions.execute(&app, &id, confirmed)
    } else if let Some(raw_id) = id.strip_prefix("file:") {
        let id = raw_id
            .parse::<i64>()
            .map_err(|_| "invalid file identifier")?;
        file_index.open_item(id)
    } else {
        catalog.launch(&id)
    };
    result?;
    history.record(&query, &id, &title, &kind)?;
    Ok(None)
}

#[tauri::command]
fn get_index_roots(file_index: tauri::State<'_, Arc<FileIndex>>) -> Vec<String> {
    file_index
        .roots()
        .into_iter()
        .map(|root| root.to_string_lossy().into_owned())
        .collect()
}

#[tauri::command]
fn add_index_root(
    root: String,
    file_index: tauri::State<'_, Arc<FileIndex>>,
) -> Result<Vec<String>, String> {
    file_index
        .inner()
        .add_root(std::path::PathBuf::from(root))
        .map(paths_to_strings)
}

#[tauri::command]
fn remove_index_root(
    root: String,
    file_index: tauri::State<'_, Arc<FileIndex>>,
) -> Result<Vec<String>, String> {
    file_index
        .inner()
        .remove_root(std::path::Path::new(&root))
        .map(paths_to_strings)
}

fn paths_to_strings(paths: Vec<std::path::PathBuf>) -> Vec<String> {
    paths
        .into_iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect()
}

#[tauri::command]
async fn application_icon(
    id: String,
    catalog: tauri::State<'_, AppCatalog>,
) -> Result<Option<String>, String> {
    Ok(catalog.icon(&id))
}

fn show_launcher(app: &tauri::AppHandle) {
    let started_at = Instant::now();
    let Some(window) = app.get_webview_window("main") else {
        eprintln!("main window is unavailable");
        return;
    };

    if let Err(error) = window.center() {
        eprintln!("failed to center launcher: {error}");
    }
    if let Err(error) = window.show() {
        eprintln!("failed to show launcher: {error}");
        return;
    }
    if let Err(error) = window.set_focus() {
        eprintln!("failed to focus launcher: {error}");
    }

    let elapsed_micros = started_at.elapsed().as_micros();
    let _ = window.emit("launcher-shown", elapsed_micros);
    eprintln!("launcher show request completed in {elapsed_micros} µs");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let catalog = AppCatalog::discover();
    eprintln!("discovered {} applications", catalog.len());
    let file_index = FileIndex::open().expect("failed to open persistent file index");
    let history =
        HistoryStore::open(&file_index.database_path()).expect("failed to open local history");
    let shortcut_settings = ShortcutSettings::load();
    let clipboard_history = ClipboardHistory::open(&file_index.database_path())
        .expect("failed to open local clipboard history");
    let dictionary = DictionaryProvider::open(&file_index.database_path())
        .expect("failed to initialize Portuguese dictionary");

    tauri::Builder::default()
        .manage(catalog)
        .manage(ActionCatalog::default())
        .manage(Arc::clone(&file_index))
        .manage(history)
        .manage(shortcut_settings)
        .manage(Arc::clone(&clipboard_history))
        .manage(dictionary)
        .manage(PluginManager)
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        show_launcher(app);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(move |app| {
            let settings = app.state::<ShortcutSettings>();
            if let Err(error) = shortcut::register_initial(app.handle(), &settings) {
                eprintln!("{error}");
                show_launcher(app.handle());
            }
            app.state::<Arc<FileIndex>>().start();
            clipboard_history.start(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            search,
            execute_result,
            get_global_shortcut,
            set_global_shortcut,
            get_index_roots,
            add_index_root,
            remove_index_root,
            application_icon,
            clipboard::get_clipboard_settings,
            clipboard::set_clipboard_settings,
            clipboard::list_clipboard_items,
            clipboard::clipboard_image,
            clipboard::restore_clipboard_item,
            clipboard::clear_clipboard_history
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Arclume");
}
