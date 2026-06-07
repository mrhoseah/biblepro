import { useMemo, useState } from 'react';
import { BookOpen, Search } from 'lucide-react';
import { useScriptureEngine } from '../hooks/useScriptureEngine';
import type { Verse } from '../lib/types';

type ScriptureEngine = ReturnType<typeof useScriptureEngine>;

type Props = {
  engine: ScriptureEngine;
  title?: string;
  compactBooks?: boolean;
  stagedVerse?: Verse | null;
  liveVerse?: Verse | null;
  onVerseSelect?: (verse: Verse) => void;
  onVerseDoubleClick?: (verse: Verse) => void;
};

export default function ScriptureNavigator({
  engine,
  title = 'Navigation',
  compactBooks = false,
  stagedVerse,
  liveVerse,
  onVerseSelect,
  onVerseDoubleClick,
}: Props) {
  const [bookQuery, setBookQuery] = useState('');

  const filteredBooks = useMemo(() => {
    const query = bookQuery.trim().toLowerCase();
    if (!query) return engine.books;
    return engine.books.filter(book =>
      book.name.toLowerCase().startsWith(query) ||
      book.short_name.toLowerCase().startsWith(query),
    );
  }, [bookQuery, engine.books]);

  const selectedVersion = engine.translations.find(item => item.id === engine.translationId);
  const selectedRef = [
    selectedVersion?.abbreviation,
    engine.selectedBook?.name,
    engine.chapter,
    engine.verse ? `:${engine.verse}` : '',
  ].filter(Boolean).join(' ');

  const handleSearch = async () => {
    const verse = await engine.goToReference();
    if (verse) onVerseSelect?.(verse);
  };

  const handleVerseClick = (verse: Verse) => {
    engine.selectVerse(verse.verse);
    onVerseSelect?.(verse);
  };

  return (
    <aside className="flex min-h-0 flex-col border-r border-bdr bg-surface-950">
      <div className="shrink-0 border-b border-bdr p-4">
        <div className="mb-2 flex items-center justify-between">
          <p className="flex items-center gap-2 text-2xs font-black uppercase tracking-wider text-ink-faint">
            <BookOpen size={13} /> {title}
          </p>
          <span className="max-w-[210px] truncate text-2xs font-bold text-accent">{selectedRef}</span>
        </div>
        <div className="relative">
          <Search size={13} className="absolute left-3 top-1/2 -translate-y-1/2 text-accent" />
          <input
            data-scripture-search="true"
            value={engine.query}
            onChange={event => engine.setQuery(event.target.value)}
            onKeyDown={event => {
              if (event.key === 'Enter') handleSearch();
            }}
            placeholder="Type reference: John 3:16, jn316"
            className="w-full rounded-xl border border-bdr bg-surface-800 py-3 pl-9 pr-3 text-sm font-semibold text-ink outline-none placeholder:text-ink-faint focus:border-accent"
          />
        </div>
        {engine.message && <p className="mt-2 text-xs text-danger">{engine.message}</p>}
        <div className="mt-3 grid grid-cols-4 gap-2">
          <StatusPill label="Version" value={selectedVersion?.abbreviation ?? '-'} />
          <StatusPill label="Book" value={engine.selectedBook?.short_name ?? '-'} />
          <StatusPill label="Chapter" value={String(engine.chapter ?? '-')} />
          <StatusPill label="Verse" value={engine.verse ? String(engine.verse) : '-'} />
        </div>
      </div>

      <div className="grid min-h-0 flex-1 grid-rows-[auto_minmax(190px,0.9fr)_auto_minmax(170px,0.7fr)]">
        <NavSection title="Bible Versions" meta={engine.translations.find(item => item.id === engine.translationId)?.abbreviation}>
          <div className="grid grid-cols-3 gap-2">
            {engine.translations.map(item => (
              <button
                key={item.id}
                onClick={() => engine.setTranslationId(item.id)}
                className={[
                  'rounded-xl border px-3 py-2 text-left text-xs font-black transition-all',
                  item.id === engine.translationId ? 'border-accent bg-accent text-surface-950 shadow-glow' : 'border-bdr bg-surface-800 text-ink-faint hover:border-accent/50 hover:text-ink',
                ].join(' ')}
                title={item.name}
              >
                {item.abbreviation}
                <span className="mt-0.5 block truncate text-[10px] font-semibold opacity-70">{item.language || 'Bible'}</span>
              </button>
            ))}
            {!engine.translations.length && (
              <p className="col-span-4 rounded-lg border border-bdr bg-surface-800 px-3 py-2 text-xs text-ink-faint">
                No Bible versions installed. Install one from Library.
              </p>
            )}
          </div>
        </NavSection>

        <NavSection title="Books" meta={engine.selectedBook?.name} scroll>
          <input
            value={bookQuery}
            onChange={event => setBookQuery(event.target.value)}
            placeholder="Search books..."
            className="mb-2 w-full rounded-xl border border-bdr bg-surface-800 px-3 py-2 text-xs text-ink outline-none placeholder:text-ink-faint focus:border-accent"
          />
          <div className={compactBooks ? 'grid grid-cols-2 gap-2' : 'grid grid-cols-2 gap-2'}>
            {filteredBooks.map(book => (
              <button
                key={book.id}
                onClick={() => engine.selectBook(book)}
                className={[
                  'rounded-xl border px-3 py-2 text-left text-xs font-bold transition-all',
                  engine.bookId === book.id ? 'border-accent bg-accent/15 text-accent ring-1 ring-accent/30' : 'border-bdr bg-surface-800 text-ink-muted hover:border-accent/50 hover:text-ink',
                ].join(' ')}
              >
                <span className="block truncate">{compactBooks ? book.short_name : book.name}</span>
                {!compactBooks && <span className="mt-0.5 block text-[10px] text-ink-faint">{book.short_name}</span>}
              </button>
            ))}
          </div>
        </NavSection>

        <NavSection title="Chapters" meta={engine.chapterInfo ? `${engine.chapterInfo.book_name} ${engine.chapter}` : 'Select book'}>
          {engine.chapterInfo ? (
            <div className="grid max-h-48 grid-cols-8 gap-2 overflow-y-auto pr-1">
                {Array.from({ length: engine.chapterInfo.total_chapters }, (_, index) => index + 1).map(chapter => (
                  <button
                    key={chapter}
                    onClick={() => engine.selectChapter(chapter)}
                    className={[
                      'rounded-xl border py-2 text-sm font-black transition-all',
                      engine.chapter === chapter ? 'border-accent bg-accent text-surface-950 shadow-glow' : 'border-bdr bg-surface-800 text-ink-faint hover:border-accent/50 hover:text-ink',
                    ].join(' ')}
                  >
                    {chapter}
                  </button>
                ))}
            </div>
          ) : (
            <p className="rounded-lg border border-bdr bg-surface-800 px-3 py-2 text-xs text-ink-faint">
              Select a book to load chapters.
            </p>
          )}
        </NavSection>

        <NavSection title="Verses" meta={engine.verse ? `${engine.chapter}:${engine.verse}` : 'Select chapter'} scroll>
          {engine.chapterInfo ? (
            <div className="grid grid-cols-8 gap-2">
              {engine.chapterInfo.verses.map(verse => (
                <button
                  key={verse.id}
                  onClick={() => handleVerseClick(verse)}
                  onDoubleClick={() => onVerseDoubleClick?.(verse)}
                  className={[
                    'rounded-xl border py-2 text-sm font-black transition-all',
                    liveVerse?.id === verse.id
                      ? 'border-live bg-live text-surface-950'
                      : stagedVerse?.id === verse.id || engine.verse === verse.verse
                      ? 'border-accent bg-accent text-surface-950 shadow-glow'
                      : 'border-bdr bg-surface-800 text-ink-faint hover:border-accent/50 hover:text-ink',
                  ].join(' ')}
                  title={`${engine.chapterInfo!.book_name} ${engine.chapter}:${verse.verse}`}
                >
                  {verse.verse}
                </button>
              ))}
            </div>
          ) : (
            <p className="rounded-lg border border-bdr bg-surface-800 px-3 py-2 text-xs text-ink-faint">
              Select a chapter to load verses.
            </p>
          )}
        </NavSection>
      </div>
    </aside>
  );
}

function StatusPill({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-xl border border-bdr bg-surface-900 px-2 py-2 text-center">
      <p className="text-[9px] font-black uppercase tracking-wider text-ink-faint">{label}</p>
      <p className="mt-1 truncate text-xs font-black text-ink">{value}</p>
    </div>
  );
}

function NavSection({
  title,
  meta,
  scroll = false,
  children,
}: {
  title: string;
  meta?: string;
  scroll?: boolean;
  children: React.ReactNode;
}) {
  return (
    <section className="flex min-h-0 flex-col border-b border-bdr p-4">
      <div className="mb-2 flex items-center justify-between gap-2">
        <span className="text-2xs font-black uppercase tracking-wider text-ink-faint">{title}</span>
        {meta && <span className="truncate text-2xs font-bold text-accent">{meta}</span>}
      </div>
      <div className={scroll ? 'min-h-0 overflow-y-auto pr-1' : ''}>{children}</div>
    </section>
  );
}
