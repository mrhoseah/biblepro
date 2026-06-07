import { useEffect, useState } from 'react';
import { BookMarked, Copy, GitCompare, Highlighter, History, Star, Zap } from 'lucide-react';
import ScriptureNavigator from '../components/ScriptureNavigator';
import { useScriptureEngine } from '../hooks/useScriptureEngine';
import { getChapterHighlights, removeHighlight, setHighlight, toggleBookmark } from '../lib/commands';
import { pushToAll } from '../lib/commands';
import type { Highlight, Verse } from '../lib/types';

interface Props { onPresent: () => void }

const HIGHLIGHT_COLORS = ['#F6E37A', '#A8F5A0', '#A0C8FF', '#F5A0C8', '#F5C0A0'];

export default function Read({ onPresent }: Props) {
  const engine = useScriptureEngine();
  const [highlights, setHighlights] = useState<Highlight[]>([]);
  const [highlightColor, setHighlightColor] = useState(HIGHLIGHT_COLORS[0]);
  const [recent, setRecent] = useState<Verse[]>([]);
  const [favorites, setFavorites] = useState<Verse[]>([]);
  const [tool, setTool] = useState<'compare' | 'notes' | 'history'>('compare');

  useEffect(() => {
    if (!engine.bookId || !engine.chapter) return;
    getChapterHighlights(engine.bookId, engine.chapter).then(setHighlights).catch(() => setHighlights([]));
  }, [engine.bookId, engine.chapter]);

  const activeTranslation = engine.translations.find(item => item.id === engine.translationId);

  const presentVerse = async (verse: Verse) => {
    await pushToAll(verse.text, `${verse.book_name} ${verse.chapter}:${verse.verse}`);
    setRecent(current => [verse, ...current.filter(item => item.id !== verse.id)].slice(0, 8));
    onPresent();
  };

  const copyVerse = async (verse: Verse) => {
    await navigator.clipboard.writeText(`${verse.book_name} ${verse.chapter}:${verse.verse} ${verse.text}`);
  };

  const toggleFavorite = (verse: Verse) => {
    setFavorites(current =>
      current.some(item => item.id === verse.id)
        ? current.filter(item => item.id !== verse.id)
        : [verse, ...current].slice(0, 12),
    );
  };

  const highlightForVerse = (verseNum: number) =>
    highlights.find(item => item.verse === verseNum);

  const toggleHighlight = async (verseNum: number) => {
    if (!engine.bookId) return;
    const existing = highlightForVerse(verseNum);
    if (existing) await removeHighlight(engine.bookId, engine.chapter, verseNum);
    else await setHighlight(engine.bookId, engine.chapter, verseNum, highlightColor);
    getChapterHighlights(engine.bookId, engine.chapter).then(setHighlights);
  };

  return (
    <div className="grid h-full grid-rows-[auto_1fr_auto] bg-surface-900">
      <header className="border-b border-bdr bg-surface-950 px-4 py-3">
        <div className="flex items-center justify-between gap-4">
          <div>
            <p className="text-2xs font-black uppercase tracking-wider text-accent">Read Scripture</p>
            <h1 className="text-lg font-black text-ink">
              {engine.chapterInfo?.book_name ?? engine.selectedBook?.name ?? 'Bible'} {engine.chapter}
              {engine.verse ? `:${engine.verse}` : ''}
            </h1>
          </div>
          <p className="text-xs font-bold text-ink-faint">{activeTranslation?.name ?? 'Select a translation'}</p>
        </div>
      </header>

      <div className="grid min-h-0 grid-cols-[300px_minmax(0,1fr)_280px]">
        <ScriptureNavigator engine={engine} title="Read Navigation" onVerseSelect={verse => engine.selectVerse(verse.verse)} />

        <main className="flex min-h-0 flex-col">
          <div className="flex items-center justify-between border-b border-bdr bg-surface-900 px-6 py-3">
            <div>
              <h1 className="text-xl font-black text-ink">
                {engine.chapterInfo?.book_name ?? engine.selectedBook?.name ?? 'Bible'} {engine.chapter}
              </h1>
              <p className="text-xs text-ink-faint">{activeTranslation?.name ?? 'Select a translation'}</p>
            </div>
            <div className="rounded-lg border border-bdr bg-surface-800 px-3 py-2 text-right">
              <p className="text-2xs font-black uppercase tracking-wider text-ink-faint">Selected</p>
              <p className="text-sm font-black text-accent">
                {engine.chapterInfo?.book_name ?? engine.selectedBook?.name ?? 'Book'} {engine.chapter}
                {engine.verse ? `:${engine.verse}` : ''}
              </p>
            </div>
          </div>

          <div className="min-h-0 flex-1 overflow-y-auto px-8 py-6">
            <div className="mx-auto max-w-3xl space-y-1">
              {engine.chapterInfo?.verses.map(item => {
                const active = engine.verse === item.verse;
                const highlight = highlightForVerse(item.verse);
                return (
                  <div
                    key={item.id}
                    onClick={() => engine.selectVerse(item.verse)}
                    className={[
                      'group relative rounded-xl border px-4 py-3 transition-all',
                      active ? 'border-accent bg-accent/10' : 'border-transparent hover:border-bdr hover:bg-surface-800',
                    ].join(' ')}
                    style={highlight ? { boxShadow: `inset 3px 0 0 ${highlight.color}` } : {}}
                  >
                    <span className="mr-3 align-super text-xs font-black text-accent">{item.verse}</span>
                    <span className="font-serif text-[17px] leading-[1.85] text-ink">{item.text}</span>
                    <div className="absolute right-2 top-2 flex gap-1 opacity-0 transition-opacity group-hover:opacity-100">
                      <button onClick={event => { event.stopPropagation(); presentVerse(item); }} className="rounded bg-accent px-2 py-1 text-2xs font-black text-surface-950">
                        Present
                      </button>
                      <button onClick={event => { event.stopPropagation(); toggleHighlight(item.verse); }} className="rounded bg-surface-700 p-1 text-ink-faint hover:text-ink">
                        <Highlighter size={12} />
                      </button>
                      <button onClick={event => { event.stopPropagation(); toggleFavorite(item); }} className="rounded bg-surface-700 p-1 text-ink-faint hover:text-accent">
                        <Star size={12} />
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        </main>

        <aside className="flex min-h-0 flex-col border-l border-bdr bg-surface-950">
          <div className="grid grid-cols-3 border-b border-bdr">
            {(['compare', 'notes', 'history'] as const).map(item => (
              <button
                key={item}
                onClick={() => setTool(item)}
                className={[
                  'py-2 text-2xs font-black uppercase tracking-wider',
                  tool === item ? 'text-accent' : 'text-ink-faint hover:text-ink',
                ].join(' ')}
              >
                {item}
              </button>
            ))}
          </div>

          <div className="min-h-0 flex-1 overflow-y-auto p-3">
            {tool === 'compare' && (
              <div className="space-y-3">
                <p className="text-xs text-ink-faint">Version switching keeps the current book, chapter and verse selected.</p>
                {engine.translations.map(item => (
                  <button
                    key={item.id}
                    onClick={() => engine.setTranslationId(item.id)}
                    className={[
                      'w-full rounded-lg border px-3 py-2 text-left text-xs transition-all',
                      item.id === engine.translationId ? 'border-accent bg-accent/10 text-accent' : 'border-bdr bg-surface-800 text-ink-muted hover:text-ink',
                    ].join(' ')}
                  >
                    <span className="font-black">{item.abbreviation}</span>
                    <span className="ml-2 text-ink-faint">{item.name}</span>
                  </button>
                ))}
              </div>
            )}

            {tool === 'notes' && (
              <div>
                <p className="mb-3 text-xs text-ink-faint">Verse tools</p>
                {engine.selectedVerse && (
                  <div className="rounded-xl border border-bdr bg-surface-800 p-3">
                    <p className="text-xs font-black text-accent">
                      {engine.selectedVerse.book_name} {engine.selectedVerse.chapter}:{engine.selectedVerse.verse}
                    </p>
                    <p className="mt-2 text-xs leading-relaxed text-ink-muted">{engine.selectedVerse.text}</p>
                  </div>
                )}
                <div className="mt-4 flex gap-2">
                  {HIGHLIGHT_COLORS.map(color => (
                    <button
                      key={color}
                      onClick={() => setHighlightColor(color)}
                      className="size-6 rounded-full"
                      style={{ backgroundColor: color, outline: highlightColor === color ? `2px solid ${color}` : 'none', outlineOffset: 2 }}
                    />
                  ))}
                </div>
              </div>
            )}

            {tool === 'history' && (
              <div className="space-y-5">
                <div>
                  <p className="mb-2 flex items-center gap-2 text-2xs font-black uppercase tracking-wider text-ink-faint">
                    <History size={12} /> Recent
                  </p>
                  {recent.map(item => (
                    <button key={item.id} onClick={() => presentVerse(item)} className="mb-1 block w-full rounded-lg bg-surface-800 px-3 py-2 text-left text-xs text-ink-muted hover:text-accent">
                      {item.book_name} {item.chapter}:{item.verse}
                    </button>
                  ))}
                  {!recent.length && <p className="text-xs text-ink-faint">Presented verses appear here.</p>}
                </div>
                <div>
                  <p className="mb-2 flex items-center gap-2 text-2xs font-black uppercase tracking-wider text-ink-faint">
                    <Star size={12} /> Favorites
                  </p>
                  {favorites.map(item => (
                    <button key={item.id} onClick={() => presentVerse(item)} className="mb-1 block w-full rounded-lg bg-surface-800 px-3 py-2 text-left text-xs text-ink-muted hover:text-accent">
                      {item.book_name} {item.chapter}:{item.verse}
                    </button>
                  ))}
                  {!favorites.length && <p className="text-xs text-ink-faint">Star verses to pin them here.</p>}
                </div>
              </div>
            )}
          </div>
        </aside>
      </div>

      <footer className="flex items-center gap-2 border-t border-bdr bg-surface-950 px-4 py-2">
        <button disabled={!engine.selectedVerse} onClick={() => engine.selectedVerse && presentVerse(engine.selectedVerse)} className="flex items-center gap-1.5 rounded-lg bg-accent px-4 py-2 text-xs font-black text-surface-950 disabled:opacity-40">
          <Zap size={13} /> Present
        </button>
        <button disabled={!engine.selectedVerse} onClick={() => engine.selectedVerse && copyVerse(engine.selectedVerse)} className="flex items-center gap-1.5 rounded-lg border border-bdr bg-surface-800 px-3 py-2 text-xs font-bold text-ink-faint hover:text-ink disabled:opacity-40">
          <Copy size={13} /> Copy
        </button>
        <button disabled={!engine.selectedVerse} onClick={() => engine.selectedVerse && toggleBookmark(engine.selectedVerse.book_id, engine.selectedVerse.chapter, engine.selectedVerse.verse, engine.selectedVerse.book_name, engine.selectedVerse.text)} className="flex items-center gap-1.5 rounded-lg border border-bdr bg-surface-800 px-3 py-2 text-xs font-bold text-ink-faint hover:text-ink disabled:opacity-40">
          <BookMarked size={13} /> Bookmark
        </button>
        <button onClick={() => setTool('compare')} className="flex items-center gap-1.5 rounded-lg border border-bdr bg-surface-800 px-3 py-2 text-xs font-bold text-ink-faint hover:text-ink">
          <GitCompare size={13} /> Compare
        </button>
        <div className="ml-auto flex items-center gap-2 overflow-hidden text-2xs text-ink-faint">
          <span>Recent:</span>
          {recent.slice(0, 5).map(item => (
            <button key={item.id} onClick={() => presentVerse(item)} className="rounded bg-surface-800 px-2 py-1 hover:text-accent">
              {item.book_name} {item.chapter}:{item.verse}
            </button>
          ))}
        </div>
      </footer>
    </div>
  );
}
