use tauri::Manager;

mod bible;
mod license;
mod output;
mod present;

use bible::{
    db::BibleDb,
    importer::{fetch_and_cache_passage, import_from_json, pick_and_import},
    lookup::{get_books, get_chapter, get_db_stats, get_translations, get_verse},
    search::{search_by_reference, search_verses},
    study::{
        get_chapter_highlights, set_highlight, remove_highlight,
        get_chapter_notes, get_all_notes, save_note,
        get_bookmarks, toggle_bookmark, delete_bookmark, is_bookmarked,
        get_tags, create_tag, tag_verse, untag_verse, get_verse_tags,
        get_study_sets, create_study_set, delete_study_set,
        get_set_verses, add_to_study_set, update_set_verse_note, remove_from_study_set,
        export_study_set,
        get_reading_plans, get_plan_days, mark_plan_day,
    },
};

use license::{
    LicenseState,
    commands::{activate_license, deactivate_license, get_license_status, refresh_license},
};

use output::{
    OutputManager,
    commands::{
        list_monitors, get_outputs, add_ndi_output, add_display_output,
        remove_output, toggle_output, push_to_all, clear_all,
        list_ndi_sources, connect_presentation_source,
        disconnect_presentation_source, get_presentation_preview,
    },
};

use present::{
    commands::{
        get_present_config, ndi_clear, ndi_is_active, ndi_preview,
        ndi_push_verse, ndi_start, ndi_stop, set_present_config,
    },
    PresentState,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;

            let db_path = app_dir.join("bible.db");
            let bible_db = BibleDb::open(&db_path).expect("Failed to initialise Bible database");
            app.manage(bible_db);
            app.manage(PresentState::new());
            app.manage(OutputManager::new());

            // Licensing — load any stored token before the window opens
            let lic = LicenseState::new(app_dir);
            lic.init();
            app.manage(lic);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Bible
            get_translations,
            get_books,
            get_chapter,
            get_verse,
            get_db_stats,
            search_verses,
            search_by_reference,
            import_from_json,
            pick_and_import,
            fetch_and_cache_passage,
            // Study
            get_chapter_highlights, set_highlight, remove_highlight,
            get_chapter_notes, get_all_notes, save_note,
            get_bookmarks, toggle_bookmark, delete_bookmark, is_bookmarked,
            get_tags, create_tag, tag_verse, untag_verse, get_verse_tags,
            get_study_sets, create_study_set, delete_study_set,
            get_set_verses, add_to_study_set, update_set_verse_note, remove_from_study_set,
            export_study_set,
            get_reading_plans, get_plan_days, mark_plan_day,
            // Presentation / NDI
            get_present_config,
            set_present_config,
            ndi_start,
            ndi_stop,
            ndi_push_verse,
            ndi_preview,
            ndi_clear,
            ndi_is_active,
            // Output manager
            list_monitors,
            get_outputs,
            add_ndi_output,
            add_display_output,
            remove_output,
            toggle_output,
            push_to_all,
            clear_all,
            // Presentation source (NDI input / compositor)
            list_ndi_sources,
            connect_presentation_source,
            disconnect_presentation_source,
            get_presentation_preview,
            // Licensing
            activate_license,
            deactivate_license,
            get_license_status,
            refresh_license,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
