import { useEffect, useMemo, useState } from 'react';
import { Film, Folder, Image, Layers, Monitor, Palette, Play, Radio, Search, Shuffle } from 'lucide-react';
import { useProductionEngine } from '../hooks/useProductionEngine';
import { open } from '@tauri-apps/plugin-dialog';
import { importMediaFile, importVideoFile, listProductionMedia } from '../lib/commands';
import type { MediaDef } from '../lib/types';

type MediaType = 'image' | 'video' | 'color' | 'gradient';
type ContentType = 'Scripture' | 'Songs' | 'Announcements' | 'Presentations' | 'Countdowns';

type MediaAsset = MediaDef & {
  type: MediaType;
  tags: string[];
  duration?: string;
  preview: string;
  description: string;
};

const MEDIA_UI: Record<string, { preview: string; tags: string[]; description: string; duration?: string }> = {
  'blue-motion': { preview: 'from-blue-500 via-indigo-700 to-surface-950', tags: ['loop', 'worship', 'blue'], description: 'Looping motion background for worship and scripture moments.', duration: '00:30' },
  'gold-clouds': { preview: 'from-amber-300 via-orange-700 to-surface-950', tags: ['scripture', 'sermon', 'gold'], description: 'Warm still background for sermon passages and altar calls.' },
  'purple-prayer': { preview: 'from-purple-500 via-fuchsia-800 to-surface-950', tags: ['prayer', 'gradient'], description: 'Soft gradient for prayer, response, and quiet moments.' },
  'solid-black': { preview: 'from-black via-surface-950 to-black', tags: ['minimal', 'contrast'], description: 'Maximum readability for confidence, stage, and lower thirds.' },
  'conference-lines': { preview: 'from-cyan-500 via-slate-800 to-surface-950', tags: ['conference', 'branding'], description: 'Modern movement for conferences, crusades, and youth events.', duration: '00:45' },
  'countdown-rays': { preview: 'from-sky-400 via-blue-900 to-black', tags: ['countdown', 'motion', 'service'], description: 'Motion loop for service countdowns and livestream openings.', duration: '01:00' },
};

function toAsset(def: MediaDef): MediaAsset {
  const ui = MEDIA_UI[def.id] ?? { preview: 'from-surface-700 to-surface-950', tags: [def.category.toLowerCase()], description: def.title };
  return {
    ...def,
    type: (def.media_type === 'video' ? 'video' : def.media_type === 'image' ? 'image' : def.media_type === 'gradient' ? 'gradient' : 'color') as MediaType,
    ...ui,
  };
}

type ThemePreset = {
  id: string;
  name: string;
  contentType: ContentType;
  background: string;
  font: string;
  layout: string;
  overlay: string;
  transition: string;
};

const CATEGORIES = ['All', 'Worship', 'Prayer', 'Youth', 'Conference', 'Christmas', 'Easter', 'Sermons', 'Scripture', 'Announcements', 'Countdowns'];


const THEMES: ThemePreset[] = [
  {
    id: 'scripture-modern',
    name: 'Modern Scripture',
    contentType: 'Scripture',
    background: 'Golden Clouds',
    font: 'Montserrat Bold',
    layout: 'Centered passage',
    overlay: '40% darken + blur',
    transition: 'Fade',
  },
  {
    id: 'worship-motion',
    name: 'Worship Lyrics',
    contentType: 'Songs',
    background: 'Blue Worship Motion',
    font: 'Inter ExtraBold',
    layout: 'Large centered lyrics',
    overlay: '30% darken',
    transition: 'Crossfade',
  },
  {
    id: 'announcement-clean',
    name: 'Announcement Theme',
    contentType: 'Announcements',
    background: 'Announcement Card',
    font: 'Inter Semibold',
    layout: 'Title + details card',
    overlay: 'None',
    transition: 'Slide up',
  },
  {
    id: 'countdown-event',
    name: 'Event Countdown',
    contentType: 'Countdowns',
    background: 'Conference Lines',
    font: 'Montserrat Black',
    layout: 'Clock focused',
    overlay: '20% vignette',
    transition: 'Loop',
  },
];

export default function Media() {
  const production = useProductionEngine(500);
  const [assets, setAssets] = useState<MediaAsset[]>([]);
  const [query, setQuery] = useState('');
  const [category, setCategory] = useState('All');
  const [selectedAsset, setSelectedAsset] = useState<MediaAsset | null>(null);
  const [status, setStatus] = useState('');
  const [randomMode, setRandomMode] = useState(true);

  useEffect(() => {
    listProductionMedia()
      .then(defs => {
        const mapped = defs.map(toAsset);
        setAssets(mapped);
        if (!selectedAsset && mapped[0]) setSelectedAsset(mapped[0]);
      })
      .catch(() => setStatus('Failed to load media library'));
  }, []);

  const currentAsset = assets.find(a => a.id === production.snapshot?.current_media_id) ?? selectedAsset;

  const filteredAssets = useMemo(() => {
    const q = query.trim().toLowerCase();
    return assets.filter(asset => {
      const matchesCategory = category === 'All' || asset.category === category;
      const matchesQuery = !q ||
        asset.title.toLowerCase().includes(q) ||
        asset.category.toLowerCase().includes(q) ||
        asset.tags.some(tag => tag.includes(q));
      return matchesCategory && matchesQuery;
    });
  }, [category, query, assets]);

  const selectedTheme = selectedAsset
    ? THEMES.find(theme => theme.background === selectedAsset.title) ?? THEMES[0]
    : THEMES[0];

  return (
    <div className="grid h-full grid-rows-[auto_1fr] bg-surface-900">
      <header className="border-b border-bdr bg-surface-950 px-5 py-4">
        <div className="flex items-center justify-between gap-4">
          <div>
            <p className="text-2xs font-black uppercase tracking-wider text-accent">Media Engine</p>
            <h1 className="mt-1 text-2xl font-black text-ink">Media</h1>
            <p className="mt-1 text-sm text-ink-faint">Themes, backgrounds, playlists, and output-aware media for every content type.</p>
          </div>
          <div className="flex gap-2">
            <button
              onClick={async () => {
                const path = await open({ multiple: false, filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp'] }] });
                if (!path || typeof path !== 'string') return;
                await importMediaFile(path, path.split(/[/\\]/).pop() ?? 'Imported', category === 'All' ? 'Imported' : category);
                const defs = await listProductionMedia();
                setAssets(defs.map(toAsset));
                setStatus('Image imported');
              }}
              className="flex items-center gap-2 rounded-lg border border-bdr bg-surface-800 px-3 py-2 text-xs font-bold text-ink-faint hover:text-ink"
            >
              <Image size={14} /> Import Image
            </button>
            <button
              onClick={async () => {
                const path = await open({
                  multiple: false,
                  filters: [{ name: 'Video', extensions: ['mp4', 'webm', 'mov', 'mkv', 'gif'] }],
                });
                if (!path || typeof path !== 'string') return;
                await importVideoFile(path, path.split(/[/\\]/).pop() ?? 'Imported', category === 'All' ? 'Imported' : category);
                const defs = await listProductionMedia();
                setAssets(defs.map(toAsset));
                setStatus('Video loop imported');
              }}
              className="flex items-center gap-2 rounded-lg border border-bdr bg-surface-800 px-3 py-2 text-xs font-bold text-ink-faint hover:text-ink"
            >
              <Film size={14} /> Import Video
            </button>
            <button className="flex items-center gap-2 rounded-lg bg-accent px-3 py-2 text-xs font-black text-surface-950">
              <Palette size={14} /> New Theme
            </button>
          </div>
        </div>
      </header>

      <div className="grid min-h-0 grid-cols-[260px_minmax(0,1fr)_360px]">
        <aside className="flex min-h-0 flex-col border-r border-bdr bg-surface-950">
          <div className="border-b border-bdr p-3">
            <div className="relative">
              <Search size={13} className="absolute left-3 top-1/2 -translate-y-1/2 text-accent" />
              <input
                value={query}
                onChange={event => setQuery(event.target.value)}
                placeholder="Search media..."
                className="w-full rounded-lg border border-bdr bg-surface-800 py-2 pl-9 pr-3 text-xs font-semibold text-ink outline-none placeholder:text-ink-faint focus:border-accent"
              />
            </div>
          </div>

          <div className="min-h-0 flex-1 overflow-y-auto p-3">
            <p className="mb-2 text-2xs font-black uppercase tracking-wider text-ink-faint">Categories</p>
            <div className="space-y-1">
              {CATEGORIES.map(item => (
                <button
                  key={item}
                  onClick={() => setCategory(item)}
                  className={[
                    'flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-xs font-bold transition-all',
                    category === item ? 'bg-accent/15 text-accent ring-1 ring-accent/30' : 'text-ink-muted hover:bg-surface-800 hover:text-ink',
                  ].join(' ')}
                >
                  <Folder size={13} />
                  {item}
                </button>
              ))}
            </div>
          </div>

          <div className="border-t border-bdr p-3">
            <p className="mb-2 text-2xs font-black uppercase tracking-wider text-ink-faint">Playback</p>
            <button
              onClick={() => setRandomMode(value => !value)}
              className={[
                'flex w-full items-center justify-between rounded-lg border px-3 py-2 text-xs font-bold',
                randomMode ? 'border-accent/40 bg-accent/10 text-accent' : 'border-bdr bg-surface-800 text-ink-faint',
              ].join(' ')}
            >
              <span className="flex items-center gap-2"><Shuffle size={13} /> Random collection</span>
              <span>{randomMode ? 'On' : 'Off'}</span>
            </button>
          </div>
        </aside>

        <main className="min-h-0 overflow-y-auto p-5">
          {status && <p className="mb-3 text-xs font-bold text-accent">{status}</p>}
          {production.snapshot && (
            <p className="mb-3 text-xs text-ink-faint">
              Active layer: <span className="font-bold text-ink">{production.snapshot.active_layer}</span>
              {production.snapshot.media_live && <span className="ml-2 text-live">Media live</span>}
            </p>
          )}
          {selectedAsset && currentAsset && (
          <section className="mb-5 grid gap-4 md:grid-cols-3">
            <div className="rounded-2xl border border-live/30 bg-live/5 p-4">
              <p className="mb-3 text-2xs font-black uppercase tracking-wider text-live">Program Output</p>
              {production.preview?.png_b64 && production.snapshot?.media_live ? (
                <img src={`data:image/png;base64,${production.preview.png_b64}`} alt="Live media" className="h-32 w-full rounded-xl border border-bdr object-cover" />
              ) : (
                <PreviewCard label="Current" asset={currentAsset} tone="live" />
              )}
            </div>
            <PreviewCard label="Preview" asset={selectedAsset} tone="accent" />
            <div className="rounded-2xl border border-bdr bg-surface-800 p-4">
              <p className="mb-3 flex items-center gap-2 text-2xs font-black uppercase tracking-wider text-ink-faint">
                <Layers size={13} /> Composition
              </p>
              <div className="space-y-2 text-xs">
                {['Theme', 'Media', 'Blur / Darken', 'Text Layout', 'Output Route'].map((item, index) => (
                  <div key={item} className="flex items-center gap-2 rounded-lg bg-surface-900 px-3 py-2 text-ink-muted">
                    <span className="flex size-5 items-center justify-center rounded-full bg-accent/15 text-2xs font-black text-accent">{index + 1}</span>
                    {item}
                  </div>
                ))}
              </div>
            </div>
          </section>
          )}

          <section>
            <div className="mb-3 flex items-center justify-between">
              <div>
                <p className="text-2xs font-black uppercase tracking-wider text-ink-faint">Media Library</p>
                <h2 className="text-lg font-black text-ink">{category === 'All' ? 'All media' : category}</h2>
              </div>
              <p className="text-xs text-ink-faint">{filteredAssets.length} assets</p>
            </div>

            <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
              {filteredAssets.map(asset => (
                <button
                  key={asset.id}
                  onClick={() => setSelectedAsset(asset)}
                  className={[
                    'overflow-hidden rounded-2xl border bg-surface-800 text-left transition-all hover:border-accent/50',
                    selectedAsset?.id === asset.id ? 'border-accent shadow-glow' : 'border-bdr',
                  ].join(' ')}
                >
                  <div className={`flex h-28 items-center justify-center bg-gradient-to-br ${asset.preview}`}>
                    {asset.type === 'video' ? <Film size={28} className="text-white/80" /> : <Image size={28} className="text-white/80" />}
                  </div>
                  <div className="p-3">
                    <div className="flex items-center justify-between gap-2">
                      <p className="truncate text-sm font-black text-ink">{asset.title}</p>
                      <span className="rounded-full bg-surface-900 px-2 py-0.5 text-2xs font-bold uppercase text-ink-faint">{asset.type}</span>
                    </div>
                    <p className="mt-1 line-clamp-2 text-xs text-ink-faint">{asset.description}</p>
                    <div className="mt-3 flex flex-wrap gap-1">
                      {asset.tags.map(tag => (
                        <span key={tag} className="rounded-full bg-accent/10 px-2 py-0.5 text-2xs font-bold text-accent">{tag}</span>
                      ))}
                    </div>
                  </div>
                </button>
              ))}
            </div>
          </section>
        </main>

        <aside className="flex min-h-0 flex-col border-l border-bdr bg-surface-950">
          {selectedAsset && (
          <section className="border-b border-bdr p-4">
            <p className="text-2xs font-black uppercase tracking-wider text-ink-faint">Selected Media</p>
            <div className={`mt-3 flex h-32 items-center justify-center rounded-xl bg-gradient-to-br ${selectedAsset.preview}`}>
              {selectedAsset.type === 'video' ? <Film size={30} className="text-white/80" /> : <Image size={30} className="text-white/80" />}
            </div>
            <h2 className="mt-3 text-lg font-black text-ink">{selectedAsset.title}</h2>
            <p className="mt-1 text-xs leading-relaxed text-ink-faint">{selectedAsset.description}</p>
            <div className="mt-3 grid grid-cols-2 gap-2">
              <button
                onClick={async () => {
                  await production.selectMedia(selectedAsset.id);
                  setStatus(`Selected ${selectedAsset.title}`);
                }}
                className="flex items-center justify-center gap-2 rounded-lg border border-bdr bg-surface-800 px-3 py-2 text-xs font-bold text-ink-faint hover:text-ink"
              >
                <Play size={13} /> Select
              </button>
              <button
                onClick={async () => {
                  await production.selectMedia(selectedAsset.id);
                  await production.setMediaLive(true);
                  setStatus(`Live: ${selectedAsset.title}`);
                }}
                className="flex items-center justify-center gap-2 rounded-lg bg-accent px-3 py-2 text-xs font-black text-surface-950"
              >
                <Radio size={13} /> Go Live
              </button>
            </div>
            {production.snapshot?.media_live && (
              <button
                onClick={() => production.setMediaLive(false).then(() => setStatus('Media released'))}
                className="mt-2 w-full rounded-lg border border-bdr bg-surface-800 px-3 py-2 text-xs font-bold text-ink-faint hover:text-ink"
              >
                Stop Media
              </button>
            )}
          </section>
          )}

          <section className="min-h-0 flex-1 overflow-y-auto p-4">
            <p className="mb-3 flex items-center gap-2 text-2xs font-black uppercase tracking-wider text-ink-faint">
              <Palette size={13} /> Theme Assignments
            </p>
            <div className="space-y-3">
              {THEMES.map(theme => (
                <div
                  key={theme.id}
                  className={[
                    'rounded-xl border p-3',
                    selectedTheme.id === theme.id ? 'border-accent/50 bg-accent/10' : 'border-bdr bg-surface-800',
                  ].join(' ')}
                >
                  <div className="flex items-center justify-between gap-2">
                    <p className="text-sm font-black text-ink">{theme.name}</p>
                    <span className="rounded-full bg-surface-900 px-2 py-0.5 text-2xs font-bold text-accent">{theme.contentType}</span>
                  </div>
                  <div className="mt-2 space-y-1 text-xs text-ink-faint">
                    <p>Background: <span className="text-ink-muted">{theme.background}</span></p>
                    <p>Font: <span className="text-ink-muted">{theme.font}</span></p>
                    <p>Layout: <span className="text-ink-muted">{theme.layout}</span></p>
                    <p>Overlay: <span className="text-ink-muted">{theme.overlay}</span></p>
                    <p>Transition: <span className="text-ink-muted">{theme.transition}</span></p>
                  </div>
                </div>
              ))}
            </div>
          </section>

          <section className="border-t border-bdr p-4">
            <p className="mb-2 flex items-center gap-2 text-2xs font-black uppercase tracking-wider text-ink-faint">
              <Monitor size={13} /> Output Awareness
            </p>
            <div className="space-y-2 text-xs text-ink-faint">
              <p>Program can use motion media while stage and confidence stay solid for readability.</p>
              <p>Scripture can replace the background or overlay on the current presentation source.</p>
            </div>
          </section>
        </aside>
      </div>
    </div>
  );
}

function PreviewCard({ label, asset, tone }: { label: string; asset: MediaAsset; tone: 'live' | 'accent' }) {
  const toneClass = tone === 'live' ? 'text-live border-live/30 bg-live/5' : 'text-accent border-accent/30 bg-accent/5';

  return (
    <div className={`rounded-2xl border p-4 ${toneClass}`}>
      <p className="mb-3 text-2xs font-black uppercase tracking-wider">{label}</p>
      <div className={`flex h-32 items-center justify-center rounded-xl bg-gradient-to-br ${asset.preview}`}>
        {asset.type === 'video' ? <Film size={30} className="text-white/80" /> : <Image size={30} className="text-white/80" />}
      </div>
      <p className="mt-3 text-sm font-black text-ink">{asset.title}</p>
      <p className="mt-1 text-xs text-ink-faint">{asset.category} {asset.duration ? `- ${asset.duration}` : ''}</p>
    </div>
  );
}
