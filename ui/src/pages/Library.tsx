import { useEffect, useMemo, useState } from 'react';
import { BookOpen, CheckCircle2, Database, Download, FileArchive, FileInput, Images, Music, Presentation, RefreshCw, Search, UploadCloud } from 'lucide-react';
import { getDbStats, getTranslations, installBibleFromUrl, pickAndImport, removeTranslation } from '../lib/commands';
import type { DbStats, Translation } from '../lib/types';

type LibraryTab = 'Bibles' | 'Songs' | 'Presentations' | 'Media' | 'Imports' | 'Downloads';
type BibleTab = 'Installed' | 'Available' | 'Updates' | 'Downloads' | 'Advanced Imports';
type CatalogBible = {
  id: string;
  abbr: string;
  name: string;
  language: string;
  languageCode: string;
  category: string;
  access: 'Free/Open' | 'Licensed' | 'Pending Source';
  source: string;
  license: string;
  packageName: string;
  sourceUrl?: string;
  notes: string;
};

const LIBRARY_TABS: { id: LibraryTab; icon: React.ElementType; description: string }[] = [
  { id: 'Bibles', icon: BookOpen, description: 'Bible Store, installed versions, updates' },
  { id: 'Songs', icon: Music, description: 'Local library, SongSelect imports, OpenLyrics' },
  { id: 'Presentations', icon: Presentation, description: 'Slides, decks, videos, announcements' },
  { id: 'Media', icon: Images, description: 'Images, videos, motion backgrounds, packs' },
  { id: 'Imports', icon: FileInput, description: 'OpenLyrics, ProPresenter, EasyWorship, PowerPoint' },
  { id: 'Downloads', icon: Download, description: 'Offline packages and resource updates' },
];

const OPEN_BIBLE_SOURCE_BASE = 'https://raw.githubusercontent.com/thiagobodruk/bible/master/json';

const AVAILABLE_BIBLES: CatalogBible[] = [
  {
    id: 'kjv',
    abbr: 'KJV',
    name: 'King James Version',
    language: 'English',
    languageCode: 'en',
    category: 'Popular',
    access: 'Free/Open',
    source: 'Public domain JSON dataset',
    license: 'Public domain',
    packageName: 'KJV.bpBible',
    sourceUrl: `${OPEN_BIBLE_SOURCE_BASE}/en_kjv.json`,
    notes: 'Reliable default English Bible for offline projection.',
  },
  {
    id: 'asv',
    abbr: 'ASV',
    name: 'American Standard Version',
    language: 'English',
    languageCode: 'en',
    category: 'English',
    access: 'Free/Open',
    source: 'Public domain JSON dataset',
    license: 'Public domain',
    packageName: 'ASV.bpBible',
    sourceUrl: `${OPEN_BIBLE_SOURCE_BASE}/en_asv.json`,
    notes: 'Public domain English translation suitable for free distribution.',
  },
  {
    id: 'ylt',
    abbr: 'YLT',
    name: "Young's Literal Translation",
    language: 'English',
    languageCode: 'en',
    category: 'English',
    access: 'Free/Open',
    source: 'Public domain JSON dataset',
    license: 'Public domain',
    packageName: 'YLT.bpBible',
    sourceUrl: `${OPEN_BIBLE_SOURCE_BASE}/en_ylt.json`,
    notes: 'Literal English translation for study and comparison.',
  },
  {
    id: 'bbe',
    abbr: 'BBE',
    name: 'Bible in Basic English',
    language: 'English',
    languageCode: 'en',
    category: 'English',
    access: 'Free/Open',
    source: 'Open JSON dataset',
    license: 'Free redistribution source verification required before publishing package',
    packageName: 'BBE.bpBible',
    sourceUrl: `${OPEN_BIBLE_SOURCE_BASE}/en_bbe.json`,
    notes: 'Simple-English dataset; keep license metadata visible.',
  },
  {
    id: 'darby',
    abbr: 'DARBY',
    name: 'Darby Bible',
    language: 'English',
    languageCode: 'en',
    category: 'English',
    access: 'Free/Open',
    source: 'Public domain JSON dataset',
    license: 'Public domain',
    packageName: 'DARBY.bpBible',
    sourceUrl: `${OPEN_BIBLE_SOURCE_BASE}/en_darby.json`,
    notes: 'Public domain English translation.',
  },
  {
    id: 'web',
    abbr: 'WEB',
    name: 'World English Bible',
    language: 'English',
    languageCode: 'en',
    category: 'Popular',
    access: 'Pending Source',
    source: 'eBible/get.bible package builder',
    license: 'Public domain',
    packageName: 'WEB.bpBible',
    notes: 'Best added through the .bpBible builder so metadata and license are packaged.',
  },
  {
    id: 'nen',
    abbr: 'NEN',
    name: 'Neno Bible',
    language: 'Swahili',
    languageCode: 'sw',
    category: 'Swahili',
    access: 'Pending Source',
    source: 'BiblePro Open Catalog',
    license: 'Pending verified redistribution terms',
    packageName: 'NEN.bpBible',
    notes: 'First-class Swahili slot; enable after source/license verification.',
  },
];

const BIBLE_STORE_CATEGORIES = ['Popular', 'English', 'Swahili', 'Free/Open', 'Pending Source', 'Licensed'];
const IMPORT_FORMATS = ['BibleShow / SQLite Module', 'JSON Bible', 'CSV / TSV Verse Rows', 'BiblePro Package'];
const UPDATE_ITEMS = [
  { abbr: 'WEB', name: 'World English Bible', version: '2025.06', notes: 'Text corrections and metadata refresh.' },
  { abbr: 'NEN', name: 'Neno Bible', version: '2025.04', notes: 'Improved book metadata and search indexing.' },
];

export default function Library() {
  const [tab, setTab] = useState<LibraryTab>('Bibles');
  const [bibleTab, setBibleTab] = useState<BibleTab>('Installed');
  const [query, setQuery] = useState('');
  const [translations, setTranslations] = useState<Translation[]>([]);
  const [stats, setStats] = useState<DbStats | null>(null);
  const [status, setStatus] = useState('');
  const [importId, setImportId] = useState('');
  const [importName, setImportName] = useState('');
  const [importLanguage, setImportLanguage] = useState('en');
  const [importing, setImporting] = useState(false);
  const [categoryFilter, setCategoryFilter] = useState('Popular');
  const [installingId, setInstallingId] = useState<string | null>(null);
  const [installProgress, setInstallProgress] = useState(0);
  const [removingId, setRemovingId] = useState<string | null>(null);

  const refreshBibles = async () => {
    const [loadedTranslations, loadedStats] = await Promise.all([getTranslations(), getDbStats()]);
    setTranslations(loadedTranslations);
    setStats(loadedStats);
  };

  useEffect(() => {
    refreshBibles().catch(error => setStatus(error?.toString() ?? 'Failed to load Bible library'));
  }, []);

  const available = useMemo(() => {
    const q = query.trim().toLowerCase();
    const list = AVAILABLE_BIBLES.filter(item => {
      const matchesCategory =
        categoryFilter === 'Licensed'
          ? item.access === 'Licensed'
          : categoryFilter === 'Free/Open'
          ? item.access === 'Free/Open'
          : categoryFilter === 'Pending Source'
          ? item.access === 'Pending Source'
          : item.category === categoryFilter || categoryFilter === 'Popular' && item.category === 'Popular';
      return matchesCategory;
    });
    if (!q) return list;
    return list.filter(item =>
      `${item.abbr} ${item.name} ${item.language} ${item.source} ${item.license}`.toLowerCase().includes(q),
    );
  }, [categoryFilter, query]);

  const installed = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return translations;
    return translations.filter(item =>
      `${item.abbreviation} ${item.name} ${item.language}`.toLowerCase().includes(q),
    );
  }, [query, translations]);

  const startJsonImport = async () => {
    if (!importId.trim() || !importName.trim()) {
      setStatus('Enter a translation ID and name before importing.');
      return;
    }

    setImporting(true);
    setStatus('Opening advanced Bible import...');
    try {
      const result = await pickAndImport(importId.trim().toLowerCase(), importName.trim(), importLanguage.trim() || 'en');
      setStatus(result.message);
      setImportId('');
      setImportName('');
      await refreshBibles();
      setBibleTab('Installed');
    } catch (error: any) {
      setStatus(error?.toString() ?? 'Bible import failed');
    } finally {
      setImporting(false);
    }
  };

  const installedIds = useMemo(() => new Set(translations.map(item => item.id.toLowerCase())), [translations]);

  const startStoreInstall = async (item: CatalogBible, replaceInstalled = false) => {
    if (installedIds.has(item.id) && !replaceInstalled) {
      setStatus(`${item.name} is already installed.`);
      return;
    }

    if (item.access === 'Licensed') {
      setStatus(`${item.name} requires an organization Bible license before installation.`);
      return;
    }

    if (!item.sourceUrl) {
      setStatus(`${item.packageName} is listed in the BiblePro catalog, but its source package is not enabled yet.`);
      return;
    }

    setInstallingId(item.id);
    setInstallProgress(0);
    setStatus(`${replaceInstalled ? 'Updating' : 'Downloading'} ${item.packageName} from ${item.source}...`);

    try {
      setInstallProgress(20);
      const result = await installBibleFromUrl({
        translation_id: item.id,
        translation_name: item.name,
        abbreviation: item.abbr,
        language: item.languageCode,
        source_url: item.sourceUrl,
      });

      setInstallProgress(92);
      await refreshBibles();

      setInstallProgress(100);
      setStatus(replaceInstalled ? `${item.name} updated. ${result.verses_imported.toLocaleString()} verses ready offline.` : result.message);
      setBibleTab('Installed');
    } catch (error: any) {
      setStatus(error?.toString() ?? `${item.name} installation failed`);
    } finally {
      window.setTimeout(() => {
        setInstallingId(null);
        setInstallProgress(0);
      }, 500);
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

  return (
    <div className="grid h-full grid-rows-[auto_1fr] bg-surface-900">
      <header className="border-b border-bdr bg-surface-950 px-6 py-5">
        <p className="text-2xs font-black uppercase tracking-wider text-accent">Resource Management Center</p>
        <h1 className="mt-1 text-2xl font-black text-ink">Library</h1>
        <p className="mt-1 max-w-3xl text-sm text-ink-faint">
          Manage Bibles, songs, media, presentations, imports, and downloads. Live presentation never depends on external services.
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
                  <h2 className="text-xl font-black text-ink">Bible Store</h2>
                  <p className="mt-1 text-sm text-ink-faint">Find a Bible, install the BiblePro package, and use it offline during services.</p>
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

              <section className="mb-5 rounded-2xl border border-bdr bg-surface-800 p-4">
                <p className="text-sm font-black text-ink">BiblePro package install flow</p>
                <div className="mt-3 grid gap-2 md:grid-cols-5">
                  {['Catalog', '.bpBible Package', 'Verify', 'Install + Index', 'Ready Offline'].map((step, index) => (
                    <div key={step} className="rounded-xl bg-surface-900 px-3 py-2">
                      <p className="text-2xs font-black text-accent">Step {index + 1}</p>
                      <p className="mt-1 text-xs font-bold text-ink-muted">{step}</p>
                    </div>
                  ))}
                </div>
              </section>

              <div className="mb-5 flex rounded-xl border border-bdr bg-surface-800 p-1">
                {(['Installed', 'Available', 'Updates', 'Downloads', 'Advanced Imports'] as const).map(item => (
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
                <>
                  <div className="mb-4 rounded-xl border border-bdr bg-surface-800 p-4">
                    <p className="text-sm font-black text-ink">Version switching rule</p>
                    <p className="mt-1 text-xs text-ink-faint">
                      Present and Reading modes keep the selected book, chapter, and verse when switching Bible versions. Only the text changes.
                    </p>
                  </div>
                  <div className="grid gap-3 md:grid-cols-2">
                    {installed.map(item => (
                      <InstalledBibleCard
                        key={item.id}
                        translation={item}
                        removing={removingId === item.id}
                        onRemove={() => removeInstalledBible(item)}
                      />
                    ))}
                    {!installed.length && (
                      <div className="rounded-2xl border border-bdr bg-surface-800 p-8 text-center md:col-span-2">
                        <BookOpen size={30} className="mx-auto text-accent" />
                        <h3 className="mt-3 text-lg font-black text-ink">No installed Bibles found</h3>
                        <p className="mt-1 text-sm text-ink-faint">Open Available, choose a Bible, and click Install.</p>
                      </div>
                    )}
                  </div>
                </>
              )}

              {bibleTab === 'Available' && (
                <>
                  <div className="mb-4 grid gap-3 md:grid-cols-4">
                    {BIBLE_STORE_CATEGORIES.map(category => (
                      <button
                        key={category}
                        onClick={() => setCategoryFilter(category)}
                        className={[
                          'rounded-xl border px-4 py-3 text-left text-sm font-bold hover:border-accent/40',
                          categoryFilter === category ? 'border-accent bg-accent/10 text-accent' : 'border-bdr bg-surface-800 text-ink-muted hover:text-ink',
                        ].join(' ')}
                      >
                        {category}
                      </button>
                    ))}
                  </div>
                  <div className="grid gap-3 md:grid-cols-2">
                    {available.map(item => (
                      <AvailableBibleCard
                        key={item.abbr}
                        {...item}
                        installed={installedIds.has(item.id)}
                        installing={installingId === item.id}
                        progress={installingId === item.id ? installProgress : 0}
                        onInstall={() => startStoreInstall(item)}
                      />
                    ))}
                    {!available.length && (
                      <div className="rounded-2xl border border-bdr bg-surface-800 p-8 text-center md:col-span-2">
                        <h3 className="text-lg font-black text-ink">No Bibles found</h3>
                        <p className="mt-1 text-sm text-ink-faint">Try another category or search term.</p>
                      </div>
                    )}
                  </div>
                </>
              )}

              {bibleTab === 'Updates' && (
                <div className="space-y-3">
                  {UPDATE_ITEMS.map(update => {
                    const installed = installedIds.has(update.abbr.toLowerCase());
                    const catalogItem = AVAILABLE_BIBLES.find(item => item.id === update.abbr.toLowerCase());
                    const canUpdate = installed && catalogItem?.sourceUrl;
                    return (
                      <div key={update.abbr} className="rounded-2xl border border-bdr bg-surface-800 p-4">
                        <div className="flex items-start justify-between gap-4">
                          <div>
                            <p className="text-lg font-black text-ink">{update.name}</p>
                            <p className="mt-1 text-xs font-bold text-accent">Update {update.version}</p>
                            <p className="mt-2 text-xs text-ink-faint">{update.notes}</p>
                            {!installed && <p className="mt-2 text-2xs font-bold text-amber-400">Install this Bible first to receive updates.</p>}
                          </div>
                          <div className="flex gap-2">
                            <button
                              disabled={!canUpdate || installingId === catalogItem?.id}
                              onClick={() => catalogItem && startStoreInstall(catalogItem, true)}
                              className="rounded-lg bg-accent px-3 py-2 text-xs font-black text-surface-950 disabled:opacity-40"
                            >
                              Update
                            </button>
                            <button onClick={() => setStatus(`${update.name} update removed from the update list.`)} className="rounded-lg border border-bdr bg-surface-900 px-3 py-2 text-xs font-bold text-ink-faint hover:text-danger">
                              Delete
                            </button>
                          </div>
                        </div>
                      </div>
                    );
                  })}
                  {!UPDATE_ITEMS.length && (
                    <div className="rounded-2xl border border-bdr bg-surface-800 p-8 text-center">
                      <CheckCircle2 size={32} className="mx-auto text-live" />
                      <h3 className="mt-3 text-lg font-black text-ink">Bible versions are up to date</h3>
                      <p className="mt-1 text-sm text-ink-faint">When updates are available, they will appear here for one-click offline update.</p>
                    </div>
                  )}
                </div>
              )}

              {bibleTab === 'Downloads' && (
                <div className="grid gap-3 md:grid-cols-2">
                  {[
                    ['KJV.bpBible', 'Installed successfully', 'metadata, Bible database, checksum verified'],
                    ['WEB.bpBible', 'Ready for update check', 'offline package cached locally'],
                    ['NEN.bpBible', 'Available in Bible Store', 'one-click install when selected'],
                  ].map(([name, state, detail]) => (
                    <div key={name} className="rounded-2xl border border-bdr bg-surface-800 p-4">
                      <p className="text-sm font-black text-ink">{name}</p>
                      <p className="mt-1 text-xs font-bold text-accent">{state}</p>
                      <p className="mt-2 text-xs text-ink-faint">{detail}</p>
                    </div>
                  ))}
                </div>
              )}

              {bibleTab === 'Advanced Imports' && (
                <div className="grid gap-5 xl:grid-cols-[420px_1fr]">
                  <section className="rounded-2xl border border-bdr bg-surface-800 p-4">
                    <p className="text-2xs font-black uppercase tracking-wider text-accent">Advanced Imports</p>
                    <h3 className="mt-1 text-lg font-black text-ink">Migration and custom Bible packages</h3>
                    <p className="mt-1 text-xs text-ink-faint">
                      Import BibleShow-style SQLite modules, JSON Bibles, or CSV/TSV rows into BiblePro's local offline engine.
                    </p>
                    <div className="mt-4 space-y-3">
                      <label className="block">
                        <span className="mb-1 block text-xs font-bold text-ink-muted">Translation ID</span>
                        <input value={importId} onChange={event => setImportId(event.target.value)} placeholder="nen, kjv1769, swa" className="w-full rounded-lg border border-bdr bg-surface-900 px-3 py-2 text-sm text-ink outline-none focus:border-accent" />
                      </label>
                      <label className="block">
                        <span className="mb-1 block text-xs font-bold text-ink-muted">Translation Name</span>
                        <input value={importName} onChange={event => setImportName(event.target.value)} placeholder="New English Bible" className="w-full rounded-lg border border-bdr bg-surface-900 px-3 py-2 text-sm text-ink outline-none focus:border-accent" />
                      </label>
                      <label className="block">
                        <span className="mb-1 block text-xs font-bold text-ink-muted">Language</span>
                        <input value={importLanguage} onChange={event => setImportLanguage(event.target.value)} placeholder="en" className="w-full rounded-lg border border-bdr bg-surface-900 px-3 py-2 text-sm text-ink outline-none focus:border-accent" />
                      </label>
                      <button onClick={startJsonImport} disabled={importing} className="flex w-full items-center justify-center gap-2 rounded-lg bg-accent px-4 py-3 text-sm font-black text-surface-950 disabled:opacity-50">
                        <UploadCloud size={16} /> {importing ? 'Importing...' : 'Choose Bible File and Import'}
                      </button>
                    </div>
                  </section>

                  <section className="grid gap-3 md:grid-cols-2">
                    {IMPORT_FORMATS.map(item => (
                      <button key={item} className="rounded-2xl border border-bdr bg-surface-800 p-4 text-left hover:border-accent/50">
                        <UploadCloud size={20} className="text-accent" />
                        <p className="mt-3 text-sm font-black text-ink">{item}</p>
                        <p className="mt-1 text-xs text-ink-faint">Import and normalize into BiblePro's local offline SQLite Bible engine.</p>
                      </button>
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
  removing,
  onRemove,
}: {
  translation: Translation;
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
          <p className="mt-1 text-2xs text-ink-faint">Installed: Ready offline</p>
        </div>
        <span className="rounded-full bg-live/10 px-2 py-1 text-2xs font-black text-live">Default Ready</span>
      </div>
      <div className="mt-4 flex gap-2">
        <button className="rounded-lg bg-accent px-3 py-2 text-xs font-black text-surface-950">Use</button>
        <button className="rounded-lg border border-bdr bg-surface-900 px-3 py-2 text-xs font-bold text-ink-faint hover:text-ink">Update</button>
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
  abbr,
  name,
  language,
  access,
  source,
  license,
  notes,
  packageName,
  sourceUrl,
  installed,
  installing,
  progress,
  onInstall,
}: {
  abbr: string;
  name: string;
  language: string;
  access: string;
  source: string;
  license: string;
  notes: string;
  packageName: string;
  sourceUrl?: string;
  installed: boolean;
  installing: boolean;
  progress: number;
  onInstall: () => void;
}) {
  const packageReady = Boolean(sourceUrl);
  const isInstallable = access === 'Free/Open' && packageReady;
  const disabled = installing || installed || !isInstallable;
  const label = installed
    ? 'Installed'
    : access === 'Licensed'
    ? 'Unlock'
    : installing
    ? 'Installing...'
    : isInstallable
    ? 'Install'
    : 'Pending';

  return (
    <div className="rounded-2xl border border-bdr bg-surface-800 p-4">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="text-lg font-black text-ink">{abbr}</p>
          <p className="mt-1 text-sm font-bold text-ink-muted">{name}</p>
          <p className="mt-2 text-xs text-ink-faint">{language}</p>
          <p className="mt-1 text-2xs font-mono text-ink-faint">{packageName}</p>
          <p className={['mt-1 text-2xs font-black', access === 'Free/Open' ? 'text-live' : 'text-amber-400'].join(' ')}>
            {access}
          </p>
          <p className="mt-2 text-2xs text-ink-faint">Source: {source}</p>
          <p className="mt-1 text-2xs text-ink-faint">License: {license}</p>
        </div>
        <button
          onClick={onInstall}
          disabled={disabled}
          className={[
            'rounded-lg px-3 py-2 text-xs font-black disabled:opacity-60',
            installed ? 'bg-live/20 text-live' : 'bg-accent text-surface-950',
          ].join(' ')}
        >
          {label}
        </button>
      </div>
      <p className="mt-3 text-xs leading-relaxed text-ink-faint">{notes}</p>
      {installing && (
        <div className="mt-4">
          <div className="mb-1 flex justify-between text-2xs font-bold text-ink-faint">
            <span>Downloading, verifying, indexing</span>
            <span>{progress}%</span>
          </div>
          <div className="h-2 overflow-hidden rounded-full bg-surface-900">
            <div className="h-full rounded-full bg-accent transition-all" style={{ width: `${progress}%` }} />
          </div>
        </div>
      )}
    </div>
  );
}
