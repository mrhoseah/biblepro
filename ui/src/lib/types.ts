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

export interface DbStats {
  translation_count: number;
  book_count: number;
  verse_count: number;
  translations: Translation[];
}

export interface ImportResult {
  translation_id: string;
  verses_imported: number;
  message: string;
}

export interface InstallBibleRequest {
  translation_id: string;
  translation_name: string;
  abbreviation: string;
  language: string;
  source_url: string;
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

export type OutputRole = 'program' | 'preview' | 'confidence' | 'stage' | 'lobby' | 'livestream';
export type OutputSource = 'auto' | 'presentation' | 'scripture' | 'media' | 'countdown';
export type RoleLayout = 'auto' | 'full' | 'stage_timer' | 'confidence_text' | 'lobby_countdown' | 'livestream_safe';
export type ScriptureMode = 'replace' | 'overlay';
export type TransitionTarget = 'idle' | 'media' | 'stop';

export interface OutputInfo {
  id: string;
  label: string;
  kind: OutputKind;
  enabled: boolean;
  active: boolean;
  role: OutputRole;
  source: OutputSource;
  layout?: RoleLayout;
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

// ── Production engine ─────────────────────────────────────────────────────────

export type CountdownStyle = 'numeric' | 'ring' | 'loader' | 'theme';
export type CountdownStatus = 'idle' | 'running' | 'paused' | 'ended';

export interface CountdownDef {
  id: string;
  name: string;
  duration: number;
  style: CountdownStyle;
  theme_id: string;
  headline: string;
  subline: string;
  loader: string;
  media_id?: string | null;
}

export interface CountdownRuntime {
  def: CountdownDef;
  status: CountdownStatus;
  remaining_secs: number;
}

export interface MediaDef {
  id: string;
  title: string;
  category: string;
  media_type: string;
  background: BackgroundDesign;
  motion_id?: string | null;
}

export interface CountdownSchedule {
  enabled: boolean;
  countdown_id: string;
  service_at_unix: number;
  lead_secs: number;
  fired: boolean;
}

export interface ScheduleStatus {
  schedule: CountdownSchedule;
  seconds_until_start: number;
  ready: boolean;
}

export interface ProductionTheme {
  id: string;
  name: string;
  background: BackgroundDesign;
  headline_color: Rgba;
  timer_color: Rgba;
  subline_color: Rgba;
}

export interface ProductionSnapshot {
  countdown: CountdownRuntime | null;
  current_media_id: string | null;
  media_live: boolean;
  presentation_connected: boolean;
  active_layer: 'scripture' | 'countdown' | 'presentation' | 'media' | 'idle' | string;
  auto_transition: boolean;
  transition_target: TransitionTarget;
  scripture_mode: ScriptureMode | string;
  custom_countdown_count: number;
  custom_media_count: number;
  schedule?: ScheduleStatus;
}

export interface ProductionPreview extends PreviewResult {
  snapshot: ProductionSnapshot;
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
