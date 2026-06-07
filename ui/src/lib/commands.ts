import { invoke } from '@tauri-apps/api/core';
import type {
  Translation, Book, Verse, ChapterInfo, SearchResult, DbStats, ImportResult,
  InstallBibleRequest,
  PresentConfig, PreviewResult,
  OutputInfo, MonitorInfo, OutputRole, OutputSource, RoleLayout, ScriptureMode,
  TransitionTarget, CountdownSchedule,
  NdiSourceInfo,
  LicenseStatus,
  Highlight, Note, Bookmark, Tag,
  CountdownDef, MediaDef, ProductionTheme, ProductionSnapshot, ProductionPreview,
} from './types';

// ── Bible ─────────────────────────────────────────────────────────────────────

export const getTranslations = () =>
  invoke<Translation[]>('get_translations');

export const getDbStats = () =>
  invoke<DbStats>('get_db_stats');

export const getBooks = () =>
  invoke<Book[]>('get_books');

export const getChapter = (translationId: string, bookId: number, chapter: number) =>
  invoke<ChapterInfo | null>('get_chapter', { translationId, bookId, chapter });

export const getVerse = (translationId: string, bookId: number, chapter: number, verse: number) =>
  invoke<Verse | null>('get_verse', { translationId, bookId, chapter, verse });

export const searchByReference = (translationId: string, reference: string) =>
  invoke<Verse | null>('search_by_reference', { translationId, reference });

export const searchVerses = (translationId: string, query: string, limit = 30) =>
  invoke<SearchResult[]>('search_verses', {
    translationId, query, limit, bookId: null, testament: null,
  });

export const importFromJson = (jsonStr: string, translationId: string, translationName: string, language: string) =>
  invoke<ImportResult>('import_from_json', { jsonStr, translationId, translationName, language });

export const pickAndImport = (translationId: string, translationName: string, language: string) =>
  invoke<ImportResult>('pick_and_import', { translationId, translationName, language });

export const installBibleFromUrl = (request: InstallBibleRequest) =>
  invoke<ImportResult>('install_bible_from_url', { request });

export const removeTranslation = (translationId: string) =>
  invoke<ImportResult>('remove_translation', { translationId });

// ── Present / Config ──────────────────────────────────────────────────────────

export const getPresentConfig = () =>
  invoke<PresentConfig>('get_present_config');

export const setPresentConfig = (config: PresentConfig) =>
  invoke<void>('set_present_config', { config });

export const ndiPreview = (verseText: string, reference: string) =>
  invoke<PreviewResult>('ndi_preview', { verseText, reference });

export const ndiClear = () =>
  invoke<PreviewResult>('ndi_clear');

// ── Outputs ───────────────────────────────────────────────────────────────────

export const pushToAll = (verseText: string, reference: string) =>
  invoke<PreviewResult>('push_to_all', { verseText, reference });

export const clearAll = () =>
  invoke<PreviewResult>('clear_all');

export const listMonitors = () =>
  invoke<MonitorInfo[]>('list_monitors');

export const getOutputs = () =>
  invoke<OutputInfo[]>('get_outputs');

export const addNdiOutput = (label: string, sourceName: string, role?: OutputRole) =>
  invoke<OutputInfo>('add_ndi_output', { label, sourceName, role: role ?? null });

export const addDisplayOutput = (
  label: string, monitorIndex: number, monitorName: string,
  x: number, y: number, width: number, height: number,
  role?: OutputRole,
) => invoke<OutputInfo>('add_display_output', { label, monitorIndex, monitorName, x, y, width, height, role: role ?? null });

export const removeOutput = (id: string) =>
  invoke<void>('remove_output', { id });

export const toggleOutput = (id: string) =>
  invoke<OutputInfo>('toggle_output', { id });

// ── Presentation source ───────────────────────────────────────────────────────

export const listNdiSources = () =>
  invoke<NdiSourceInfo[]>('list_ndi_sources');

export const connectPresentationSource = (sourceName: string) =>
  invoke<void>('connect_presentation_source', { sourceName });

export const disconnectPresentationSource = () =>
  invoke<void>('disconnect_presentation_source');

export const getPresentationPreview = () =>
  invoke<string | null>('get_presentation_preview');

// ── Production engine ─────────────────────────────────────────────────────────

export const getProductionState = () =>
  invoke<ProductionSnapshot>('get_production_state');

export const getProductionPreview = () =>
  invoke<ProductionPreview>('get_production_preview');

export const listProductionCountdowns = () =>
  invoke<CountdownDef[]>('list_production_countdowns');

export const listProductionMedia = () =>
  invoke<MediaDef[]>('list_production_media');

export const listProductionThemes = () =>
  invoke<ProductionTheme[]>('list_production_themes');

export const setCountdown = (id: string) =>
  invoke<ProductionSnapshot>('set_countdown', { id });

export const startCountdown = () =>
  invoke<ProductionSnapshot>('start_countdown');

export const pauseCountdown = () =>
  invoke<ProductionSnapshot>('pause_countdown');

export const resumeCountdown = () =>
  invoke<ProductionSnapshot>('resume_countdown');

export const stopCountdown = () =>
  invoke<ProductionSnapshot>('stop_countdown');

export const setProductionMedia = (id: string) =>
  invoke<ProductionSnapshot>('set_production_media', { id });

export const setMediaLive = (live: boolean) =>
  invoke<ProductionSnapshot>('set_media_live', { live });

export const setAutoTransition = (enabled: boolean, target: TransitionTarget) =>
  invoke<ProductionSnapshot>('set_auto_transition', { enabled, target });

export const setScriptureMode = (mode: ScriptureMode) =>
  invoke<void>('set_scripture_mode', { mode });

export const setOutputRole = (id: string, role: OutputRole) =>
  invoke<OutputInfo>('set_output_role', { id, role });

export const setOutputSource = (id: string, source: OutputSource) =>
  invoke<OutputInfo>('set_output_source', { id, source });

export const setOutputLayout = (id: string, layout: RoleLayout) =>
  invoke<OutputInfo>('set_output_layout', { id, layout });

export const exportCountdownPack = (id: string) =>
  invoke<string>('export_countdown_pack', { id });

export const importCountdownPack = (json: string) =>
  invoke<ProductionSnapshot>('import_countdown_pack', { json });

export const importMediaFile = (path: string, title: string, category: string) =>
  invoke<ProductionSnapshot>('import_media_file', { path, title, category });

export const importVideoFile = (path: string, title: string, category: string) =>
  invoke<ProductionSnapshot>('import_video_file', { path, title, category });

export const setCountdownSchedule = (schedule: CountdownSchedule) =>
  invoke<ProductionSnapshot>('set_countdown_schedule', { schedule });

// ── Licensing ─────────────────────────────────────────────────────────────────

export const getLicenseStatus = () =>
  invoke<LicenseStatus>('get_license_status');

export const activateLicense = (tokenStr: string) =>
  invoke<LicenseStatus>('activate_license', { tokenStr });

export const deactivateLicense = () =>
  invoke<void>('deactivate_license');

export const refreshLicense = () =>
  invoke<LicenseStatus>('refresh_license');

// ── Study ─────────────────────────────────────────────────────────────────────

export const getChapterHighlights = (bookId: number, chapter: number) =>
  invoke<Highlight[]>('get_chapter_highlights', { bookId, chapter });

export const setHighlight = (bookId: number, chapter: number, verse: number, color: string) =>
  invoke<void>('set_highlight', { bookId, chapter, verse, color });

export const removeHighlight = (bookId: number, chapter: number, verse: number) =>
  invoke<void>('remove_highlight', { bookId, chapter, verse });

export const getChapterNotes = (bookId: number, chapter: number) =>
  invoke<Note[]>('get_chapter_notes', { bookId, chapter });

export const saveNote = (bookId: number, chapter: number, verse: number, body: string) =>
  invoke<void>('save_note', { bookId, chapter, verse, body });

export const getBookmarks = () =>
  invoke<Bookmark[]>('get_bookmarks');

export const toggleBookmark = (bookId: number, chapter: number, verse: number, bookName: string, verseText: string) =>
  invoke<boolean>('toggle_bookmark', { bookId, chapter, verse, bookName, verseText });

export const isBookmarked = (bookId: number, chapter: number, verse: number) =>
  invoke<boolean>('is_bookmarked', { bookId, chapter, verse });

export const getTags = () =>
  invoke<Tag[]>('get_tags');

export const createTag = (name: string, color: string) =>
  invoke<Tag>('create_tag', { name, color });
