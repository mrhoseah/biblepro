// ── Bible ─────────────────────────────────────────────────────────────────────

export interface Translation {
  id: string;
  name: string;
  abbreviation: string;
  language: string;
}

export interface Book {
  id: number;
  name: string;
  short_name: string;
  testament: string;
  book_order: number;
}

export interface Verse {
  id: number;
  translation_id: string;
  book_id: number;
  book_name: string;
  chapter: number;
  verse: number;
  text: string;
}

export interface ChapterInfo {
  book_name: string;
  book_id: number;
  chapter: number;
  total_chapters: number;
  verses: Verse[];
}

export interface SearchResult {
  verse: Verse;
  snippet: string;
}

// ── Present / Config ──────────────────────────────────────────────────────────

export interface Rgba { r: number; g: number; b: number; a: number }

export type BgMode = 'solid' | 'linear_h' | 'linear_v' | 'diagonal' | 'radial' | 'vignette';
export type Template = 'FullScreen' | 'LowerThird' | 'LowerThirdAccent' | 'LowerThirdSplit' | 'CardCenter' | 'MinimalText';
export type TextPosition = 'Center' | 'LowerThird' | 'UpperThird';

export interface BgStop { pos: number; color: Rgba }
export interface BackgroundDesign { mode: BgMode; stops: BgStop[] }

export interface PresentConfig {
  ndi_name: string;
  width: number;
  height: number;
  template: Template;
  background: Rgba;
  band_color: Rgba;
  accent_color: Rgba;
  verse_color: Rgba;
  reference_color: Rgba;
  verse_font_size: number;
  reference_font_size: number;
  line_spacing: number;
  position: TextPosition;
  padding_x: number;
  band_height: number;
  accent_px: number;
  card_radius: number;
  card_alpha: number;
  show_reference: boolean;
  bg_design: BackgroundDesign | null;
  band_design: BackgroundDesign | null;
}

export interface PreviewResult {
  png_b64: string;
  width: number;
  height: number;
}

// ── Outputs ───────────────────────────────────────────────────────────────────

export type OutputKind =
  | { type: 'ndi'; source_name: string }
  | { type: 'display'; monitor_index: number; monitor_name: string };

export interface OutputInfo {
  id: string;
  label: string;
  kind: OutputKind;
  enabled: boolean;
  active: boolean;
}

export interface MonitorInfo {
  index: number;
  name: string;
  width: number;
  height: number;
  x: number;
  y: number;
  is_primary: boolean;
}

// ── Presentation source ───────────────────────────────────────────────────────

export interface NdiSourceInfo {
  name: string;
}

// ── Licensing ─────────────────────────────────────────────────────────────────

export type Plan = 'free' | 'standard' | 'premium';

export interface LicenseStatus {
  plan: Plan;
  org: string;
  org_id: string;
  device_id: string;
  expires_at: number | null;
  is_in_grace: boolean;
  grace_days_remaining: number | null;
  is_active: boolean;
}

// ── Study ─────────────────────────────────────────────────────────────────────

export interface Highlight { book_id: number; chapter: number; verse: number; color: string }
export interface Note { book_id: number; chapter: number; verse: number; body: string; created_at: number; updated_at: number }
export interface Bookmark { id: number; book_id: number; chapter: number; verse: number; book_name: string; verse_text: string; label: string; created_at: number }
export interface Tag { id: number; name: string; color: string }

// ── Songs ─────────────────────────────────────────────────────────────────────

export interface Song {
  id: number;
  title: string;
  artist: string;
  key: string;
  tags: string[];
  created_at: number;
}

export interface SongSection {
  label: string; // "Verse 1", "Chorus", "Bridge"
  lyrics: string;
}

// ── Presentation queue ────────────────────────────────────────────────────────

export interface QueueItem {
  id: string;
  type: 'verse' | 'song' | 'blank' | 'image';
  title: string;
  subtitle: string;
  text: string;
  book_id?: number;
  book_name?: string;
  chapter?: number;
  verse?: number;
  translation_id?: string;
}
