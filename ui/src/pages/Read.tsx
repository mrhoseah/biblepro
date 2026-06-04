import { useState, useEffect, useRef, useCallback } from 'react';
import { ChevronLeft, ChevronRight, BookMarked, Highlighter, GitCompare, Zap } from 'lucide-react';
import {
  getTranslations, getBooks, getChapter, searchByReference,
  setHighlight, removeHighlight, getChapterHighlights,
  toggleBookmark, isBookmarked,
} from '../lib/commands';
import type { Translation, Book, ChapterInfo, Verse, Highlight } from '../lib/types';
import { pushToAll } from '../lib/commands';

interface Props { onPresent: () => void }

const HIGHLIGHT_COLORS = ['#F6E37A', '#A8F5A0', '#A0C8FF', '#F5A0C8', '#F5C0A0'];

export default function Read({ onPresent }: Props) {
  // Navigation state
  const [translations, setTranslations] = useState<Translation[]>([]);
  const [books, setBooks] = useState<Book[]>([]);
  const [activeTrans, setActiveTrans] = useState('');
  const [compareTrans, setCompareTrans] = useState<string[]>([]);
  const [selectedBook, setSelectedBook] = useState<Book | null>(null);
  const [chapter, setChapter] = useState(1);
  const [chapterInfo, setChapterInfo] = useState<ChapterInfo | null>(null);

  // Study state
  const [highlights, setHighlights] = useState<Highlight[]>([]);
  const [activeVerse, setActiveVerse] = useState<number | null>(null);
  const [bookmarkedVerse, setBookmarkedVerse] = useState<number | null>(null);
  const [highlightColor, setHighlightColor] = useState(HIGHLIGHT_COLORS[0]);
  const [sidePanel, setSidePanel] = useState<'compare' | 'notes' | 'ai' | null>('compare');

  // Compare state
  const [compareData, setCompareData] = useState<{ trans: Translation; verse: Verse | null }[]>([]);
  const [compareVerse, setCompareVerse] = useState<number | null>(null);

  // Autocomplete
  const [bookSearch, setBookSearch] = useState('');
  const bookSuggestions = books.filter(b =>
    bookSearch.length > 0 &&
    (b.name.toLowerCase().startsWith(bookSearch.toLowerCase()) ||
     b.short_name.toLowerCase().startsWith(bookSearch.toLowerCase()))
  ).slice(0, 6);

  // Initial load
  useEffect(() => {
    Promise.all([getTranslations(), getBooks()]).then(([ts, bks]) => {
      setTranslations(ts);
      setBooks(bks);
      if (ts.length) setActiveTrans(ts[0].id);
      if (bks.length) {
        setSelectedBook(bks[0]);
        setChapter(1);
      }
    });
  }, []);

  // Load chapter
  useEffect(() => {
    if (!activeTrans || !selectedBook) return;
    getChapter(activeTrans, selectedBook.id, chapter).then(info => {
      setChapterInfo(info);
    });
    getChapterHighlights(selectedBook.id, chapter).then(setHighlights);
  }, [activeTrans, selectedBook, chapter]);

  // Load compare data when a verse is selected
  useEffect(() => {
    if (compareVerse === null || !selectedBook || compareTrans.length === 0) {
      setCompareData([]);
      return;
    }
    const tids = compareTrans.filter(t => t !== activeTrans);
    Promise.all(
      tids.map(async tid => {
        const t = translations.find(t => t.id === tid)!;
        try {
          const info = await getChapter(tid, selectedBook.id, chapter);
          const v = info?.verses.find(v => v.verse === compareVerse) ?? null;
          return { trans: t, verse: v };
        } catch { return { trans: t, verse: null }; }
      })
    ).then(setCompareData);
  }, [compareVerse, compareTrans, selectedBook, chapter, activeTrans, translations]);

  const highlightForVerse = (verseNum: number) =>
    highlights.find(h => h.verse === verseNum);

  const handleVerseClick = (verseNum: number) => {
    setActiveVerse(v => v === verseNum ? null : verseNum);
    setCompareVerse(verseNum);
  };

  const handleHighlight = async (verseNum: number) => {
    if (!selectedBook) return;
    const existing = highlightForVerse(verseNum);
    if (existing) {
      await removeHighlight(selectedBook.id, chapter, verseNum);
    } else {
      await setHighlight(selectedBook.id, chapter, verseNum, highlightColor);
    }
    getChapterHighlights(selectedBook.id, chapter).then(setHighlights);
  };

  const handlePresentVerse = async (verse: Verse) => {
    const ref = `${verse.book_name} ${verse.chapter}:${verse.verse}`;
    try {
      await pushToAll(verse.text, ref);
      onPresent();
    } catch (e) {
      console.error(e);
    }
  };

  const prevChapter = () => {
    if (chapter > 1) setChapter(c => c - 1);
    else if (books.length && selectedBook) {
      const idx = books.findIndex(b => b.id === selectedBook.id);
      if (idx > 0) { setSelectedBook(books[idx - 1]); setChapter(1); }
    }
  };

  const nextChapter = () => {
    if (chapterInfo && chapter < chapterInfo.total_chapters) setChapter(c => c + 1);
    else if (books.length && selectedBook) {
      const idx = books.findIndex(b => b.id === selectedBook.id);
      if (idx < books.length - 1) { setSelectedBook(books[idx + 1]); setChapter(1); }
    }
  };

  const oldTestament = books.filter(b => b.testament === 'OT');
  const newTestament = books.filter(b => b.testament === 'NT');

  return (
    <div className="flex h-full bg-surface-900">
      {/* ── Left: Bible nav ──────────────────────────────────────────────── */}
      <aside className="w-48 flex flex-col h-full bg-surface-950 border-r border-bdr shrink-0">
        {/* Translation selector */}
        <div className="px-3 py-2 border-b border-bdr">
          <select
            value={activeTrans}
            onChange={e => setActiveTrans(e.target.value)}
            className="w-full bg-surface-700 border border-bdr rounded-md px-2 py-1.5 text-ink text-xs font-semibold focus:outline-none focus:border-accent"
          >
            {translations.map(t => (
              <option key={t.id} value={t.id}>{t.abbreviation} — {t.name}</option>
            ))}
          </select>
        </div>

        {/* Book search */}
        <div className="px-3 py-2 border-b border-bdr relative">
          <input
            value={bookSearch}
            onChange={e => setBookSearch(e.target.value)}
            placeholder="Search book…"
            className="w-full bg-surface-700 border border-bdr rounded-md px-2 py-1 text-xs text-ink placeholder:text-ink-faint focus:outline-none focus:border-accent"
          />
          {bookSuggestions.length > 0 && (
            <div className="absolute left-3 right-3 top-full mt-0.5 bg-surface-700 border border-bdr rounded-md z-50 shadow-panel">
              {bookSuggestions.map(b => (
                <button
                  key={b.id}
                  className="w-full text-left px-3 py-1.5 text-xs text-ink hover:bg-surface-600 hover:text-accent transition-colors"
                  onMouseDown={() => {
                    setSelectedBook(b);
                    setChapter(1);
                    setBookSearch('');
                  }}
                >
                  {b.name}
                </button>
              ))}
            </div>
          )}
        </div>

        {/* Book list */}
        <div className="flex-1 overflow-y-auto py-1">
          {[{ label: 'Old Testament', books: oldTestament }, { label: 'New Testament', books: newTestament }].map(group => (
            <div key={group.label}>
              <div className="px-3 py-1 text-2xs font-bold text-ink-faint uppercase tracking-wider mt-2">{group.label}</div>
              {group.books.map(b => (
                <button
                  key={b.id}
                  onClick={() => { setSelectedBook(b); setChapter(1); }}
                  className={[
                    'w-full text-left px-3 py-1 text-xs transition-colors',
                    selectedBook?.id === b.id
                      ? 'text-accent bg-accent-glow font-semibold'
                      : 'text-ink-muted hover:text-ink hover:bg-surface-800',
                  ].join(' ')}
                >
                  {b.name}
                </button>
              ))}
            </div>
          ))}
        </div>

        {/* Chapter grid */}
        {chapterInfo && (
          <div className="border-t border-bdr px-3 py-2">
            <div className="text-2xs text-ink-faint font-bold uppercase tracking-wider mb-1.5">Chapters</div>
            <div className="grid grid-cols-6 gap-0.5">
              {Array.from({ length: chapterInfo.total_chapters }, (_, i) => i + 1).map(n => (
                <button
                  key={n}
                  onClick={() => setChapter(n)}
                  className={[
                    'rounded text-2xs py-0.5 font-medium transition-colors',
                    chapter === n
                      ? 'bg-accent text-surface-950 font-bold'
                      : 'text-ink-faint hover:bg-surface-700 hover:text-ink',
                  ].join(' ')}
                >
                  {n}
                </button>
              ))}
            </div>
          </div>
        )}
      </aside>

      {/* ── Center: Reading canvas ──────────────────────────────────────── */}
      <main className="flex-1 flex flex-col overflow-hidden min-w-0">
        {/* Chapter header */}
        <div className="flex items-center justify-between px-8 py-3 border-b border-bdr bg-surface-900 shrink-0">
          <button
            onClick={prevChapter}
            className="flex items-center gap-1.5 text-xs text-ink-faint hover:text-ink px-2 py-1 rounded hover:bg-surface-800 transition-colors"
          >
            <ChevronLeft size={14} /> Prev
          </button>

          <div className="text-center">
            <h1 className="text-base font-bold text-ink">
              {chapterInfo?.book_name ?? selectedBook?.name} {chapter}
            </h1>
            {activeTrans && (
              <div className="flex items-center gap-1.5 justify-center mt-0.5">
                {translations.map(t => (
                  <button
                    key={t.id}
                    onClick={() => setActiveTrans(t.id)}
                    className={[
                      'px-2 py-0.5 rounded text-2xs font-bold border transition-colors',
                      activeTrans === t.id
                        ? 'bg-accent text-surface-950 border-accent'
                        : 'text-ink-faint border-bdr hover:border-bdr-strong hover:text-ink',
                    ].join(' ')}
                  >
                    {t.abbreviation}
                  </button>
                ))}
              </div>
            )}
          </div>

          <button
            onClick={nextChapter}
            className="flex items-center gap-1.5 text-xs text-ink-faint hover:text-ink px-2 py-1 rounded hover:bg-surface-800 transition-colors"
          >
            Next <ChevronRight size={14} />
          </button>
        </div>

        {/* Verses */}
        <div className="flex-1 overflow-y-auto px-8 py-6">
          <div className="max-w-2xl mx-auto space-y-1">
            {chapterInfo?.verses.map(v => {
              const hl = highlightForVerse(v.verse);
              const isActive = activeVerse === v.verse;
              return (
                <div
                  key={v.verse}
                  className={[
                    'group relative py-1.5 px-2 rounded-lg -mx-2 cursor-pointer transition-all duration-100',
                    isActive ? 'bg-surface-700' : 'hover:bg-surface-800',
                    hl ? '' : '',
                  ].join(' ')}
                  style={hl ? { backgroundColor: hl.color + '20', borderLeft: `3px solid ${hl.color}` } : {}}
                  onClick={() => handleVerseClick(v.verse)}
                >
                  <span className="select-none text-xs font-bold text-accent mr-2 align-super">{v.verse}</span>
                  <span className="verse-body text-ink leading-relaxed">{v.text}</span>

                  {/* Hover actions */}
                  <div className={[
                    'absolute right-1 top-1/2 -translate-y-1/2 flex items-center gap-1',
                    'opacity-0 group-hover:opacity-100 transition-opacity',
                    isActive ? 'opacity-100' : '',
                  ].join(' ')}>
                    <button
                      title="Present this verse live"
                      onClick={e => { e.stopPropagation(); handlePresentVerse(v); }}
                      className="flex items-center gap-1 px-2 py-1 bg-accent text-surface-950 rounded text-2xs font-bold hover:brightness-110 transition-all"
                    >
                      <Zap size={10} /> Present
                    </button>
                    <button
                      title="Highlight"
                      onClick={e => { e.stopPropagation(); handleHighlight(v.verse); }}
                      className="p-1 rounded bg-surface-600 text-ink-muted hover:text-ink hover:bg-surface-500 transition-colors"
                    >
                      <Highlighter size={12} />
                    </button>
                    <button
                      title="Bookmark"
                      onClick={e => { e.stopPropagation(); toggleBookmark(v.book_id, v.chapter, v.verse, v.book_name, v.text); }}
                      className="p-1 rounded bg-surface-600 text-ink-muted hover:text-ink hover:bg-surface-500 transition-colors"
                    >
                      <BookMarked size={12} />
                    </button>
                    <button
                      title="Compare versions"
                      onClick={e => { e.stopPropagation(); setSidePanel('compare'); setCompareVerse(v.verse); }}
                      className="p-1 rounded bg-surface-600 text-ink-muted hover:text-ink hover:bg-surface-500 transition-colors"
                    >
                      <GitCompare size={12} />
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </main>

      {/* ── Right: Context panel ─────────────────────────────────────────── */}
      <aside className="w-64 flex flex-col h-full bg-surface-950 border-l border-bdr shrink-0">
        {/* Panel tabs */}
        <div className="flex border-b border-bdr shrink-0">
          {(['compare', 'notes', 'ai'] as const).map(panel => (
            <button
              key={panel}
              onClick={() => setSidePanel(p => p === panel ? null : panel)}
              className={[
                'flex-1 py-2 text-2xs font-bold uppercase tracking-wider transition-colors capitalize',
                sidePanel === panel
                  ? 'text-accent border-b-2 border-accent'
                  : 'text-ink-faint hover:text-ink',
              ].join(' ')}
            >
              {panel === 'ai' ? 'AI Study' : panel}
            </button>
          ))}
        </div>

        <div className="flex-1 overflow-y-auto">
          {/* Compare panel */}
          {sidePanel === 'compare' && (
            <div className="p-3">
              <div className="text-2xs text-ink-faint font-bold uppercase tracking-wider mb-2">Compare Translations</div>
              {/* Translation toggles */}
              <div className="flex flex-wrap gap-1 mb-3">
                {translations.filter(t => t.id !== activeTrans).map(t => (
                  <button
                    key={t.id}
                    onClick={() => setCompareTrans(prev =>
                      prev.includes(t.id) ? prev.filter(x => x !== t.id) : [...prev, t.id]
                    )}
                    className={[
                      'px-2 py-0.5 rounded text-2xs font-bold border transition-colors',
                      compareTrans.includes(t.id)
                        ? 'bg-accent text-surface-950 border-accent'
                        : 'text-ink-faint border-bdr hover:border-bdr-strong',
                    ].join(' ')}
                  >
                    {t.abbreviation}
                  </button>
                ))}
              </div>

              {compareVerse !== null && compareData.length > 0 && (
                <div className="space-y-3">
                  <div className="text-2xs text-ink-faint font-semibold">
                    {chapterInfo?.book_name} {chapter}:{compareVerse}
                  </div>
                  {/* Active translation */}
                  <div className="p-2.5 bg-surface-700 rounded-lg border border-bdr">
                    <div className="text-2xs font-bold text-accent mb-1">
                      {translations.find(t => t.id === activeTrans)?.abbreviation}
                    </div>
                    <p className="text-xs text-ink leading-relaxed">
                      {chapterInfo?.verses.find(v => v.verse === compareVerse)?.text}
                    </p>
                  </div>
                  {compareData.map(({ trans, verse }) => verse && (
                    <div key={trans.id} className="p-2.5 bg-surface-800 rounded-lg border border-bdr">
                      <div className="text-2xs font-bold text-ink-muted mb-1">{trans.abbreviation}</div>
                      <p className="text-xs text-ink-muted leading-relaxed">{verse.text}</p>
                    </div>
                  ))}
                </div>
              )}
              {compareVerse === null && (
                <p className="text-xs text-ink-faint">Click a verse to compare.</p>
              )}
            </div>
          )}

          {/* Notes panel */}
          {sidePanel === 'notes' && (
            <div className="p-3">
              <div className="text-2xs text-ink-faint font-bold uppercase tracking-wider mb-2">Notes</div>
              <p className="text-xs text-ink-faint">Click a verse, then add a note.</p>
            </div>
          )}

          {/* AI Study panel */}
          {sidePanel === 'ai' && (
            <div className="p-3">
              <div className="text-2xs text-ink-faint font-bold uppercase tracking-wider mb-2">AI Study</div>
              <div className="p-3 bg-surface-700 border border-bdr rounded-lg">
                <p className="text-xs text-ink-faint">AI study features require Standard or Premium plan.</p>
                <button className="mt-2 w-full py-1.5 bg-accent text-surface-950 rounded text-2xs font-bold hover:brightness-110 transition-all">
                  Upgrade Plan
                </button>
              </div>
            </div>
          )}
        </div>

        {/* Highlight palette (always visible) */}
        <div className="border-t border-bdr p-3 shrink-0">
          <div className="text-2xs text-ink-faint font-bold uppercase tracking-wider mb-2">Highlight Color</div>
          <div className="flex gap-1.5">
            {HIGHLIGHT_COLORS.map(c => (
              <button
                key={c}
                onClick={() => setHighlightColor(c)}
                className="w-5 h-5 rounded-full transition-transform hover:scale-110"
                style={{
                  backgroundColor: c,
                  outline: highlightColor === c ? `2px solid ${c}` : 'none',
                  outlineOffset: '2px',
                }}
              />
            ))}
          </div>
        </div>
      </aside>
    </div>
  );
}
