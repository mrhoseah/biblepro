import { useState, useEffect, useCallback, useRef } from 'react';
import { Plus, X, Play, ChevronLeft, ChevronRight, Zap, ArrowUp, SkipForward, Radio, RotateCw } from 'lucide-react';
import {
  getTranslations, getBooks, getChapter, searchByReference,
  pushToAll, clearAll, getOutputs,
  listNdiSources, connectPresentationSource,
  disconnectPresentationSource, getPresentationPreview,
} from '../lib/commands';
import type { Translation, Book, ChapterInfo, Verse, QueueItem, OutputInfo, NdiSourceInfo } from '../lib/types';
import { nanoid } from '../lib/utils';

// ── helpers ────────────────────────────────────────────────────────────────────

function verseToItem(v: Verse): QueueItem {
  return {
    id: nanoid(),
    type: 'verse',
    title: `${v.book_name} ${v.chapter}:${v.verse}`,
    subtitle: v.translation_id.toUpperCase(),
    text: v.text,
    book_id: v.book_id,
    book_name: v.book_name,
    chapter: v.chapter,
    verse: v.verse,
    translation_id: v.translation_id,
  };
}

function fmtSecs(s: number) {
  if (s < 60) return `${s}s`;
  return `${Math.floor(s / 60)}m${s % 60}s`;
}

// ── Present ────────────────────────────────────────────────────────────────────

export default function Present() {
  // Translations & books
  const [translations, setTranslations] = useState<Translation[]>([]);
  const [books, setBooks] = useState<Book[]>([]);
  const [activeTrans, setActiveTrans] = useState('');

  // Service queue
  const [queue, setQueue] = useState<QueueItem[]>([]);
  const [queueIdx, setQueueIdx] = useState(0);
  const [history, setHistory] = useState<QueueItem[]>([]);

  // LIVE state
  const [liveVerse, setLiveVerse] = useState<Verse | null>(null);
  const [liveRef, setLiveRef] = useState('');
  const [previewSrc, setPreviewSrc] = useState('');
  const [status, setStatus] = useState('');
  const [loading, setLoading] = useState(false);

  // Live timer — how long current verse has been on screen
  const [liveSeconds, setLiveSeconds] = useState(0);

  // STAGED state — keyboard cursor, ready to push
  const [stagedVerse, setStagedVerse] = useState<Verse | null>(null);

  // Chapter browser
  const [browseBookId, setBrowseBookId] = useState(1);
  const [browseChapter, setBrowseChapter] = useState(1);
  const [chapterData, setChapterData] = useState<ChapterInfo | null>(null);

  // Jump bar
  const [jumpQuery, setJumpQuery] = useState('');
  const [jumpMsg, setJumpMsg] = useState('');
  const [jumpSugs, setJumpSugs] = useState<string[]>([]);
  const [jumpSugIdx, setJumpSugIdx] = useState(-1);
  const [showJumpSugs, setShowJumpSugs] = useState(false);

  // Left panel add-to-queue
  const [addQuery, setAddQuery] = useState('');
  const [addMsg, setAddMsg] = useState('');
  const [addSugs, setAddSugs] = useState<string[]>([]);
  const [addSugIdx, setAddSugIdx] = useState(-1);
  const [showAddSugs, setShowAddSugs] = useState(false);

  // Outputs
  const [outputs, setOutputs] = useState<OutputInfo[]>([]);

  // Presentation source (NDI input)
  const [ndiSources, setNdiSources] = useState<NdiSourceInfo[]>([]);
  const [selectedSource, setSelectedSource] = useState('');
  const [connectedSource, setConnectedSource] = useState<string | null>(null);
  const [presPreviewSrc, setPresPreviewSrc] = useState('');
  const [scanLoading, setScanLoading] = useState(false);
  const [connectLoading, setConnectLoading] = useState(false);

  // Refs
  const jumpInputRef = useRef<HTMLInputElement>(null);

  // ── Load ──────────────────────────────────────────────────────────────────────

  useEffect(() => {
    Promise.all([getTranslations(), getBooks(), getOutputs()]).then(([ts, bks, outs]) => {
      setTranslations(ts);
      setBooks(bks);
      setOutputs(outs);
      if (ts.length) setActiveTrans(ts[0].id);
    });
  }, []);

  // ── Live timer ────────────────────────────────────────────────────────────────

  useEffect(() => {
    if (!liveVerse) { setLiveSeconds(0); return; }
    setLiveSeconds(0);
    const iv = setInterval(() => setLiveSeconds(s => s + 1), 1000);
    return () => clearInterval(iv);
  }, [liveVerse]);

  // ── Presentation source thumbnail polling ─────────────────────────────────────

  useEffect(() => {
    if (!connectedSource) { setPresPreviewSrc(''); return; }
    const poll = () => {
      getPresentationPreview().then(b64 => {
        if (b64) setPresPreviewSrc(`data:image/png;base64,${b64}`);
      }).catch(() => {});
    };
    poll();
    const iv = setInterval(poll, 500);
    return () => clearInterval(iv);
  }, [connectedSource]);

  // ── Presentation source handlers ──────────────────────────────────────────────

  const handleScan = useCallback(async () => {
    setScanLoading(true);
    try {
      const sources = await listNdiSources();
      setNdiSources(sources);
      if (sources.length > 0 && !selectedSource) setSelectedSource(sources[0].name);
    } catch (e: any) {
      setNdiSources([]);
    } finally {
      setScanLoading(false);
    }
  }, [selectedSource]);

  const handleConnect = useCallback(async () => {
    if (!selectedSource) return;
    setConnectLoading(true);
    try {
      await connectPresentationSource(selectedSource);
      setConnectedSource(selectedSource);
    } catch (e: any) {
      setConnectedSource(null);
    } finally {
      setConnectLoading(false);
    }
  }, [selectedSource]);

  const handleDisconnect = useCallback(async () => {
    await disconnectPresentationSource();
    setConnectedSource(null);
    setPresPreviewSrc('');
  }, []);

  // ── Chapter browser ───────────────────────────────────────────────────────────

  useEffect(() => {
    if (!activeTrans || !browseBookId) return;
    setChapterData(null);
    getChapter(activeTrans, browseBookId, browseChapter)
      .then(info => setChapterData(info))
      .catch(() => {});
  }, [activeTrans, browseBookId, browseChapter]);

  // Auto-scroll staged verse into view
  useEffect(() => {
    if (!stagedVerse) return;
    const el = document.getElementById(`bvr-${stagedVerse.verse}`);
    if (el) el.scrollIntoView({ block: 'center', behavior: 'smooth' });
  }, [stagedVerse]);

  // ── Autocomplete ──────────────────────────────────────────────────────────────

  useEffect(() => {
    const q = jumpQuery.trim().toLowerCase();
    if (q.length < 2) { setJumpSugs([]); setShowJumpSugs(false); return; }
    const s = books.filter(b => b.name.toLowerCase().startsWith(q) || b.short_name.toLowerCase().startsWith(q)).slice(0, 8).map(b => b.name);
    setJumpSugs(s); setShowJumpSugs(s.length > 0); setJumpSugIdx(-1);
  }, [jumpQuery, books]);

  useEffect(() => {
    const q = addQuery.trim().toLowerCase();
    if (q.length < 2) { setAddSugs([]); setShowAddSugs(false); return; }
    const s = books.filter(b => b.name.toLowerCase().startsWith(q) || b.short_name.toLowerCase().startsWith(q)).slice(0, 6).map(b => b.name);
    setAddSugs(s); setShowAddSugs(s.length > 0); setAddSugIdx(-1);
  }, [addQuery, books]);

  // ── Core push — auto-stages next verse after push ─────────────────────────────
  // This is the key UX advantage: after pushing, the operator just presses Space
  // repeatedly to follow a fast-reading preacher verse by verse.

  const pushVerseLive = useCallback(async (v: Verse, autoAdvance = true) => {
    const ref = `${v.book_name} ${v.chapter}:${v.verse}`;
    setLoading(true);
    try {
      const p = await pushToAll(v.text, ref);
      setPreviewSrc(`data:image/png;base64,${p.png_b64}`);
      setLiveVerse(v);
      setLiveRef(ref);
      setStatus('Live');
      setHistory(prev =>
        [verseToItem(v), ...prev.filter(h => !(h.verse === v.verse && h.chapter === v.chapter && h.book_id === v.book_id))].slice(0, 20)
      );
      // Auto-stage next verse — the ProPresenter killer feature
      if (autoAdvance) {
        setChapterData(current => {
          if (current) {
            const pos = current.verses.findIndex(vv => vv.verse === v.verse);
            if (pos >= 0 && pos < current.verses.length - 1) {
              setStagedVerse(current.verses[pos + 1]);
            } else {
              setStagedVerse(null); // end of chapter
            }
          }
          return current;
        });
      }
      // Sync browser to pushed verse
      setBrowseBookId(v.book_id);
      setBrowseChapter(v.chapter);
    } catch (e: any) {
      setStatus(e?.toString() ?? 'Send failed');
    } finally {
      setLoading(false);
    }
  }, []);

  // ── Push staged ───────────────────────────────────────────────────────────────

  const pushStaged = useCallback(async () => {
    if (!stagedVerse) return;
    await pushVerseLive(stagedVerse);
  }, [stagedVerse, pushVerseLive]);

  // ── Stage without push ────────────────────────────────────────────────────────

  const stageVerse = useCallback((v: Verse) => {
    setStagedVerse(v);
  }, []);

  const stagePrev = useCallback(() => {
    if (!chapterData) return;
    const vers = chapterData.verses;
    const curr = stagedVerse?.verse;
    const pos = curr != null ? vers.findIndex(v => v.verse === curr) : 0;
    if (pos > 0) setStagedVerse(vers[pos - 1]);
  }, [chapterData, stagedVerse]);

  const stageNext = useCallback(() => {
    if (!chapterData) return;
    const vers = chapterData.verses;
    const curr = stagedVerse?.verse;
    const pos = curr != null ? vers.findIndex(v => v.verse === curr) : -1;
    if (pos < vers.length - 1) setStagedVerse(vers[pos + 1]);
  }, [chapterData, stagedVerse]);

  // ── Jump bar ──────────────────────────────────────────────────────────────────

  const doJump = useCallback(async () => {
    const q = jumpQuery.trim();
    if (!q || !activeTrans) return;
    setJumpMsg('…'); setShowJumpSugs(false);
    try {
      const v = await searchByReference(activeTrans, q);
      if (!v) { setJumpMsg('Not found.'); return; }
      await pushVerseLive(v);
      setJumpQuery(''); setJumpMsg('');
    } catch { setJumpMsg('Error.'); }
  }, [jumpQuery, activeTrans, pushVerseLive]);

  // ── Add to service flow ───────────────────────────────────────────────────────

  const doAdd = useCallback(async () => {
    const q = addQuery.trim();
    if (!q || !activeTrans) return;
    setAddMsg('');
    try {
      const v = await searchByReference(activeTrans, q);
      if (!v) { setAddMsg('Not found.'); return; }
      const item = verseToItem(v);
      setQueue(prev => [...prev, item]);
      setAddQuery(''); setAddMsg(`✓ ${item.title}`); setShowAddSugs(false);
      setBrowseBookId(v.book_id); setBrowseChapter(v.chapter);
    } catch { setAddMsg('Error.'); }
  }, [addQuery, activeTrans]);

  // ── Queue navigation ──────────────────────────────────────────────────────────

  const loadQueueItem = useCallback((i: number) => {
    setQueueIdx(i);
    const item = queue[i];
    if (!item?.book_id) return;
    setBrowseBookId(item.book_id); setBrowseChapter(item.chapter!);
  }, [queue]);

  const goLive = useCallback(async (i: number) => {
    const item = queue[i];
    if (!item?.book_id) return;
    setQueueIdx(i);
    const info = await getChapter(activeTrans, item.book_id, item.chapter!);
    const v = info?.verses.find(vv => vv.verse === item.verse);
    if (v) await pushVerseLive(v);
  }, [queue, activeTrans, pushVerseLive]);

  // ── Keyboard shortcuts ────────────────────────────────────────────────────────

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const tag = (e.target as HTMLElement).tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;
      if (e.key === ' ')                               { e.preventDefault(); pushStaged(); }
      else if (e.key === 'ArrowDown' || e.key === 'ArrowRight') { e.preventDefault(); stageNext(); }
      else if (e.key === 'ArrowUp'   || e.key === 'ArrowLeft')  { e.preventDefault(); stagePrev(); }
      else if (e.key === ']') { setBrowseChapter(c => chapterData ? Math.min(c + 1, chapterData.total_chapters) : c); setStagedVerse(null); }
      else if (e.key === '[') { setBrowseChapter(c => Math.max(c - 1, 1)); setStagedVerse(null); }
      else if (e.key === '.') loadQueueItem(Math.min(queueIdx + 1, queue.length - 1));
      else if (e.key === ',') loadQueueItem(Math.max(queueIdx - 1, 0));
      else if (e.key === 'c' || e.key === 'C') clearAll().then(r => { setPreviewSrc(r.png_b64 ? `data:image/png;base64,${r.png_b64}` : ''); setLiveVerse(null); setLiveRef(''); setStatus(''); setStagedVerse(null); });
      else if (e.key === '/') { e.preventDefault(); jumpInputRef.current?.focus(); jumpInputRef.current?.select(); }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [pushStaged, stageNext, stagePrev, loadQueueItem, queueIdx, queue.length, chapterData]);

  // ── Derived ───────────────────────────────────────────────────────────────────

  const liveOutputCount = outputs.filter(o => o.enabled).length;
  const nextQueueItem = queue[queueIdx + 1] ?? null;
  const hasStaged = stagedVerse != null &&
    !(stagedVerse.verse === liveVerse?.verse && stagedVerse.chapter === liveVerse?.chapter && stagedVerse.book_id === liveVerse?.book_id);

  // ── Render ────────────────────────────────────────────────────────────────────

  return (
    <div className="flex h-full overflow-hidden bg-surface-900" onContextMenu={e => e.preventDefault()}>

      {/* ════ LEFT: Service Flow ═════════════════════════════════════════════ */}
      <aside className="w-56 flex flex-col h-full bg-surface-950 border-r border-bdr shrink-0">

        {/* Live status bar */}
        <div className={['flex items-center gap-2 px-3 py-2 border-b border-bdr shrink-0', liveRef ? 'bg-live/5' : ''].join(' ')}>
          <span className={liveRef ? 'live-dot' : 'w-2 h-2 rounded-full bg-surface-600'} />
          <span className={['text-2xs font-bold truncate', liveRef ? 'text-live' : 'text-ink-faint'].join(' ')}>
            {liveRef || 'STANDBY'}
          </span>
          {liveRef && liveSeconds > 0 && (
            <span className="ml-auto text-2xs text-ink-faint font-mono shrink-0">{fmtSecs(liveSeconds)}</span>
          )}
          {liveOutputCount > 0 && !liveRef && (
            <span className="ml-auto text-2xs text-ink-faint">{liveOutputCount}×</span>
          )}
        </div>

        {/* Add to service */}
        <div className="px-2.5 pt-2.5 pb-2 border-b border-bdr shrink-0">
          <div className="text-2xs font-bold text-ink-faint uppercase tracking-wider mb-1.5">Add to Service</div>
          <div className="relative">
            <div className="flex gap-1">
              <input
                value={addQuery}
                onChange={e => { setAddQuery(e.target.value); setAddMsg(''); }}
                onBlur={() => setTimeout(() => setShowAddSugs(false), 150)}
                onKeyDown={e => {
                  if (e.key === 'ArrowDown') { setAddSugIdx(i => Math.min(i + 1, addSugs.length - 1)); e.preventDefault(); }
                  if (e.key === 'ArrowUp') { setAddSugIdx(i => Math.max(i - 1, -1)); e.preventDefault(); }
                  if (e.key === 'Enter') {
                    if (addSugIdx >= 0 && addSugs[addSugIdx]) { setAddQuery(addSugs[addSugIdx] + ' '); setShowAddSugs(false); setAddSugIdx(-1); }
                    else doAdd();
                  }
                  if (e.key === 'Escape') setShowAddSugs(false);
                }}
                placeholder="John 3:16…"
                className="flex-1 min-w-0 bg-surface-700 border border-bdr rounded px-2 py-1.5 text-xs text-ink placeholder:text-ink-faint focus:outline-none focus:border-accent"
              />
              <button onClick={doAdd} className="px-2 py-1.5 bg-accent text-surface-950 rounded text-xs font-black hover:brightness-110 transition-all shrink-0" title="Add to service"><Plus size={11} /></button>
            </div>
            {addMsg && <div className={['mt-1 text-2xs', addMsg.startsWith('✓') ? 'text-live' : 'text-ink-faint'].join(' ')}>{addMsg}</div>}
            {showAddSugs && addSugs.length > 0 && (
              <div className="absolute top-full left-0 right-8 mt-0.5 bg-surface-600 border border-bdr rounded shadow-panel z-50">
                {addSugs.map((s, i) => (
                  <button key={s} className={['w-full text-left px-2 py-1.5 text-xs transition-colors', i === addSugIdx ? 'bg-surface-500 text-accent' : 'text-ink hover:bg-surface-500'].join(' ')}
                    onMouseDown={() => { setAddQuery(s + ' '); setShowAddSugs(false); setAddSugIdx(-1); }}>{s}</button>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* Translation chips */}
        <div className="flex items-center gap-1 px-2.5 py-1.5 border-b border-bdr shrink-0 flex-wrap">
          {translations.map(t => (
            <button key={t.id} onClick={() => setActiveTrans(t.id)} title={`${t.name} · ${t.language}`}
              className={['px-2 py-0.5 rounded text-2xs font-bold border transition-all', activeTrans === t.id ? 'bg-accent text-surface-950 border-accent' : 'border-bdr text-ink-faint hover:text-ink hover:border-bdr-strong'].join(' ')}>
              {t.abbreviation}
            </button>
          ))}
          {!translations.length && <span className="text-2xs text-ink-faint italic">Import translations in Library</span>}
        </div>

        {/* Service queue */}
        <div className="flex-1 overflow-y-auto py-1">
          {queue.length === 0 && (
            <div className="px-3 py-4 text-2xs text-ink-faint text-center leading-relaxed">Add verses above to build your service.</div>
          )}
          {queue.map((item, i) => {
            const isActive = i === queueIdx;
            const isLive = liveVerse != null && item.book_id === liveVerse.book_id && item.chapter === liveVerse.chapter && item.verse === liveVerse.verse;
            return (
              <button key={item.id} onClick={() => loadQueueItem(i)} onDoubleClick={() => goLive(i)}
                className={['w-full text-left px-2.5 py-2 border-l-2 transition-all group relative', isLive ? 'border-live bg-live/10 text-ink' : isActive ? 'border-accent bg-accent-glow text-ink' : 'border-transparent text-ink-faint hover:text-ink hover:bg-surface-800'].join(' ')}>
                <div className="flex items-start gap-1.5">
                  <span className="text-2xs font-mono text-ink-faint w-4 shrink-0 mt-0.5">{i + 1}</span>
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-1">
                      <span className="text-xs font-semibold truncate">{item.title}</span>
                      {isLive && <span className="text-2xs font-black text-live shrink-0">●</span>}
                    </div>
                    <div className="text-2xs text-ink-faint truncate mt-0.5">{item.text.slice(0, 38)}…</div>
                  </div>
                </div>
                <button onClick={e => { e.stopPropagation(); setQueue(q => q.filter((_, j) => j !== i)); if (queueIdx >= i && queueIdx > 0) setQueueIdx(q => q - 1); }}
                  className="absolute right-1 top-1/2 -translate-y-1/2 p-0.5 rounded opacity-0 group-hover:opacity-100 hover:bg-danger/20 hover:text-danger transition-all text-ink-faint">
                  <X size={9} />
                </button>
              </button>
            );
          })}
        </div>

        {/* Queue nav */}
        {queue.length > 0 && (
          <div className="flex items-center gap-1 px-2.5 py-1.5 border-t border-bdr shrink-0">
            <button onClick={() => loadQueueItem(Math.max(queueIdx - 1, 0))} disabled={queueIdx === 0}
              className="p-1 rounded border border-bdr text-ink-faint hover:text-ink hover:bg-surface-700 disabled:opacity-30 transition-all">
              <ChevronLeft size={12} />
            </button>
            <span className="flex-1 text-center text-2xs text-ink-faint">{queueIdx + 1}/{queue.length}</span>
            <button onClick={() => loadQueueItem(Math.min(queueIdx + 1, queue.length - 1))} disabled={queueIdx >= queue.length - 1}
              className="p-1 rounded border border-bdr text-ink-faint hover:text-ink hover:bg-surface-700 disabled:opacity-30 transition-all">
              <ChevronRight size={12} />
            </button>
          </div>
        )}

        {/* History */}
        {history.length > 0 && (
          <div className="border-t border-bdr max-h-32 overflow-y-auto shrink-0">
            <div className="px-2.5 py-1 text-2xs font-bold text-ink-faint uppercase tracking-wider">History</div>
            {history.map((item, i) => (
              <button key={`h${i}`}
                onClick={async () => {
                  if (!item.book_id) return;
                  const info = await getChapter(activeTrans, item.book_id, item.chapter!);
                  const v = info?.verses.find(vv => vv.verse === item.verse);
                  if (v) pushVerseLive(v, false);
                }}
                className="w-full text-left px-2.5 py-1.5 border-b border-bdr/30 hover:bg-surface-800 transition-colors flex items-center gap-2 group">
                <div className="min-w-0 flex-1">
                  <div className="text-2xs font-semibold text-accent truncate">{item.title}</div>
                  <div className="text-2xs text-ink-faint truncate">{item.text.slice(0, 32)}…</div>
                </div>
                <ArrowUp size={9} className="shrink-0 text-ink-faint group-hover:text-accent" />
              </button>
            ))}
          </div>
        )}
      </aside>

      {/* ════ CENTER: Jump Bar + Verse Browser ═══════════════════════════════ */}
      <div className="flex-1 flex flex-col overflow-hidden min-w-0">

        {/* ── Jump Bar — always at top, always prominent ────────────────────── */}
        <div className="shrink-0 px-3 py-2 border-b border-bdr bg-surface-800/40">
          <div className="relative flex items-center gap-2">
            <Zap size={14} className="text-accent shrink-0" />
            <div className="relative flex-1">
              <input
                ref={jumpInputRef}
                value={jumpQuery}
                onChange={e => { setJumpQuery(e.target.value); setJumpMsg(''); }}
                onBlur={() => setTimeout(() => setShowJumpSugs(false), 150)}
                onKeyDown={e => {
                  if (e.key === 'ArrowDown') { setJumpSugIdx(i => Math.min(i + 1, jumpSugs.length - 1)); e.preventDefault(); }
                  if (e.key === 'ArrowUp') { setJumpSugIdx(i => Math.max(i - 1, -1)); e.preventDefault(); }
                  if (e.key === 'Enter') {
                    if (jumpSugIdx >= 0 && jumpSugs[jumpSugIdx]) { setJumpQuery(jumpSugs[jumpSugIdx] + ' '); setShowJumpSugs(false); setJumpSugIdx(-1); }
                    else doJump();
                  }
                  if (e.key === 'Escape') { setShowJumpSugs(false); setJumpQuery(''); setJumpMsg(''); }
                }}
                placeholder="Jump live: John 3:16, Psalm 23…  Enter to send  (/ to focus)"
                className="w-full bg-surface-700 border border-bdr rounded-md px-3 py-2 text-sm text-ink placeholder:text-ink-faint focus:outline-none focus:border-accent focus:shadow-glow transition-all"
              />
              {jumpMsg && <span className="absolute right-3 top-1/2 -translate-y-1/2 text-2xs text-ink-faint pointer-events-none">{jumpMsg}</span>}
              {showJumpSugs && jumpSugs.length > 0 && (
                <div className="absolute top-full left-0 right-0 mt-0.5 bg-surface-700 border border-bdr rounded-md shadow-panel z-50">
                  {jumpSugs.map((s, i) => (
                    <button key={s} className={['w-full text-left px-3 py-2 text-xs transition-colors', i === jumpSugIdx ? 'bg-surface-600 text-accent' : 'text-ink hover:bg-surface-600'].join(' ')}
                      onMouseDown={() => { setJumpQuery(s + ' '); setShowJumpSugs(false); setJumpSugIdx(-1); }}>{s}</button>
                  ))}
                </div>
              )}
            </div>
            <button onClick={doJump} disabled={loading || !jumpQuery.trim()}
              className="flex items-center gap-1.5 px-4 py-2 bg-accent text-surface-950 rounded-md text-xs font-black hover:brightness-110 disabled:opacity-40 transition-all shrink-0">
              <Play size={11} fill="currentColor" /> Go Live
            </button>
          </div>
        </div>

        {/* ── Book + Chapter navigator ──────────────────────────────────────── */}
        <div className="flex items-center gap-2 px-3 py-1.5 border-b border-bdr bg-surface-950/60 shrink-0 overflow-hidden">
          <select value={browseBookId} onChange={e => { setBrowseBookId(Number(e.target.value)); setBrowseChapter(1); setStagedVerse(null); }}
            className="bg-surface-700 border border-bdr rounded px-2 py-1 text-xs text-ink focus:outline-none focus:border-accent shrink-0 max-w-[140px]">
            <optgroup label="Old Testament">
              {books.filter(b => b.testament === 'OT').map(b => <option key={b.id} value={b.id}>{b.name}</option>)}
            </optgroup>
            <optgroup label="New Testament">
              {books.filter(b => b.testament === 'NT').map(b => <option key={b.id} value={b.id}>{b.name}</option>)}
            </optgroup>
          </select>
          {chapterData && (
            <>
              <div className="flex-1 flex gap-1 overflow-x-auto pb-0.5">
                {Array.from({ length: chapterData.total_chapters }, (_, i) => i + 1).map(ch => (
                  <button key={ch} onClick={() => { setBrowseChapter(ch); setStagedVerse(null); }}
                    className={['px-2 py-0.5 rounded text-2xs font-bold shrink-0 transition-all', ch === browseChapter ? 'bg-accent text-surface-950' : 'bg-surface-700 text-ink-faint hover:text-ink hover:bg-surface-600'].join(' ')}>{ch}</button>
                ))}
              </div>
              <span className="text-2xs text-ink-faint font-mono shrink-0 ml-1">
                {chapterData.verses.length}v
              </span>
            </>
          )}
        </div>

        {/* ── Verse Theater ─────────────────────────────────────────────────── */}
        <div className="flex-1 overflow-y-auto">
          {!chapterData ? (
            <div className="flex items-center justify-center h-full gap-2 text-ink-faint text-sm">
              <div className="w-4 h-4 border-2 border-bdr border-t-accent rounded-full animate-spin" /> Loading…
            </div>
          ) : (
            <div className="py-2">
              {chapterData.verses.map(v => {
                const isLive    = liveVerse?.verse === v.verse && liveVerse?.chapter === v.chapter && liveVerse?.book_id === v.book_id;
                const isStaged  = !isLive && stagedVerse?.verse === v.verse && stagedVerse?.chapter === v.chapter && stagedVerse?.book_id === v.book_id;
                const isNearLive = liveVerse?.chapter === v.chapter && liveVerse?.book_id === v.book_id && Math.abs((liveVerse?.verse ?? 0) - v.verse) === 1;
                return (
                  <div
                    key={v.verse}
                    id={`bvr-${v.verse}`}
                    onClick={() => pushVerseLive(v)}
                    onContextMenu={e => { e.preventDefault(); stageVerse(v); }}
                    className={[
                      'flex items-baseline gap-3 px-4 py-3.5 cursor-pointer transition-all border-l-[3px] select-none group',
                      isLive    ? 'border-accent bg-accent/10'
                      : isStaged  ? 'border-blue-500 bg-blue-600/10'
                      : isNearLive ? 'border-transparent bg-surface-800/40'
                      : 'border-transparent hover:bg-surface-800/60',
                    ].join(' ')}
                  >
                    <span className={['text-sm font-black w-7 shrink-0 text-right tabular-nums',
                      isLive ? 'text-accent' : isStaged ? 'text-blue-400' : 'text-ink-faint'].join(' ')}>{v.verse}</span>
                    <span className={['flex-1 font-serif text-[15px] leading-[1.75]',
                      isLive ? 'text-white font-semibold' : isStaged ? 'text-ink' : isNearLive ? 'text-ink-muted' : 'text-ink-muted group-hover:text-ink'].join(' ')}>{v.text}</span>
                    <div className="flex gap-1 shrink-0">
                      {isLive && (
                        <span className="text-2xs font-black tracking-widest text-accent bg-accent/15 border border-accent/30 rounded px-1.5 py-0.5">LIVE</span>
                      )}
                      {isStaged && (
                        <span className="text-2xs font-black tracking-widest text-blue-400 bg-blue-600/15 border border-blue-500/30 rounded px-1.5 py-0.5">NEXT</span>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* ── Keyboard guide ────────────────────────────────────────────────── */}
        <div className="flex items-center gap-2.5 px-4 py-1.5 border-t border-bdr bg-surface-950/80 shrink-0 flex-wrap">
          {[['Click','send live'],['Right-click','stage'],['Space','send next'],['↑↓','stage'],['[ ]','chapter'],['/','jump'],['C','clear']].map(([k, d]) => (
            <span key={k} className="flex items-center gap-1 text-2xs text-ink-faint">
              <kbd className="px-1 py-0.5 bg-surface-700 border border-bdr rounded font-mono text-2xs text-ink-muted">{k}</kbd>
              <span>{d}</span>
            </span>
          ))}
        </div>
      </div>

      {/* ════ RIGHT: Operator Monitor ════════════════════════════════════════ */}
      <aside className="w-72 flex flex-col h-full bg-surface-950 border-l border-bdr shrink-0">

        {/* Big push button */}
        <div className="px-3 pt-3 pb-2 border-b border-bdr shrink-0">
          <button onClick={() => liveVerse && pushVerseLive(liveVerse, false)} disabled={!liveVerse || loading}
            className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-accent text-surface-950 rounded-lg text-sm font-black hover:brightness-110 disabled:opacity-35 transition-all shadow-glow mb-2">
            <Play size={13} fill="currentColor" />
            {loading ? 'Sending…' : '▶  Send Live'}
          </button>
          <div className="flex gap-2">
            {nextQueueItem && (
              <button onClick={() => goLive(queueIdx + 1)}
                className="flex-1 flex items-center justify-center gap-1 px-2 py-1.5 bg-surface-700 border border-bdr text-ink rounded text-xs font-bold hover:bg-surface-600 transition-all">
                <SkipForward size={11} /> Advance Queue
              </button>
            )}
            <button
              onClick={() => clearAll().then(r => { setPreviewSrc(r.png_b64 ? `data:image/png;base64,${r.png_b64}` : ''); setLiveVerse(null); setLiveRef(''); setStatus(''); setStagedVerse(null); })}
              className={['flex items-center justify-center gap-1 px-3 py-1.5 border rounded text-xs font-bold transition-all',
                connectedSource
                  ? 'bg-surface-700 border-bdr text-accent hover:bg-accent/10 hover:border-accent'
                  : 'bg-surface-700 border-bdr text-ink-faint hover:text-danger hover:border-danger',
              ].join(' ')}
              title={connectedSource ? 'Release to presentation (C)' : 'Clear all outputs (C)'}>
              {connectedSource ? <><Radio size={12} /> Release</> : <><X size={12} /> Clear</>}
            </button>
          </div>
        </div>

        {/* Presentation Source */}
        <div className="px-3 py-2 border-b border-bdr shrink-0">
          <div className="flex items-center gap-1.5 mb-1.5">
            <span className={['w-1.5 h-1.5 rounded-full shrink-0', connectedSource ? 'bg-live shadow-[0_0_6px_rgba(74,222,128,.8)]' : 'bg-surface-600'].join(' ')} />
            <span className="text-2xs font-bold text-ink-faint uppercase tracking-wider flex-1">Presentation Source</span>
            {connectedSource && <span className="text-2xs font-black text-live">LIVE</span>}
          </div>

          {connectedSource ? (
            <div className="flex items-center gap-2">
              {presPreviewSrc && (
                <img src={presPreviewSrc} alt="Presentation" className="w-20 h-11 object-cover rounded border border-bdr shrink-0 bg-surface-800" />
              )}
              <div className="flex-1 min-w-0">
                <div className="text-2xs text-ink truncate font-medium" title={connectedSource}>{connectedSource}</div>
                <button onClick={handleDisconnect}
                  className="mt-1 text-2xs text-ink-faint hover:text-danger transition-colors flex items-center gap-1">
                  <X size={9} /> Disconnect
                </button>
              </div>
            </div>
          ) : (
            <div className="flex items-center gap-1.5 flex-wrap">
              {ndiSources.length > 0 ? (
                <select value={selectedSource} onChange={e => setSelectedSource(e.target.value)}
                  className="flex-1 min-w-0 bg-surface-700 border border-bdr rounded px-1.5 py-1 text-2xs text-ink focus:outline-none focus:border-accent">
                  {ndiSources.map(s => <option key={s.name} value={s.name}>{s.name}</option>)}
                </select>
              ) : (
                <span className="flex-1 text-2xs text-ink-faint italic">
                  {scanLoading ? 'Scanning…' : 'No sources found'}
                </span>
              )}
              {ndiSources.length > 0 && (
                <button onClick={handleConnect} disabled={!selectedSource || connectLoading}
                  className="px-2 py-1 bg-accent text-surface-950 rounded text-2xs font-black hover:brightness-110 disabled:opacity-40 transition-all shrink-0">
                  {connectLoading ? '…' : 'Connect'}
                </button>
              )}
              <button onClick={handleScan} disabled={scanLoading}
                className="p-1 rounded border border-bdr text-ink-faint hover:text-ink hover:bg-surface-700 disabled:opacity-40 transition-all shrink-0"
                title="Scan for NDI sources">
                <RotateCw size={10} className={scanLoading ? 'animate-spin' : ''} />
              </button>
            </div>
          )}
        </div>

        {/* NOW LIVE */}
        {liveVerse ? (
          <div className="border-b border-accent/20 px-3 py-3 shrink-0 bg-gradient-to-b from-accent/10 to-transparent">
            <div className="flex items-center gap-2 mb-1.5">
              <span className="live-dot" />
              <span className="text-2xs font-black tracking-widest text-live">NOW LIVE</span>
              {liveSeconds > 0 && <span className="text-2xs text-ink-faint font-mono">{fmtSecs(liveSeconds)}</span>}
              <button onClick={() => pushVerseLive(liveVerse, false)} disabled={loading}
                className="ml-auto text-2xs text-ink-faint hover:text-accent transition-colors flex items-center gap-1 disabled:opacity-40">
                <ArrowUp size={10} /> Resend
              </button>
            </div>
            <div className="text-sm font-bold text-accent mb-1 leading-tight">{liveRef}</div>
            <p className="text-xs text-ink leading-relaxed line-clamp-4">{liveVerse.text}</p>
          </div>
        ) : (
          <div className="border-b border-bdr px-3 py-4 shrink-0 flex items-center gap-2 text-ink-faint">
            <span className="w-2 h-2 rounded-full bg-surface-600" />
            <span className="text-xs">Nothing live — use Jump bar or click a verse</span>
          </div>
        )}

        {/* STAGED / NEXT */}
        {hasStaged && stagedVerse ? (
          <div className="border-b border-blue-900/40 px-3 py-3 shrink-0 bg-gradient-to-b from-blue-950/40 to-transparent">
            <div className="flex items-center gap-2 mb-1.5">
              <span className="w-2 h-2 rounded-full bg-blue-400 shadow-[0_0_8px_rgba(96,165,250,.6)]" />
              <span className="text-2xs font-black tracking-widest text-blue-400">STAGED — NEXT</span>
              <span className="ml-auto text-2xs text-ink-faint font-mono">Space →</span>
            </div>
            <div className="text-sm font-bold text-blue-300 mb-1 leading-tight">
              {stagedVerse.book_name} {stagedVerse.chapter}:{stagedVerse.verse}
            </div>
            <p className="text-xs text-ink-muted leading-relaxed line-clamp-3 mb-2">{stagedVerse.text}</p>
            <button onClick={pushStaged} disabled={loading}
              className="w-full px-3 py-1.5 bg-blue-700/30 border border-blue-500/40 text-blue-300 rounded text-xs font-black hover:bg-blue-700/50 hover:border-blue-400 transition-all disabled:opacity-40">
              ▶ Send Staged Live
            </button>
          </div>
        ) : liveVerse && !hasStaged ? (
          <div className="border-b border-bdr/40 px-3 py-2 shrink-0">
            <div className="text-2xs text-ink-faint italic">
              {stagedVerse ? 'Staged = live. Press ↓ to stage next.' : 'Press ↓ or right-click a verse to stage next.'}
            </div>
          </div>
        ) : null}

        {/* Up Next from queue */}
        {nextQueueItem && (
          <div className="border-b border-bdr/40 px-3 py-2.5 shrink-0">
            <div className="flex items-center justify-between mb-1">
              <span className="text-2xs font-black tracking-widest text-ink-faint">QUEUE NEXT</span>
              <button onClick={() => goLive(queueIdx + 1)}
                className="text-2xs text-ink-faint hover:text-accent transition-colors">Send ▶</button>
            </div>
            <div className="text-xs font-semibold text-ink-muted">{nextQueueItem.title}</div>
            <p className="text-2xs text-ink-faint mt-0.5 line-clamp-2">{nextQueueItem.text.slice(0, 70)}…</p>
          </div>
        )}

        {status && (
          <div className="mx-3 mt-2 px-2 py-1 bg-surface-700 border-l-2 border-accent rounded-r text-2xs text-ink-muted shrink-0">{status}</div>
        )}

        {/* Slide preview */}
        <div className="flex-1 flex flex-col px-3 pt-3 overflow-hidden min-h-0">
          <div className="text-2xs font-bold text-ink-faint uppercase tracking-wider mb-2 shrink-0">PREVIEW</div>
          <div className="slide-thumb shrink-0">
            {previewSrc
              ? <img src={previewSrc} alt="Current slide" className="w-full h-full object-contain" />
              : <div className="w-full h-full flex items-center justify-center"><span className="text-xs text-ink-faint">Preview appears here</span></div>
            }
          </div>
        </div>
      </aside>
    </div>
  );
}
