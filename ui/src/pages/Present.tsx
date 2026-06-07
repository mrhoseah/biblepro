import { useCallback, useEffect, useMemo, useState } from 'react';
import { Clock, History, ListPlus, Play, Radio, RefreshCw, Star, X, Zap } from 'lucide-react';
import ScriptureNavigator from '../components/ScriptureNavigator';
import { useScriptureEngine } from '../hooks/useScriptureEngine';
import { useProductionEngine } from '../hooks/useProductionEngine';
import {
  clearAll,
  connectPresentationSource,
  disconnectPresentationSource,
  getOutputs,
  getPresentationPreview,
  listNdiSources,
  pushToAll, addVerseToServicePlan, removeServicePlanItem, clearServicePlan, toggleBookmark,
} from '../lib/commands';
import type { NdiSourceInfo, OutputInfo, ServicePlanItem, Verse } from '../lib/types';

type QueueItem = { id: string; verse: Verse };
type HistoryEntry = { id: string; verse: Verse; time: string };
type PresentMode = 'Live' | 'Reading' | 'Planning';

function refOf(verse: Verse) {
  return `${verse.book_name} ${verse.chapter}:${verse.verse}`;
}

function shortTime() {
  return new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

export default function Present() {
  const engine = useScriptureEngine();
  const production = useProductionEngine(600);

  const [liveVerse, setLiveVerse] = useState<Verse | null>(null);
  const [stagedVerse, setStagedVerse] = useState<Verse | null>(null);
  const [previewSrc, setPreviewSrc] = useState('');
  const [status, setStatus] = useState('Standby');
  const [queue, setQueue] = useState<QueueItem[]>([]);
  const [queueIndex, setQueueIndex] = useState(0);
  const [recent, setRecent] = useState<Verse[]>([]);
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [favorites, setFavorites] = useState<Verse[]>([]);
  const [outputs, setOutputs] = useState<OutputInfo[]>([]);
  const [mode, setMode] = useState<PresentMode>('Live');

  const [ndiSources, setNdiSources] = useState<NdiSourceInfo[]>([]);
  const [selectedSource, setSelectedSource] = useState('');
  const [connectedSource, setConnectedSource] = useState<string | null>(null);
  const [presPreviewSrc, setPresPreviewSrc] = useState('');
  const [sourceStatus, setSourceStatus] = useState('');

  useEffect(() => {
    getOutputs().then(setOutputs).catch(() => setOutputs([]));
  }, []);

  useEffect(() => {
    if (!engine.selectedVerse) return;
    setStagedVerse(engine.selectedVerse);
  }, [engine.selectedVerse]);

  useEffect(() => {
    if (!connectedSource) {
      setPresPreviewSrc('');
      return;
    }
    const poll = () => {
      getPresentationPreview()
        .then(b64 => b64 && setPresPreviewSrc(`data:image/png;base64,${b64}`))
        .catch(() => {});
    };
    poll();
    const timer = setInterval(poll, 600);
    return () => clearInterval(timer);
  }, [connectedSource]);

  const stagedIndex = useMemo(() => {
    if (!stagedVerse || !engine.chapterInfo) return -1;
    return engine.chapterInfo.verses.findIndex(item => item.id === stagedVerse.id);
  }, [engine.chapterInfo, stagedVerse]);

  const liveIndex = useMemo(() => {
    if (!liveVerse || !engine.chapterInfo) return -1;
    return engine.chapterInfo.verses.findIndex(item => item.id === liveVerse.id);
  }, [engine.chapterInfo, liveVerse]);

  const navigationIndex = stagedIndex >= 0 ? stagedIndex : liveIndex;

  const nextVerse = useMemo(() => {
    if (!engine.chapterInfo || navigationIndex < 0) return null;
    return engine.chapterInfo.verses[navigationIndex + 1] ?? null;
  }, [engine.chapterInfo, navigationIndex]);

  const previousVerse = useMemo(() => {
    if (!engine.chapterInfo || navigationIndex <= 0) return null;
    return engine.chapterInfo.verses[navigationIndex - 1] ?? null;
  }, [engine.chapterInfo, navigationIndex]);

  const stageVerse = useCallback((verse: Verse | null) => {
    if (!verse) {
      setStagedVerse(null);
      return;
    }
    if (engine.chapterInfo?.book_id === verse.book_id && engine.chapterInfo.chapter === verse.chapter) {
      engine.selectVerse(verse.verse);
    }
    setStagedVerse(verse);
  }, [engine]);

  const liveOutputs = outputs.filter(output => output.enabled).length;

  const pushVerse = useCallback(async (verse: Verse) => {
    const result = await pushToAll(verse.text, refOf(verse));
    setPreviewSrc(result.png_b64 ? `data:image/png;base64,${result.png_b64}` : '');
    setLiveVerse(verse);
    setStagedVerse(null);
    setStatus(`Live: ${refOf(verse)}`);
    setRecent(current => [verse, ...current.filter(item => item.id !== verse.id)].slice(0, 8));
    setHistory(current => [{ id: `${Date.now()}`, verse, time: shortTime() }, ...current].slice(0, 30));

    if (engine.chapterInfo) {
      const index = engine.chapterInfo.verses.findIndex(item => item.id === verse.id);
      stageVerse(engine.chapterInfo.verses[index + 1] ?? null);
    }
  }, [engine.chapterInfo, stageVerse]);

  const clearLive = useCallback(async () => {
    const result = await clearAll();
    setPreviewSrc(result.png_b64 ? `data:image/png;base64,${result.png_b64}` : '');
    setLiveVerse(null);
    setStatus(connectedSource ? 'Released to presentation' : 'Cleared');
  }, [connectedSource]);

  const servicePlan = production.snapshot?.service_plan;

  const addToQueue = useCallback((verse = stagedVerse) => {
    if (!verse) return;
    setQueue(current => [...current, { id: `${Date.now()}-${verse.id}`, verse }]);
    setStatus(`Added ${refOf(verse)} to sermon flow`);
  }, [stagedVerse]);

  const addToServicePlan = useCallback(async (verse = stagedVerse) => {
    if (!verse) return;
    await addVerseToServicePlan(
      verse.translation_id,
      verse.book_id,
      verse.chapter,
      verse.verse,
      refOf(verse),
      verse.text,
    );
    setStatus(`Added ${refOf(verse)} to service plan`);
  }, [stagedVerse]);

  const planItemLabel = (item: ServicePlanItem) => {
    switch (item.kind.type) {
      case 'verse': return item.kind.reference;
      case 'song': return item.kind.title;
      case 'countdown': return item.kind.name;
      case 'media': return item.kind.title;
      case 'blank': return item.kind.label;
      default: return 'Item';
    }
  };

  const pushQueueItem = useCallback(async (index: number) => {
    const item = queue[index];
    if (!item) return;
    setQueueIndex(index);
    await pushVerse(item.verse);
  }, [pushVerse, queue]);

  const scanSources = async () => {
    setSourceStatus('Scanning...');
    try {
      const sources = await listNdiSources();
      setNdiSources(sources);
      setSelectedSource(sources[0]?.name ?? '');
      setSourceStatus(sources.length ? `${sources.length} source(s) found` : 'No NDI sources found');
    } catch (error: any) {
      setSourceStatus(error?.toString() ?? 'Scan failed');
    }
  };

  const connectSource = async () => {
    if (!selectedSource) return;
    setSourceStatus('Connecting...');
    try {
      await connectPresentationSource(selectedSource);
      setConnectedSource(selectedSource);
      setSourceStatus(`Connected: ${selectedSource}`);
    } catch (error: any) {
      setSourceStatus(error?.toString() ?? 'Connection failed');
    }
  };

  const disconnectSource = async () => {
    await disconnectPresentationSource();
    setConnectedSource(null);
    setSourceStatus('Presentation source disconnected');
  };

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      const tag = (event.target as HTMLElement).tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;

      if (event.key === '/') {
        event.preventDefault();
        const searchInput = document.querySelector<HTMLInputElement>('[data-scripture-search="true"]');
        searchInput?.focus();
        searchInput?.select();
      }
      if (event.key === ' ') {
        event.preventDefault();
        if (stagedVerse) pushVerse(stagedVerse);
      }
      if (event.key === 'ArrowRight' || event.key === 'ArrowDown') {
        event.preventDefault();
        if (nextVerse) stageVerse(nextVerse);
      }
      if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') {
        event.preventDefault();
        if (previousVerse) stageVerse(previousVerse);
      }
      if (event.key === 'c' || event.key === 'C' || event.key === 'Escape') {
        event.preventDefault();
        clearLive();
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [clearLive, nextVerse, previousVerse, pushVerse, stageVerse, stagedVerse]);

  return (
    <div className="grid h-full grid-rows-[auto_1fr_auto] overflow-hidden bg-surface-900">
      <header className="border-b border-bdr bg-surface-950 px-4 py-3">
        <div className="flex items-center gap-3">
          <div className="min-w-0 flex-1">
            <p className="text-2xs font-black uppercase tracking-wider text-accent">Present Organization</p>
            <h1 className="truncate text-lg font-black text-ink">
              {mode === 'Live'
                ? stagedVerse ? `Next: ${refOf(stagedVerse)}` : liveVerse ? `Current: ${refOf(liveVerse)}` : 'Choose Book, Chapter, Verse'
                : mode === 'Reading'
                ? `${engine.chapterInfo?.book_name ?? engine.selectedBook?.name ?? 'Bible'} ${engine.chapter}${engine.verse ? `:${engine.verse}` : ''}`
                : 'Plan scripture, songs, countdowns, and presentations'}
            </h1>
          </div>
          <div className="flex rounded-xl border border-bdr bg-surface-800 p-1">
            {(['Live', 'Reading', 'Planning'] as const).map(item => (
              <button
                key={item}
                onClick={() => setMode(item)}
                className={[
                  'rounded-lg px-3 py-2 text-xs font-black transition-all',
                  mode === item ? 'bg-accent text-surface-950' : 'text-ink-faint hover:text-ink',
                ].join(' ')}
              >
                {item}
              </button>
            ))}
          </div>
          {mode === 'Live' && (
            <button
              onClick={() => stagedVerse && pushVerse(stagedVerse)}
              disabled={!stagedVerse}
              className="flex h-12 items-center gap-2 rounded-xl bg-accent px-5 text-sm font-black text-surface-950 shadow-glow transition-all hover:brightness-110 disabled:opacity-40"
            >
              <Play size={14} fill="currentColor" /> Present
            </button>
          )}
        </div>
      </header>

      <div className="grid min-h-0 grid-cols-[460px_minmax(0,1fr)_360px]">
        <ScriptureNavigator
          engine={engine}
          title="Present Navigation"
          compactBooks={false}
          stagedVerse={stagedVerse}
          liveVerse={liveVerse}
          onVerseSelect={stageVerse}
          onVerseDoubleClick={pushVerse}
        />

        <main className="flex min-h-0 flex-col">
          {mode === 'Live' && (
          <div className="grid min-h-0 flex-1 grid-rows-[auto_1fr]">
            <section className="border-b border-bdr bg-surface-900 p-4">
              <div className="grid gap-3 md:grid-cols-2">
                <div className="rounded-2xl border border-live/30 bg-live/5 p-4">
                  <p className="mb-2 flex items-center gap-2 text-2xs font-black uppercase tracking-wider text-live">
                    <Radio size={12} /> Current
                  </p>
                  {liveVerse ? (
                    <>
                      <h2 className="text-lg font-black text-live">{refOf(liveVerse)}</h2>
                      <p className="mt-2 line-clamp-4 font-serif text-sm leading-relaxed text-ink">{liveVerse.text}</p>
                    </>
                  ) : (
                    <p className="text-sm text-ink-faint">Nothing live. Stage a verse and press Present.</p>
                  )}
                </div>

                <div className="rounded-2xl border border-accent/30 bg-accent/5 p-4">
                  <p className="mb-2 flex items-center gap-2 text-2xs font-black uppercase tracking-wider text-accent">
                    <Zap size={12} /> Next
                  </p>
                  {stagedVerse ? (
                    <>
                      <h2 className="text-lg font-black text-accent">{refOf(stagedVerse)}</h2>
                      <p className="mt-2 line-clamp-4 font-serif text-sm leading-relaxed text-ink">{stagedVerse.text}</p>
                    </>
                  ) : (
                    <p className="text-sm text-ink-faint">No next verse staged.</p>
                  )}
                </div>
              </div>
              <div className="mt-3 grid gap-2 md:grid-cols-4">
                <button disabled={!previousVerse} onClick={() => previousVerse && stageVerse(previousVerse)} className="rounded-xl border border-bdr bg-surface-800 px-3 py-2 text-xs font-bold text-ink-faint hover:text-ink disabled:opacity-40">
                  Previous Verse
                </button>
                <button disabled={!nextVerse} onClick={() => nextVerse && stageVerse(nextVerse)} className="rounded-xl border border-bdr bg-surface-800 px-3 py-2 text-xs font-bold text-ink-faint hover:text-ink disabled:opacity-40">
                  Next Verse
                </button>
                <button disabled={!stagedVerse} onClick={() => stagedVerse && addToQueue(stagedVerse)} className="rounded-xl border border-bdr bg-surface-800 px-3 py-2 text-xs font-bold text-accent disabled:opacity-40">
                  Add to Flow
                </button>
                <button disabled={!stagedVerse} onClick={() => stagedVerse && pushVerse(stagedVerse)} className="rounded-xl bg-accent px-3 py-2 text-xs font-black text-surface-950 disabled:opacity-40">
                  Present Now
                </button>
              </div>
            </section>

            <section className="min-h-0 overflow-y-auto p-4">
              <div className="mx-auto max-w-4xl">
                <div className="mb-3 flex items-center justify-between">
                  <div>
                    <p className="text-2xs font-black uppercase tracking-wider text-ink-faint">Live Preview</p>
                    <h1 className="text-xl font-black text-ink">
                      {engine.chapterInfo?.book_name ?? engine.selectedBook?.name} {engine.chapter}
                    </h1>
                  </div>
                  <span className="rounded-full border border-bdr bg-surface-800 px-3 py-1 text-2xs font-bold text-ink-faint">
                    {liveOutputs} output{liveOutputs === 1 ? '' : 's'} enabled
                  </span>
                </div>

                <div className="slide-thumb mb-4 flex min-h-[220px] items-center justify-center p-6 text-center">
                  {previewSrc ? (
                    <img src={previewSrc} alt="Live output preview" className="h-full w-full object-contain" />
                  ) : (
                    <div>
                      <p className="mb-2 text-2xs font-black uppercase tracking-wider text-accent">Media themed program output</p>
                      <p className="font-serif text-2xl leading-relaxed text-ink">
                        {stagedVerse?.text ?? liveVerse?.text ?? 'Preview appears here after presenting.'}
                      </p>
                    </div>
                  )}
                </div>

                <div className="grid gap-2">
                  {engine.chapterInfo?.verses.map(item => (
                    <button
                      key={item.id}
                      onClick={() => stageVerse(item)}
                      onDoubleClick={() => pushVerse(item)}
                      className={[
                        'grid grid-cols-[44px_1fr] items-start gap-3 rounded-xl border px-3 py-2 text-left transition-all',
                        liveVerse?.id === item.id
                          ? 'border-live bg-live/10'
                          : stagedVerse?.id === item.id
                          ? 'border-accent bg-accent/10'
                          : 'border-bdr bg-surface-800 hover:border-accent/40',
                      ].join(' ')}
                    >
                      <span className={[
                        'rounded-lg py-1 text-center text-xs font-black',
                        liveVerse?.id === item.id ? 'bg-live text-surface-950' : stagedVerse?.id === item.id ? 'bg-accent text-surface-950' : 'bg-surface-900 text-ink-faint',
                      ].join(' ')}>
                        {item.verse}
                      </span>
                      <span className="line-clamp-2 font-serif text-sm leading-relaxed text-ink-muted">{item.text}</span>
                    </button>
                  ))}
                </div>
              </div>
            </section>
          </div>
          )}

          {mode === 'Reading' && (
            <section className="min-h-0 flex-1 overflow-y-auto px-8 py-6">
              <div className="mx-auto max-w-3xl">
                <div className="mb-5 flex items-center justify-between">
                  <div>
                    <p className="text-2xs font-black uppercase tracking-wider text-accent">Reading Mode</p>
                    <h2 className="text-2xl font-black text-ink">
                      {engine.chapterInfo?.book_name ?? engine.selectedBook?.name ?? 'Bible'} {engine.chapter}
                    </h2>
                    <p className="text-sm text-ink-faint">Read, compare, bookmark, and stage without touching live output.</p>
                  </div>
                  {stagedVerse && (
                    <button onClick={() => setMode('Live')} className="rounded-lg border border-bdr bg-surface-800 px-3 py-2 text-xs font-bold text-accent">
                      Stage {refOf(stagedVerse)} for Live
                    </button>
                  )}
                </div>
                <div className="space-y-1">
                  {engine.chapterInfo?.verses.map(item => (
                    <button
                      key={item.id}
                      onClick={() => {
                        engine.selectVerse(item.verse);
                        setStagedVerse(item);
                      }}
                      className={[
                        'block w-full rounded-xl border px-4 py-3 text-left transition-all',
                        engine.verse === item.verse ? 'border-accent bg-accent/10' : 'border-transparent hover:border-bdr hover:bg-surface-800',
                      ].join(' ')}
                    >
                      <span className="mr-3 align-super text-xs font-black text-accent">{item.verse}</span>
                      <span className="font-serif text-[17px] leading-[1.85] text-ink">{item.text}</span>
                    </button>
                  ))}
                </div>
              </div>
            </section>
          )}

          {mode === 'Planning' && (
            <section className="min-h-0 flex-1 overflow-y-auto p-6">
              <div className="mx-auto max-w-5xl">
                <div className="mb-5">
                  <p className="text-2xs font-black uppercase tracking-wider text-accent">Planning Mode</p>
                  <h2 className="text-2xl font-black text-ink">Service plan</h2>
                  <p className="mt-1 text-sm text-ink-faint">Prepare scriptures, songs, countdowns, presentations, and announcements before going live.</p>
                </div>
                <div className="mb-4 flex items-center justify-between">
                  <p className="text-sm font-bold text-ink">{servicePlan?.name ?? 'Service plan'} — {servicePlan?.items.length ?? 0} items</p>
                  <button
                    onClick={() => clearServicePlan().then(() => setStatus('Service plan cleared'))}
                    className="rounded border border-bdr px-3 py-1 text-2xs font-bold text-ink-faint hover:text-danger"
                  >
                    Clear plan
                  </button>
                </div>
                <div className="space-y-2">
                  {(servicePlan?.items ?? []).map((item, index) => (
                    <div key={item.id} className="flex items-center gap-3 rounded-xl border border-bdr bg-surface-800 px-4 py-3">
                      <span className="flex size-7 items-center justify-center rounded-full bg-surface-700 text-2xs font-black text-ink-faint">{index + 1}</span>
                      <div className="min-w-0 flex-1">
                        <p className="text-sm font-bold text-ink">{planItemLabel(item)}</p>
                        <p className="text-2xs capitalize text-ink-faint">{item.kind.type}</p>
                      </div>
                      <button
                        onClick={() => removeServicePlanItem(item.id).then(() => setStatus('Removed from plan'))}
                        className="rounded p-1 text-ink-faint hover:text-danger"
                      >
                        <X size={14} />
                      </button>
                    </div>
                  ))}
                  {!servicePlan?.items.length && (
                    <p className="text-sm text-ink-faint">Add staged verses from the sidebar to build your service plan.</p>
                  )}
                </div>
              </div>
            </section>
          )}
        </main>

        <aside className="flex min-h-0 flex-col border-l border-bdr bg-surface-950">
          {mode === 'Live' && (
          <>
          <section className="border-b border-bdr p-3">
            <div className="mb-2 flex items-center gap-2">
              <span className={['size-2 rounded-full', connectedSource ? 'bg-live' : 'bg-surface-600'].join(' ')} />
              <p className="text-2xs font-black uppercase tracking-wider text-ink-faint">Presentation Source</p>
            </div>
            {connectedSource ? (
              <div className="flex gap-2">
                {presPreviewSrc && <img src={presPreviewSrc} alt="Presentation source" className="h-16 w-28 rounded border border-bdr object-cover" />}
                <div className="min-w-0 flex-1">
                  <p className="truncate text-xs font-bold text-ink">{connectedSource}</p>
                  <p className="text-2xs text-ink-faint">Scripture overlays this source.</p>
                  <button onClick={disconnectSource} className="mt-1 text-2xs font-bold text-danger">Disconnect</button>
                </div>
              </div>
            ) : (
              <div className="space-y-2">
                <div className="flex gap-2">
                  <select value={selectedSource} onChange={event => setSelectedSource(event.target.value)} className="min-w-0 flex-1 rounded border border-bdr bg-surface-800 px-2 py-1 text-2xs text-ink">
                    <option value="">Select NDI source</option>
                    {ndiSources.map(source => <option key={source.name} value={source.name}>{source.name}</option>)}
                  </select>
                  <button onClick={scanSources} className="rounded border border-bdr bg-surface-800 p-1.5 text-ink-faint hover:text-ink">
                    <RefreshCw size={12} />
                  </button>
                </div>
                <button disabled={!selectedSource} onClick={connectSource} className="w-full rounded bg-accent px-3 py-1.5 text-2xs font-black text-surface-950 disabled:opacity-40">
                  Connect presentation source
                </button>
              </div>
            )}
            {sourceStatus && <p className="mt-2 text-2xs text-ink-faint">{sourceStatus}</p>}
          </section>

          <section className="border-b border-bdr p-3">
            <div className="mb-2 flex items-center justify-between">
              <p className="flex items-center gap-2 text-2xs font-black uppercase tracking-wider text-ink-faint">
                <ListPlus size={12} /> Sermon Flow
              </p>
              <button onClick={() => addToQueue()} className="rounded bg-surface-800 px-2 py-1 text-2xs font-bold text-accent">Add</button>
            </div>
            <div className="max-h-40 space-y-1 overflow-y-auto">
              {queue.map((item, index) => (
                <button
                  key={item.id}
                      onClick={() => stageVerse(item.verse)}
                  onDoubleClick={() => pushQueueItem(index)}
                  className={[
                    'block w-full rounded-lg border px-3 py-2 text-left transition-all',
                    index === queueIndex ? 'border-accent bg-accent/10 text-accent' : 'border-bdr bg-surface-800 text-ink-muted hover:text-ink',
                  ].join(' ')}
                >
                  <span className="text-xs font-bold">{index + 1}. {refOf(item.verse)}</span>
                </button>
              ))}
              {!queue.length && <p className="text-xs text-ink-faint">Add staged verses for prepared sermon flow.</p>}
            </div>
          </section>
          </>
          )}

          {mode === 'Reading' && (
            <section className="border-b border-bdr p-3">
              <p className="mb-2 text-2xs font-black uppercase tracking-wider text-ink-faint">Reading Tools</p>
              {stagedVerse ? (
                <div className="rounded-xl border border-bdr bg-surface-800 p-3">
                  <p className="text-xs font-black text-accent">{refOf(stagedVerse)}</p>
                  <p className="mt-2 text-xs leading-relaxed text-ink-muted">{stagedVerse.text}</p>
                  <div className="mt-3 grid grid-cols-2 gap-2">
                    <button className="rounded border border-bdr px-2 py-1.5 text-2xs font-bold text-ink-faint hover:text-ink">Compare</button>
                    <button
                      onClick={() => toggleBookmark(stagedVerse.book_id, stagedVerse.chapter, stagedVerse.verse, stagedVerse.book_name, stagedVerse.text)
                        .then(ok => setStatus(ok ? `Bookmarked ${refOf(stagedVerse)}` : `Removed bookmark`))}
                      className="rounded border border-bdr px-2 py-1.5 text-2xs font-bold text-ink-faint hover:text-ink"
                    >
                      Bookmark
                    </button>
                  </div>
                </div>
              ) : (
                <p className="text-xs text-ink-faint">Select a verse to compare, bookmark, or stage for live mode.</p>
              )}
            </section>
          )}

          {mode === 'Planning' && (
            <section className="border-b border-bdr p-3">
              <div className="mb-2 flex items-center justify-between">
                <p className="flex items-center gap-2 text-2xs font-black uppercase tracking-wider text-ink-faint">
                  <ListPlus size={12} /> Scripture Queue
                </p>
                <div className="flex gap-1">
                  <button onClick={() => addToQueue()} className="rounded bg-surface-800 px-2 py-1 text-2xs font-bold text-accent">Queue</button>
                  <button onClick={() => addToServicePlan()} className="rounded bg-accent/15 px-2 py-1 text-2xs font-bold text-accent">Plan</button>
                </div>
              </div>
              <div className="max-h-52 space-y-1 overflow-y-auto">
                {queue.map((item, index) => (
                  <button key={item.id} onClick={() => stageVerse(item.verse)} className="block w-full rounded-lg border border-bdr bg-surface-800 px-3 py-2 text-left text-xs text-ink-muted hover:text-accent">
                    {index + 1}. {refOf(item.verse)}
                  </button>
                ))}
                {!queue.length && <p className="text-xs text-ink-faint">Add staged scriptures to prepare sermon flow.</p>}
              </div>
            </section>
          )}

          <section className="grid min-h-0 flex-1 grid-rows-3">
            <Panel title="Recent" icon={<Clock size={12} />}>
              {recent.map(item => (
                <button key={item.id} onClick={() => stageVerse(item)} onDoubleClick={() => pushVerse(item)} className="mb-1 block w-full rounded bg-surface-800 px-2 py-1.5 text-left text-xs text-ink-muted hover:text-accent">
                  {refOf(item)}
                </button>
              ))}
            </Panel>
            <Panel title="History" icon={<History size={12} />}>
              {history.map(item => (
                <button key={item.id} onClick={() => stageVerse(item.verse)} className="mb-1 block w-full rounded bg-surface-800 px-2 py-1.5 text-left text-xs text-ink-muted hover:text-accent">
                  <span className="text-ink-faint">{item.time}</span> {refOf(item.verse)}
                </button>
              ))}
            </Panel>
            <Panel title="Favorites" icon={<Star size={12} />}>
              {favorites.map(item => (
                <button key={item.id} onClick={() => stageVerse(item)} onDoubleClick={() => pushVerse(item)} className="mb-1 block w-full rounded bg-surface-800 px-2 py-1.5 text-left text-xs text-ink-muted hover:text-accent">
                  {refOf(item)}
                </button>
              ))}
              {stagedVerse && (
                <button onClick={() => setFavorites(current => [stagedVerse, ...current.filter(item => item.id !== stagedVerse.id)])} className="mt-2 w-full rounded border border-bdr px-2 py-1 text-2xs font-bold text-ink-faint hover:text-accent">
                  Star staged verse
                </button>
              )}
            </Panel>
          </section>
        </aside>
      </div>

      {mode === 'Live' && (
      <footer className="flex items-center gap-2 border-t border-bdr bg-surface-950 px-4 py-2">
        <button disabled={!stagedVerse} onClick={() => stagedVerse && pushVerse(stagedVerse)} className="flex items-center gap-2 rounded-lg bg-accent px-5 py-2 text-xs font-black text-surface-950 disabled:opacity-40">
          <Play size={13} fill="currentColor" /> Present staged
        </button>
        <button onClick={clearLive} className="flex items-center gap-2 rounded-lg border border-bdr bg-surface-800 px-4 py-2 text-xs font-bold text-ink-faint hover:text-accent">
          <X size={13} /> {connectedSource ? 'Release to Presentation' : 'Clear'}
        </button>
        <span className="text-xs text-ink-faint">{status}</span>
        {production.snapshot && (
          <span className="rounded-full bg-surface-800 px-2 py-1 text-2xs font-bold text-accent">
            Layer: {production.snapshot.active_layer} · {production.snapshot.scripture_mode}
            {production.snapshot.countdown?.status === 'running' && (
              <> · {Math.floor(production.snapshot.countdown.remaining_secs / 60)}:{String(production.snapshot.countdown.remaining_secs % 60).padStart(2, '0')}</>
            )}
          </span>
        )}
        <div className="ml-auto flex items-center gap-2 text-2xs text-ink-faint">
          <kbd className="rounded border border-bdr bg-surface-800 px-1.5 py-0.5">/</kbd> Search
          <kbd className="rounded border border-bdr bg-surface-800 px-1.5 py-0.5">Space</kbd> Push
          <kbd className="rounded border border-bdr bg-surface-800 px-1.5 py-0.5">Esc</kbd> Clear
        </div>
      </footer>
      )}
    </div>
  );
}

function Panel({ title, icon, children }: { title: string; icon: React.ReactNode; children: React.ReactNode }) {
  return (
    <div className="min-h-0 border-b border-bdr p-3">
      <p className="mb-2 flex items-center gap-2 text-2xs font-black uppercase tracking-wider text-ink-faint">
        {icon} {title}
      </p>
      <div className="max-h-full overflow-y-auto">{children}</div>
    </div>
  );
}
