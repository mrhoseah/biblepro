import { useEffect, useMemo, useState } from 'react';
import { Download, Edit3, Eye, FileInput, Layers, Music, Plus, Radio, Save, Search, ShieldCheck, Sparkles, X } from 'lucide-react';
import { clearAll, listSongs, pushToAll, saveSong } from '../lib/commands';
import type { Song, SongSection } from '../lib/types';

const DEMO_SONGS: Song[] = [
  {
    id: 1,
    title: 'Amazing Grace',
    artist: 'John Newton',
    ccli: '22025',
    copyright: 'Public Domain',
    key: 'G',
    tempo: '72 BPM',
    arrangement: ['v1', 'c1', 'v2', 'c1', 'b1', 'c1'],
    sections: [
      { id: 'v1', label: 'Verse 1', lyrics: 'Amazing grace, how sweet the sound\nThat saved a wretch like me\nI once was lost, but now I am found\nWas blind, but now I see' },
      { id: 'c1', label: 'Chorus', lyrics: 'My chains are gone, I have been set free\nMy God, my Savior has ransomed me\nAnd like a flood His mercy reigns\nUnending love, amazing grace' },
      { id: 'v2', label: 'Verse 2', lyrics: 'Through many dangers, toils, and snares\nI have already come\nHis grace has brought me safe thus far\nAnd grace will lead me home' },
      { id: 'b1', label: 'Bridge', lyrics: 'Forever mine, forever free\nForever held in grace\nForever singing victory\nBefore my Savior face' },
    ],
  },
  {
    id: 2,
    title: 'Way Maker',
    artist: 'Sinach',
    ccli: '7115744',
    copyright: '2016 Integrity Music Europe',
    key: 'Bb',
    tempo: '68 BPM',
    arrangement: ['v1', 'c1', 'v2', 'c1', 'b1', 'c1'],
    sections: [
      { id: 'v1', label: 'Verse 1', lyrics: 'You are here, moving in our midst\nI worship You, I worship You' },
      { id: 'c1', label: 'Chorus', lyrics: 'Way maker, miracle worker\nPromise keeper, light in the darkness\nMy God, that is who You are' },
      { id: 'v2', label: 'Verse 2', lyrics: 'You are here, touching every heart\nI worship You, I worship You' },
      { id: 'b1', label: 'Bridge', lyrics: 'Even when I do not see it, You are working\nEven when I do not feel it, You are working' },
    ],
  },
];

const SONG_TABS = ['Library', 'Import', 'SongSelect', 'Playlists', 'Themes', 'Licensing'] as const;
const SONGSELECT_RESULTS = [
  { title: 'Way Maker', author: 'Sinach', ccli: '7115744' },
  { title: 'Goodness of God', author: 'Jenn Johnson, Ed Cash, Jason Ingram', ccli: '7117726' },
  { title: 'Firm Foundation', author: 'Cody Carnes, Austin Davis, Chandler Moore', ccli: '7188203' },
  { title: 'Build My Life', author: 'Pat Barrett, Matt Redman, Brett Younker', ccli: '7070345' },
];
const IMPORT_FORMATS = ['OpenLyrics', 'ProPresenter', 'EasyWorship', 'PowerPoint', 'VideoPsalm', 'Word Documents'];

function cloneSong(song: Song): Song {
  return {
    ...song,
    arrangement: [...song.arrangement],
    sections: song.sections.map(section => ({ ...section })),
  };
}

export default function Songs() {
  const [songs, setSongs] = useState<Song[]>([]);
  const [search, setSearch] = useState('');
  const [selectedId, setSelectedId] = useState(0);
  const [stagedId, setStagedId] = useState<string | null>(null);
  const [liveId, setLiveId] = useState<string | null>(null);
  const [status, setStatus] = useState('');
  const [tab, setTab] = useState<(typeof SONG_TABS)[number]>('Library');
  const [songSelectQuery, setSongSelectQuery] = useState('');

  useEffect(() => {
    listSongs()
      .then(list => {
        setSongs(list);
        if (list[0]) {
          setSelectedId(list[0].id);
          setStagedId(list[0].arrangement[0] ?? null);
        }
      })
      .catch(() => setSongs(DEMO_SONGS));
  }, []);

  const selected = songs.find(song => song.id === selectedId) ?? songs[0];
  const arranged = useMemo(
    () => selected
      ? selected.arrangement.map(id => selected.sections.find(section => section.id === id)).filter(Boolean) as SongSection[]
      : [],
    [selected],
  );
  const staged = arranged.find(section => section.id === stagedId) ?? arranged[0];
  const live = arranged.find(section => section.id === liveId) ?? null;
  const filtered = songs.filter(song =>
    `${song.title} ${song.artist}`.toLowerCase().includes(search.toLowerCase()),
  );

  const updateSection = (id: string, lyrics: string) => {
    setSongs(current => current.map(song => song.id === selected.id
      ? { ...song, sections: song.sections.map(section => section.id === id ? { ...section, lyrics } : section) }
      : song,
    ));
  };

  const addSection = () => {
    const id = `s${Date.now()}`;
    setSongs(current => current.map(song => song.id === selected.id
      ? {
          ...song,
          sections: [...song.sections, { id, label: `Section ${song.sections.length + 1}`, lyrics: 'New lyrics...' }],
          arrangement: [...song.arrangement, id],
        }
      : song,
    ));
    setStagedId(id);
  };

  const persistSong = async (song: Song) => {
    const saved = await saveSong(song);
    setSongs(current => {
      const next = current.some(s => s.id === saved.id)
        ? current.map(s => s.id === saved.id ? saved : s)
        : [saved, ...current];
      return next;
    });
    setStatus(`Saved ${saved.title}`);
  };

  const addSong = async () => {
    const base = songs[0] ?? DEMO_SONGS[0];
    const song = cloneSong(base);
    song.id = Date.now();
    song.title = 'New Song';
    song.artist = 'BiblePro Library';
    await persistSong(song);
    setSelectedId(song.id);
    setStagedId(song.arrangement[0]);
  };

  const pushSection = async (section: SongSection) => {
    const result = await pushToAll(section.lyrics, `${selected.title} - ${section.label}`);
    setLiveId(section.id);
    setStagedId(section.id);
    setStatus(result.png_b64 ? 'Lyrics live on Program Output' : 'Lyrics sent');
  };

  const release = async () => {
    await clearAll();
    setLiveId(null);
    setStatus('Released to presentation/output background');
  };

  if (!selected) {
    return <div className="flex h-full items-center justify-center bg-surface-900 text-sm text-ink-faint">Loading songs…</div>;
  }

  return (
    <div className="flex h-full bg-surface-900">
      <aside className="w-72 flex flex-col h-full bg-surface-950 border-r border-bdr shrink-0">
        <div className="px-3 py-3 border-b border-bdr">
          <div className="flex items-center gap-2 mb-3">
            <Music size={16} className="text-accent" />
            <div>
              <h1 className="text-sm font-black text-ink">Songs</h1>
              <p className="text-2xs text-ink-faint">Realtime lyrics editor</p>
            </div>
            <button onClick={addSong} className="ml-auto p-1.5 rounded bg-accent text-surface-950 hover:brightness-110">
              <Plus size={13} />
            </button>
          </div>
          <div className="relative">
            <Search size={12} className="absolute left-2 top-1/2 -translate-y-1/2 text-ink-faint" />
            <input
              value={search}
              onChange={event => setSearch(event.target.value)}
              placeholder="Search songs..."
              className="w-full bg-surface-700 border border-bdr rounded-md pl-7 pr-2 py-1.5 text-xs text-ink placeholder:text-ink-faint focus:outline-none focus:border-accent"
            />
          </div>
        </div>

        <div className="flex-1 overflow-y-auto py-1">
          {filtered.map(song => (
            <button
              key={song.id}
              onClick={() => { setSelectedId(song.id); setStagedId(song.arrangement[0]); }}
              className={[
                'w-full text-left px-3 py-3 border-l-2 transition-all',
                selected.id === song.id ? 'border-accent bg-accent-glow text-ink' : 'border-transparent text-ink-faint hover:text-ink hover:bg-surface-800',
              ].join(' ')}
            >
              <div className="text-sm font-semibold truncate">{song.title}</div>
              <div className="text-2xs text-ink-faint truncate">{song.artist} · {song.key} · {song.sections.length} sections</div>
            </button>
          ))}
        </div>
      </aside>

      <main className="flex-1 flex flex-col overflow-hidden">
        <header className="flex items-center justify-between px-5 py-3 border-b border-bdr bg-surface-900">
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-lg font-black text-ink">{selected.title}</h2>
              <span className="px-2 py-0.5 rounded-full bg-surface-700 border border-bdr text-2xs text-ink-faint">{selected.key}</span>
              <span className="px-2 py-0.5 rounded-full bg-surface-700 border border-bdr text-2xs text-ink-faint">{selected.tempo}</span>
              {selected.ccli && <span className="px-2 py-0.5 rounded-full bg-accent/10 border border-accent/30 text-2xs text-accent">CCLI {selected.ccli}</span>}
            </div>
            <p className="text-xs text-ink-faint">{selected.artist}{selected.copyright ? ` - ${selected.copyright}` : ''}</p>
          </div>
          <div className="flex gap-2">
            <button onClick={addSection} className="flex items-center gap-1.5 px-3 py-1.5 bg-surface-700 border border-bdr rounded-md text-xs text-ink hover:bg-surface-600">
              <Plus size={12} /> Section
            </button>
            <button onClick={() => persistSong(selected)} className="flex items-center gap-1.5 px-3 py-1.5 bg-surface-700 border border-bdr rounded-md text-xs text-ink hover:bg-surface-600">
              <Save size={12} /> Save
            </button>
          </div>
        </header>

        <div className="border-b border-bdr bg-surface-950 px-5 py-2">
          <div className="flex flex-wrap gap-1">
            {SONG_TABS.map(item => (
              <button
                key={item}
                onClick={() => setTab(item)}
                className={[
                  'rounded-lg px-3 py-1.5 text-xs font-black transition-all',
                  tab === item ? 'bg-accent text-surface-950' : 'text-ink-faint hover:bg-surface-800 hover:text-ink',
                ].join(' ')}
              >
                {item}
              </button>
            ))}
          </div>
        </div>

        <div className="grid flex-1 grid-cols-[minmax(0,1fr)_320px] overflow-hidden">
          <section className="overflow-y-auto p-5">
            {tab !== 'Library' && (
              <SongWorkflowPanel
                tab={tab}
                songSelectQuery={songSelectQuery}
                onSongSelectQuery={setSongSelectQuery}
                onImportSong={(title, artist, ccli) => {
                  const song = cloneSong(DEMO_SONGS[0]);
                  song.id = Date.now();
                  song.title = title;
                  song.artist = artist;
                  song.ccli = ccli;
                  song.copyright = 'Imported for local offline use';
                  setSongs(current => [song, ...current]);
                  setSelectedId(song.id);
                  setStagedId(song.arrangement[0]);
                  setStatus(`${title} imported locally for offline presentation`);
                  setTab('Library');
                }}
              />
            )}
            <div className="mb-4 flex items-center justify-between">
              <div>
                <p className="text-2xs font-bold uppercase tracking-wider text-ink-faint">Local Song Library</p>
                <p className="text-xs text-ink-faint">Click a section to stage. Double-click to go live. Imported songs work completely offline.</p>
              </div>
              <div className="flex items-center gap-2 text-2xs text-ink-faint">
                <Layers size={12} /> Verse overlay takes priority over presentation source
              </div>
            </div>

            <div className="space-y-3">
              {arranged.map((section, index) => (
                <article
                  key={`${section.id}-${index}`}
                  onClick={() => setStagedId(section.id)}
                  onDoubleClick={() => pushSection(section)}
                  className={[
                    'rounded-xl border p-4 transition-all',
                    liveId === section.id
                      ? 'border-live bg-live/10'
                      : stagedId === section.id
                      ? 'border-accent bg-accent/10'
                      : 'border-bdr bg-surface-800 hover:border-bdr-strong',
                  ].join(' ')}
                >
                  <div className="mb-3 flex items-center gap-2">
                    <span className="text-2xs font-black text-accent uppercase tracking-wider">{section.label}</span>
                    {liveId === section.id && <span className="text-2xs font-black text-live">LIVE</span>}
                    {stagedId === section.id && liveId !== section.id && <span className="text-2xs font-black text-accent">STAGED</span>}
                    <span className="ml-auto text-2xs text-ink-faint">#{index + 1}</span>
                  </div>
                  <textarea
                    value={section.lyrics}
                    onChange={event => updateSection(section.id, event.target.value)}
                    className="min-h-28 w-full resize-y rounded-lg border border-bdr bg-surface-900 px-3 py-2 font-serif text-sm leading-relaxed text-ink outline-none focus:border-accent"
                  />
                </article>
              ))}
            </div>
          </section>

          <aside className="flex flex-col border-l border-bdr bg-surface-950">
            <div className="border-b border-bdr p-3">
              <button
                onClick={() => staged && pushSection(staged)}
                disabled={!staged}
                className="mb-2 flex w-full items-center justify-center gap-2 rounded-lg bg-accent px-4 py-3 text-sm font-black text-surface-950 shadow-glow transition-all hover:brightness-110 disabled:opacity-40"
              >
                <Radio size={14} /> Send Lyrics Live
              </button>
              <button
                onClick={release}
                className="flex w-full items-center justify-center gap-2 rounded-lg border border-bdr bg-surface-700 px-4 py-2 text-xs font-bold text-ink-faint transition-all hover:border-accent hover:text-accent"
              >
                <X size={12} /> Release / Clear
              </button>
              {status && <p className="mt-2 rounded bg-surface-800 px-2 py-1 text-2xs text-ink-faint">{status}</p>}
            </div>

            <div className="border-b border-bdr p-3">
              <div className="mb-2 flex items-center gap-2 text-2xs font-bold uppercase tracking-wider text-ink-faint">
                <Eye size={12} /> Program Preview
              </div>
              <div className="slide-thumb flex items-center justify-center p-4 text-center">
                <div>
                  <p className="mb-2 text-2xs font-black uppercase tracking-wider text-accent">
                    {live ? `${selected.title} - ${live.label}` : staged ? `Next - ${staged.label}` : 'No lyrics'}
                  </p>
                  <p className="whitespace-pre-line font-serif text-sm leading-relaxed text-ink">
                    {(live ?? staged)?.lyrics ?? 'Select a section'}
                  </p>
                </div>
              </div>
            </div>

            <div className="flex-1 overflow-y-auto p-3">
              <div className="mb-2 flex items-center gap-2 text-2xs font-bold uppercase tracking-wider text-ink-faint">
                <Edit3 size={12} /> Quick cues
              </div>
              <div className="space-y-1">
                {arranged.map((section, index) => (
                  <button
                    key={`${section.id}-cue-${index}`}
                    onClick={() => setStagedId(section.id)}
                    onDoubleClick={() => pushSection(section)}
                    className={[
                      'w-full rounded-md border px-2 py-2 text-left transition-all',
                      liveId === section.id ? 'border-live bg-live/10 text-live' : stagedId === section.id ? 'border-accent bg-accent/10 text-accent' : 'border-bdr bg-surface-800 text-ink-faint hover:text-ink',
                    ].join(' ')}
                  >
                    <div className="text-xs font-semibold">{section.label}</div>
                    <div className="truncate text-2xs opacity-70">{section.lyrics.split('\n')[0]}</div>
                  </button>
                ))}
              </div>
            </div>
          </aside>
        </div>
      </main>
    </div>
  );
}

function SongWorkflowPanel({
  tab,
  songSelectQuery,
  onSongSelectQuery,
  onImportSong,
}: {
  tab: (typeof SONG_TABS)[number];
  songSelectQuery: string;
  onSongSelectQuery: (value: string) => void;
  onImportSong: (title: string, artist: string, ccli: string) => void;
}) {
  if (tab === 'SongSelect') {
    const q = songSelectQuery.trim().toLowerCase();
    const results = SONGSELECT_RESULTS.filter(song => !q || `${song.title} ${song.author}`.toLowerCase().includes(q));
    return (
      <div className="mb-5 rounded-2xl border border-bdr bg-surface-800 p-4">
        <div className="mb-3 flex items-center justify-between">
          <div>
            <p className="text-2xs font-black uppercase tracking-wider text-accent">Optional SongSelect Integration</p>
            <h3 className="text-lg font-black text-ink">Search and import lyrics</h3>
            <p className="mt-1 text-xs text-ink-faint">Import lyrics, authors, copyright, and CCLI metadata, then run fully offline.</p>
          </div>
          <ShieldCheck size={22} className="text-accent" />
        </div>
        <div className="relative mb-3">
          <Search size={13} className="absolute left-3 top-1/2 -translate-y-1/2 text-accent" />
          <input
            value={songSelectQuery}
            onChange={event => onSongSelectQuery(event.target.value)}
            placeholder="Search SongSelect..."
            className="w-full rounded-lg border border-bdr bg-surface-900 py-2 pl-9 pr-3 text-sm text-ink outline-none placeholder:text-ink-faint focus:border-accent"
          />
        </div>
        <div className="grid gap-2 md:grid-cols-2">
          {results.map(song => (
            <div key={song.ccli} className="rounded-xl border border-bdr bg-surface-900 p-3">
              <p className="text-sm font-black text-ink">{song.title}</p>
              <p className="mt-1 text-xs text-ink-faint">{song.author}</p>
              <div className="mt-3 flex items-center justify-between">
                <span className="text-2xs font-bold text-accent">CCLI {song.ccli}</span>
                <button onClick={() => onImportSong(song.title, song.author, song.ccli)} className="rounded bg-accent px-3 py-1.5 text-2xs font-black text-surface-950">
                  Import
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>
    );
  }

  if (tab === 'Import') {
    return (
      <div className="mb-5 rounded-2xl border border-bdr bg-surface-800 p-4">
        <p className="text-2xs font-black uppercase tracking-wider text-accent">Import Pipeline</p>
        <h3 className="mt-1 text-lg font-black text-ink">Normalize external song libraries</h3>
        <p className="mt-1 text-xs text-ink-faint">Bring in existing collections and convert them into BiblePro's local offline song format.</p>
        <div className="mt-4 grid gap-2 md:grid-cols-3">
          {IMPORT_FORMATS.map(format => (
            <button key={format} className="rounded-xl border border-bdr bg-surface-900 p-3 text-left hover:border-accent/50">
              <FileInput size={16} className="text-accent" />
              <p className="mt-2 text-sm font-black text-ink">{format}</p>
            </button>
          ))}
        </div>
      </div>
    );
  }

  const panels = {
    Playlists: ['Sunday Worship', 'Youth Night', 'Conference Set', 'Prayer Meeting'],
    Themes: ['Worship Lyrics Theme', 'Lower Third Lyrics', 'Stage Display Lyrics', 'Acoustic Night'],
    Licensing: ['CCLI reporting', 'Copyright metadata', 'Author tracking', 'Local audit log'],
  } as const;
  const items = panels[tab as keyof typeof panels] ?? [];

  return (
    <div className="mb-5 rounded-2xl border border-bdr bg-surface-800 p-4">
      <p className="text-2xs font-black uppercase tracking-wider text-accent">{tab}</p>
      <h3 className="mt-1 text-lg font-black text-ink">{tab === 'Licensing' ? 'Licensing and reporting' : `${tab} management`}</h3>
      <div className="mt-4 grid gap-2 md:grid-cols-2">
        {items.map(item => (
          <div key={item} className="rounded-xl border border-bdr bg-surface-900 p-3">
            {tab === 'Themes' ? <Sparkles size={16} className="text-accent" /> : <Download size={16} className="text-accent" />}
            <p className="mt-2 text-sm font-black text-ink">{item}</p>
          </div>
        ))}
      </div>
    </div>
  );
}
