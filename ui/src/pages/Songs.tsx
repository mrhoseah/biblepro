import { useState } from 'react';
import { Plus, Music, Edit3, Search } from 'lucide-react';

const DEMO_SONGS = [
  { id: 1, title: 'Amazing Grace', artist: 'John Newton', key: 'G', sections: 4 },
  { id: 2, title: 'How Great Is Our God', artist: 'Chris Tomlin', key: 'C', sections: 3 },
  { id: 3, title: 'Way Maker', artist: 'Sinach', key: 'Bb', sections: 5 },
  { id: 4, title: 'Goodness of God', artist: 'Bethel Music', key: 'E', sections: 4 },
];

export default function Songs() {
  const [search, setSearch] = useState('');
  const [selected, setSelected] = useState<typeof DEMO_SONGS[0] | null>(null);

  const filtered = DEMO_SONGS.filter(s =>
    s.title.toLowerCase().includes(search.toLowerCase()) ||
    s.artist.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="flex h-full bg-surface-900">
      {/* Song list */}
      <aside className="w-64 flex flex-col h-full bg-surface-950 border-r border-bdr shrink-0">
        <div className="flex items-center gap-2 px-3 py-2.5 border-b border-bdr">
          <div className="flex-1 relative">
            <Search size={12} className="absolute left-2 top-1/2 -translate-y-1/2 text-ink-faint" />
            <input
              value={search}
              onChange={e => setSearch(e.target.value)}
              placeholder="Search songs…"
              className="w-full bg-surface-700 border border-bdr rounded-md pl-7 pr-2 py-1.5 text-xs text-ink placeholder:text-ink-faint focus:outline-none focus:border-accent"
            />
          </div>
          <button className="p-1.5 rounded bg-accent text-surface-950 hover:brightness-110 transition-all">
            <Plus size={13} />
          </button>
        </div>

        <div className="flex-1 overflow-y-auto py-1">
          {filtered.map(song => (
            <button
              key={song.id}
              onClick={() => setSelected(song)}
              className={[
                'w-full text-left px-3 py-2.5 transition-colors border-l-2',
                selected?.id === song.id
                  ? 'border-accent bg-accent-glow text-ink'
                  : 'border-transparent text-ink-faint hover:text-ink hover:bg-surface-800',
              ].join(' ')}
            >
              <div className="flex items-center gap-2">
                <Music size={12} className="shrink-0 text-ink-faint" />
                <div className="min-w-0">
                  <div className="text-sm font-semibold truncate">{song.title}</div>
                  <div className="text-2xs text-ink-faint truncate">{song.artist} · {song.key}</div>
                </div>
              </div>
            </button>
          ))}
        </div>
      </aside>

      {/* Editor / arranger */}
      <main className="flex-1 flex flex-col overflow-hidden">
        {selected ? (
          <>
            <div className="flex items-center justify-between px-6 py-4 border-b border-bdr bg-surface-900">
              <div>
                <h1 className="text-lg font-bold text-ink">{selected.title}</h1>
                <div className="text-sm text-ink-faint">{selected.artist} · Key of {selected.key}</div>
              </div>
              <div className="flex gap-2">
                <button className="flex items-center gap-1.5 px-3 py-1.5 bg-surface-700 border border-bdr rounded-md text-xs text-ink hover:bg-surface-600 transition-colors">
                  <Edit3 size={12} /> Edit
                </button>
                <button className="px-4 py-1.5 bg-accent text-surface-950 rounded-md text-xs font-bold hover:brightness-110 transition-all">
                  Present
                </button>
              </div>
            </div>

            {/* Arrangement */}
            <div className="flex-1 overflow-y-auto p-6">
              <div className="max-w-2xl mx-auto">
                <div className="text-2xs font-bold text-ink-faint uppercase tracking-wider mb-4">Arrangement</div>
                <div className="space-y-3">
                  {['Verse 1', 'Chorus', 'Verse 2', 'Chorus', 'Bridge', 'Chorus'].map((section, i) => (
                    <div key={i} className="p-4 bg-surface-800 border border-bdr rounded-xl">
                      <div className="text-2xs font-bold text-accent mb-2 uppercase tracking-wider">{section}</div>
                      <p className="text-sm text-ink-muted leading-relaxed">
                        {section.includes('Chorus')
                          ? 'Amazing grace! How sweet the sound\nThat saved a wretch like me!'
                          : section === 'Bridge'
                          ? 'My chains are gone, I\'ve been set free...'
                          : 'Through many dangers, toils, and snares\nI have already come...'}
                      </p>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </>
        ) : (
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center">
              <Music size={48} className="mx-auto text-ink-faint mb-4 opacity-40" />
              <p className="text-ink-muted text-sm">Select a song or create one</p>
              <button className="mt-4 flex items-center gap-2 mx-auto px-4 py-2 bg-accent text-surface-950 rounded-lg text-sm font-bold hover:brightness-110 transition-all">
                <Plus size={14} /> New Song
              </button>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}
