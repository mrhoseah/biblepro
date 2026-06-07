import { useEffect, useMemo, useState } from 'react';
import { BookOpen, Database, Download, FileArchive, FileInput, Images, Music, Presentation, RefreshCw, Search, UploadCloud } from 'lucide-react';
import {
  getDbStats,
  getTranslations,
  importBibleFile,
  installBible,
  listBibleCatalog,
  removeTranslation,
} from '../lib/commands';
import type { BibleCatalogEntry, DbStats, Translation } from '../lib/types';

type LibraryTab = 'Bibles' | 'Songs' | 'Presentations' | 'Media' | 'Imports' | 'Downloads';
type BibleTab = 'Installed' | 'Get Bibles' | 'Import File';

const LIBRARY_TABS: { id: LibraryTab; icon: React.ElementType; description: string }[] = [
  { id: 'Bibles', icon: BookOpen, description: 'Installed translations and one-click downloads' },
  { id: 'Songs', icon: Music, description: 'Local library, SongSelect imports, OpenLyrics' },
  { id: 'Presentations', icon: Presentation, description: 'Slides, decks, videos, announcements' },
  { id: 'Media', icon: Images, description: 'Images, videos, motion backgrounds, packs' },
  { id: 'Imports', icon: FileInput, description: 'OpenLyrics, ProPresenter, EasyWorship, PowerPoint' },
  { id: 'Downloads', icon: Download, description: 'Offline packages and resource updates' },
];

const IMPORT_FORMATS = [
  'JSON Bible (en_kjv.json)',
  'BibleShow / SQLite module',
  'CSV / TSV verse rows',
  'Reference lines (John 3:16 text)',
];

export default function Library() {
  const [tab, setTab] = useState<LibraryTab>('Bibles');
  const [bibleTab, setBibleTab] = useState<BibleTab>('Installed');
  const [query, setQuery] = useState('');
  const [translations, setTranslations] = useState<Translation[]>([]);
  const [catalog, setCatalog] = useState<BibleCatalogEntry[]>([]);
  const [stats, setStats] = useState<DbStats | null>(null);
  const [status, setStatus] = useState('');
  const [importing, setImporting] = useState(false);
  const [installingId, setInstallingId] = useState<string | null>(null);
  const [removingId, setRemovingId] = useState<string | null>(null);

  const refreshBibles = async () => {
    const [loadedTranslations, loadedStats, loadedCatalog] = await Promise.all([
      getTranslations(),
      getDbStats(),
      listBibleCatalog(),
    ]);
    setTranslations(loadedTranslations);
    setStats(loadedStats);
    setCatalog(loadedCatalog);
  };

  useEffect(() => {
    refreshBibles().catch(error => setStatus(error?.toString() ?? 'Failed to load Bible library'));
  }, []);

  const installedIds = useMemo(
    () => new Set(translations.map(item => item.id.toLowerCase())),
    [translations],
  );

  const installed = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return translations;
    return translations.filter(item =>
      `${item.abbreviation} ${item.name} ${item.language}`.toLowerCase().includes(q),
    );
  }, [query, translations]);

  const available = useMemo(() => {
    const q = query.trim().toLowerCase();
    const list = catalog.filter(item => !installedIds.has(item.id));
    if (!q) return list;
    return list.filter(item =>
      `${item.abbreviation} ${item.name} ${item.language_name}`.toLowerCase().includes(q),
    );
  }, [catalog, installedIds, query]);

  const startFileImport = async () => {
    setImporting(true);
    setStatus('Choose a Bible file to import...');
    try {
      const result = await importBibleFile();
      setStatus(result.message);
      await refreshBibles();
      setBibleTab('Installed');
    } catch (error: any) {
      const message = error?.toString() ?? 'Bible import failed';
      if (!message.includes('No file selected')) {
        setStatus(message);
      } else {
        setStatus('');
      }
    } finally {
      setImporting(false);
    }
  };

  const startInstall = async (item: BibleCatalogEntry) => {
    if (installedIds.has(item.id)) {
      setStatus(`${item.name} is already installed.`);
      return;
    }

    setInstallingId(item.id);
    setStatus(item.bundled ? `Installing ${item.abbreviation}...` : `Downloading ${item.abbreviation}...`);

    try {
      const result = await installBible(item.id);
      await refreshBibles();
      setStatus(result.message);
      setBibleTab('Installed');
    } catch (error: any) {
      setStatus(error?.toString() ?? `${item.name} installation failed`);
    } finally {
      setInstallingId(null);
    }
  };

  const removeInstalledBible = async (translation: Translation) => {
    setRemovingId(translation.id);
    setStatus(`Removing ${translation.abbreviation}...`);
    try {
      const result = await removeTranslation(translation.id);
      setStatus(result.message);
      await refreshBibles();
    } catch (error: any) {
      setStatus(error?.toString() ?? `${translation.name} removal failed`);
    } finally {
      setRemovingId(null);
    }
  };

  const bundledInstalled = catalog.filter(item => item.bundled && installedIds.has(item.id));

  return (
    <div className="grid h-full grid-rows-[auto_1fr] bg-surface-900">
      <header className="border-b border-bdr bg-surface-950 px-6 py-5">
        <p className="text-2xs font-black uppercase tracking-wider text-accent">Resource Management Center</p>
        <h1 className="mt-1 text-2xl font-black text-ink">Library</h1>
        <p className="mt-1 max-w-3xl text-sm text-ink-faint">
          KJV is included on first launch. Add more translations with one click or import a file from another app.
        </p>
      </header>

      <div className="grid min-h-0 grid-cols-[280px_minmax(0,1fr)]">
        <aside className="border-r border-bdr bg-surface-950 p-3">
          <div className="space-y-2">
            {LIBRARY_TABS.map(item => {
              const Icon = item.icon;
              return (
                <button
                  key={item.id}
                  onClick={() => setTab(item.id)}
                  className={[
                    'flex w-full items-start gap-3 rounded-xl border p-3 text-left transition-all',
                    tab === item.id ? 'border-accent bg-accent/10 text-accent' : 'border-bdr bg-surface-800 text-ink-muted hover:text-ink',
                  ].join(' ')}
                >
                  <Icon size={16} className="mt-0.5 shrink-0" />
                  <span>
                    <span className="block text-sm font-black">{item.id}</span>
                    <span className="mt-1 block text-2xs leading-relaxed text-ink-faint">{item.description}</span>
                  </span>
                </button>
              );
            })}
          </div>
        </aside>

        <main className="min-h-0 overflow-y-auto p-6">
          {tab === 'Bibles' ? (
            <div className="mx-auto max-w-6xl">
              <div className="mb-5 flex items-center justify-between gap-4">
                <div>
                  <h2 className="text-xl font-black text-ink">Bibles</h2>
                  <p className="mt-1 text-sm text-ink-faint">
                    {bundledInstalled.length
                      ? 'King James Version is ready offline. Install more or import your own files.'
                      : 'Install a Bible below or import a file from BibleShow, JSON, or CSV.'}
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  <div className="relative w-72">
                    <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-accent" />
                    <input
                      value={query}
                      onChange={event => setQuery(event.target.value)}
                      placeholder="Search Bibles..."
                      className="w-full rounded-lg border border-bdr bg-surface-800 py-2 pl-9 pr-3 text-sm text-ink outline-none placeholder:text-ink-faint focus:border-accent"
                    />
                  </div>
                  <button onClick={() => refreshBibles()} className="rounded-lg border border-bdr bg-surface-800 p-2 text-ink-faint hover:text-ink">
                    <RefreshCw size={16} />
                  </button>
                </div>
              </div>

              {status && (
                <div className="mb-5 rounded-xl border border-accent/30 bg-accent/10 px-4 py-3 text-sm text-accent">
                  {status}
                </div>
              )}

              <section className="mb-5 grid gap-3 md:grid-cols-3">
                <StatCard label="Installed Bibles" value={stats?.translation_count ?? translations.length} />
                <StatCard label="Books Indexed" value={stats?.book_count ?? 66} />
                <StatCard label="Verses Available" value={stats?.verse_count ?? 0} />
              </section>

              <div className="mb-5 flex rounded-xl border border-bdr bg-surface-800 p-1">
                {(['Installed', 'Get Bibles', 'Import File'] as const).map(item => (
                  <button
                    key={item}
                    onClick={() => setBibleTab(item)}
                    className={[
                      'flex-1 rounded-lg px-3 py-2 text-xs font-black transition-all',
                      bibleTab === item ? 'bg-accent text-surface-950' : 'text-ink-faint hover:text-ink',
                    ].join(' ')}
                  >
                    {item}
                  </button>
                ))}
              </div>

              {bibleTab === 'Installed' && (
                <div className="grid gap-3 md:grid-cols-2">
                  {installed.map(item => {
                    const meta = catalog.find(entry => entry.id === item.id.toLowerCase());
                    return (
                      <InstalledBibleCard
                        key={item.id}
                        translation={item}
                        bundled={meta?.bundled ?? false}
                        removing={removingId === item.id}
                        onRemove={() => removeInstalledBible(item)}
                      />
                    );
                  })}
                  {!installed.length && (
                    <div className="rounded-2xl border border-bdr bg-surface-800 p-8 text-center md:col-span-2">
                      <BookOpen size={30} className="mx-auto text-accent" />
                      <h3 className="mt-3 text-lg font-black text-ink">No Bibles installed yet</h3>
                      <p className="mt-1 text-sm text-ink-faint">
                        Restart the app to load the built-in KJV, or open Get Bibles to install one.
                      </p>
                      <button
                        onClick={() => setBibleTab('Get Bibles')}
                        className="mt-4 rounded-lg bg-accent px-4 py-2 text-sm font-black text-surface-950"
                      >
                        Get Bibles
                      </button>
                    </div>
                  )}
                </div>
              )}

              {bibleTab === 'Get Bibles' && (
                <div className="grid gap-3 md:grid-cols-2">
                  {available.map(item => (
                    <AvailableBibleCard
                      key={item.id}
                      entry={item}
                      installing={installingId === item.id}
                      onInstall={() => startInstall(item)}
                    />
                  ))}
                  {!available.length && (
                    <div className="rounded-2xl border border-bdr bg-surface-800 p-8 text-center md:col-span-2">
                      <h3 className="text-lg font-black text-ink">All catalog Bibles are installed</h3>
                      <p className="mt-1 text-sm text-ink-faint">Use Import File to add a custom translation.</p>
                    </div>
                  )}
                </div>
              )}

              {bibleTab === 'Import File' && (
                <div className="grid gap-5 xl:grid-cols-[380px_1fr]">
                  <section className="rounded-2xl border border-bdr bg-surface-800 p-5">
                    <p className="text-2xs font-black uppercase tracking-wider text-accent">Import from file</p>
                    <h3 className="mt-1 text-lg font-black text-ink">Bring your own Bible</h3>
                    <p className="mt-2 text-sm leading-relaxed text-ink-faint">
                      Pick a file and BiblePro figures out the translation from the filename when it can
                      (for example <span className="font-mono text-ink-muted">en_kjv.json</span>).
                    </p>
                    <button
                      onClick={startFileImport}
                      disabled={importing}
                      className="mt-5 flex w-full items-center justify-center gap-2 rounded-lg bg-accent px-4 py-3 text-sm font-black text-surface-950 disabled:opacity-50"
                    >
                      <UploadCloud size={16} />
                      {importing ? 'Importing...' : 'Choose File and Import'}
                    </button>
                  </section>

                  <section className="grid gap-3 sm:grid-cols-2">
                    {IMPORT_FORMATS.map(item => (
                      <div key={item} className="rounded-2xl border border-bdr bg-surface-800 p-4">
                        <UploadCloud size={20} className="text-accent" />
                        <p className="mt-3 text-sm font-black text-ink">{item}</p>
                        <p className="mt-1 text-xs text-ink-faint">Imported into the local offline Bible database.</p>
                      </div>
                    ))}
                  </section>
                </div>
              )}
            </div>
          ) : (
            <div className="mx-auto max-w-4xl">
              <div className="rounded-2xl border border-bdr bg-surface-800 p-6">
                <div className="flex items-start gap-4">
                  <div className="flex size-12 items-center justify-center rounded-xl bg-accent/10 text-accent">
                    {tab === 'Songs' ? <Music size={22} /> : tab === 'Media' ? <Images size={22} /> : tab === 'Presentations' ? <Presentation size={22} /> : tab === 'Imports' ? <FileArchive size={22} /> : <Database size={22} />}
                  </div>
                  <div>
                    <h2 className="text-xl font-black text-ink">{tab}</h2>
                    <p className="mt-1 text-sm leading-relaxed text-ink-faint">
                      {tab === 'Songs' && 'Manage local songs, SongSelect imports, OpenLyrics, ProPresenter, EasyWorship, PowerPoint, and worship playlists.'}
                      {tab === 'Presentations' && 'Organize slide decks, announcements, sermon slides, and imported presentation files.'}
                      {tab === 'Media' && 'Manage motion backgrounds, stills, media packs, thumbnails, and category collections.'}
                      {tab === 'Imports' && 'Normalize external libraries into BiblePro local formats for offline use.'}
                      {tab === 'Downloads' && 'Download offline resource packages, Bible updates, media packs, and future marketplace content.'}
                    </p>
                  </div>
                </div>
              </div>
            </div>
          )}
        </main>
      </div>
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-2xl border border-bdr bg-surface-800 p-4">
      <p className="text-2xs font-black uppercase tracking-wider text-ink-faint">{label}</p>
      <p className="mt-2 text-3xl font-black text-accent">{value.toLocaleString()}</p>
    </div>
  );
}

function InstalledBibleCard({
  translation,
  bundled,
  removing,
  onRemove,
}: {
  translation: Translation;
  bundled: boolean;
  removing: boolean;
  onRemove: () => void;
}) {
  return (
    <div className="rounded-2xl border border-bdr bg-surface-800 p-4">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <p className="text-lg font-black text-ink">{translation.abbreviation}</p>
          <p className="mt-1 truncate text-sm font-bold text-ink-muted">{translation.name}</p>
          <p className="mt-2 text-xs text-ink-faint">Language: {translation.language || 'Unknown'}</p>
        </div>
        <span className={['rounded-full px-2 py-1 text-2xs font-black', bundled ? 'bg-live/10 text-live' : 'bg-accent/10 text-accent'].join(' ')}>
          {bundled ? 'Built-in' : 'Installed'}
        </span>
      </div>
      <div className="mt-4 flex gap-2">
        <button
          onClick={onRemove}
          disabled={removing}
          className="rounded-lg border border-bdr bg-surface-900 px-3 py-2 text-xs font-bold text-ink-faint hover:text-danger disabled:opacity-50"
        >
          {removing ? 'Removing...' : 'Remove'}
        </button>
      </div>
    </div>
  );
}

function AvailableBibleCard({
  entry,
  installing,
  onInstall,
}: {
  entry: BibleCatalogEntry;
  installing: boolean;
  onInstall: () => void;
}) {
  return (
    <div className="rounded-2xl border border-bdr bg-surface-800 p-4">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="text-lg font-black text-ink">{entry.abbreviation}</p>
          <p className="mt-1 text-sm font-bold text-ink-muted">{entry.name}</p>
          <p className="mt-2 text-xs text-ink-faint">{entry.language_name}</p>
          {entry.bundled && (
            <p className="mt-2 text-2xs font-black text-live">Included with BiblePro — no download</p>
          )}
        </div>
        <button
          onClick={onInstall}
          disabled={installing}
          className="rounded-lg bg-accent px-3 py-2 text-xs font-black text-surface-950 disabled:opacity-60"
        >
          {installing ? 'Installing...' : entry.bundled ? 'Install' : 'Download'}
        </button>
      </div>
    </div>
  );
}
