import { invoke } from '@tauri-apps/api/core';
import type {
  Translation, Book, Verse, ChapterInfo, SearchResult,
  PresentConfig, PreviewResult,
  OutputInfo, MonitorInfo,
  NdiSourceInfo,
  LicenseStatus,
  Highlight, Note, Bookmark, Tag,
} from './types';

// ── Bible ─────────────────────────────────────────────────────────────────────

export const getTranslations = () =>
  invoke<Translation[]>('get_translations');

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

export const fetchAndCachePassage = (translationId: string, reference: string) =>
  invoke<string | null>('fetch_and_cache_passage', { translationId, reference });

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

export const addNdiOutput = (label: string, sourceName: string) =>
  invoke<OutputInfo>('add_ndi_output', { label, sourceName });

export const addDisplayOutput = (
  label: string, monitorIndex: number, monitorName: string,
  x: number, y: number, width: number, height: number,
) => invoke<OutputInfo>('add_display_output', { label, monitorIndex, monitorName, x, y, width, height });

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
