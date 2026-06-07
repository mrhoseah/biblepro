use tauri::Manager;

mod bible;
mod license;
mod output;
mod present;
mod production;
mod songs;

use bible::{
    db::BibleDb,
    importer::{import_from_json, install_bible_from_url, pick_and_import, remove_translation},
    lookup::{get_books, get_chapter, get_db_stats, get_translations, get_verse},
    search::{search_by_reference, search_verses},
    study::{
        add_to_study_set, create_study_set, create_tag, delete_bookmark, delete_study_set,
        export_study_set, get_all_notes, get_bookmarks, get_chapter_highlights, get_chapter_notes,
        get_plan_days, get_reading_plans, get_set_verses, get_study_sets, get_tags, get_verse_tags,
        is_bookmarked, mark_plan_day, remove_from_study_set, remove_highlight, save_note,
        set_highlight, tag_verse, toggle_bookmark, untag_verse, update_set_verse_note,
    },
};

use license::{
    commands::{activate_license, deactivate_license, get_license_status, refresh_license},
    LicenseState,
};

use output::{
    commands::{
        add_display_output, add_ndi_output, clear_all, connect_presentation_source,
        disconnect_presentation_source, get_outputs, get_presentation_preview, list_monitors,
        list_ndi_sources, push_to_all, remove_output, set_output_layout, toggle_output,
    },
    OutputManager,
};

use present::{
    commands::{
        get_present_config, ndi_clear, ndi_is_active, ndi_preview, ndi_push_verse, ndi_start,
        ndi_stop, set_present_config,
    },
    PresentState,
};

use production::{
    commands::{
        export_countdown_pack, get_production_preview, get_production_state,
        import_countdown_pack, import_media_file, import_video_file, list_production_countdowns,
        list_production_media, list_production_themes, pause_countdown, resume_countdown,
        add_service_plan_item, add_verse_to_service_plan, apply_theme_assignment,
        clear_service_plan, create_countdown, get_service_plan, remove_service_plan_item,
        set_auto_transition, set_countdown, set_countdown_rotation, set_countdown_schedule,
        set_media_live, set_media_settings, set_output_role, set_output_source,
        set_production_media, set_scripture_mode, start_countdown, stop_countdown,
        update_countdown,
    },
    ProductionManager,
};

use songs::{
    commands::{delete_song, list_songs, save_song},
    SongStore,
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
            let production = ProductionManager::new();
            production.init_library(&app_dir);
            app.manage(production);

            let song_store = SongStore::new();
            song_store.init(&app_dir);
            app.manage(song_store);
            ProductionManager::start_compositor(app.handle().clone());

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
            install_bible_from_url,
            remove_translation,
            // Study
            get_chapter_highlights,
            set_highlight,
            remove_highlight,
            get_chapter_notes,
            get_all_notes,
            save_note,
            get_bookmarks,
            toggle_bookmark,
            delete_bookmark,
            is_bookmarked,
            get_tags,
            create_tag,
            tag_verse,
            untag_verse,
            get_verse_tags,
            get_study_sets,
            create_study_set,
            delete_study_set,
            get_set_verses,
            add_to_study_set,
            update_set_verse_note,
            remove_from_study_set,
            export_study_set,
            get_reading_plans,
            get_plan_days,
            mark_plan_day,
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
            set_output_layout,
            toggle_output,
            push_to_all,
            clear_all,
            // Presentation source (NDI input / compositor)
            list_ndi_sources,
            connect_presentation_source,
            disconnect_presentation_source,
            get_presentation_preview,
            // Production engine (countdown + media + compositor)
            get_production_state,
            get_production_preview,
            list_production_countdowns,
            list_production_media,
            list_production_themes,
            set_countdown,
            start_countdown,
            pause_countdown,
            resume_countdown,
            stop_countdown,
            set_production_media,
            set_media_live,
            set_auto_transition,
            set_scripture_mode,
            set_output_role,
            set_output_source,
            export_countdown_pack,
            import_countdown_pack,
            import_media_file,
            import_video_file,
            set_countdown_schedule,
            set_countdown_rotation,
            create_countdown,
            update_countdown,
            set_media_settings,
            apply_theme_assignment,
            get_service_plan,
            add_service_plan_item,
            remove_service_plan_item,
            clear_service_plan,
            add_verse_to_service_plan,
            // Songs
            list_songs,
            save_song,
            delete_song,
            // Licensing
            activate_license,
            deactivate_license,
            get_license_status,
            refresh_license,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
