import { useCallback, useEffect, useMemo, useState } from 'react';
import { getBooks, getChapter, getTranslations, getVerse } from '../lib/commands';
import type { Book, ChapterInfo, Translation, Verse } from '../lib/types';

type ParsedReference = {
  book: Book;
  chapter: number;
  verse?: number;
};

const BOOK_ALIASES: Record<string, string[]> = {
  genesis: ['gen', 'ge', 'gn'],
  exodus: ['exo', 'ex'],
  leviticus: ['lev', 'le', 'lv'],
  numbers: ['num', 'nu', 'nm', 'nb'],
  deuteronomy: ['deut', 'dt'],
  psalms: ['psalm', 'ps', 'psa', 'psm'],
  proverbs: ['prov', 'pr', 'prv'],
  ecclesiastes: ['eccl', 'ecc', 'qoheleth'],
  song: ['song of songs', 'song of solomon', 'sos'],
  isaiah: ['isa', 'is'],
  jeremiah: ['jer', 'je', 'jr'],
  ezekiel: ['ezek', 'eze', 'ezk'],
  daniel: ['dan', 'da', 'dn'],
  matthew: ['matt', 'mt'],
  mark: ['mk', 'mrk'],
  luke: ['lk', 'luk'],
  john: ['jn', 'jhn'],
  romans: ['rom', 'ro', 'rm'],
  corinthians: ['cor'],
  galatians: ['gal'],
  ephesians: ['eph'],
  philippians: ['phil', 'php'],
  colossians: ['col'],
  thessalonians: ['thess', 'thes'],
  timothy: ['tim'],
  hebrews: ['heb'],
  james: ['jas', 'jam'],
  peter: ['pet', 'pe'],
  revelation: ['rev', 're'],
};

function normalise(value: string) {
  return value.toLowerCase().replace(/[^a-z0-9]/g, '');
}

function bookTokens(book: Book) {
  const base = [book.name, book.short_name];
  const lowerName = book.name.toLowerCase();
  const withoutOrdinal = lowerName.replace(/^\d+\s+/, '');
  const aliases = BOOK_ALIASES[withoutOrdinal] ?? [];
  return [...base, ...aliases].map(normalise).filter(Boolean);
}

function parseReference(query: string, books: Book[]): ParsedReference | null {
  const compact = query.trim().toLowerCase().replace(/\s+/g, '');
  if (!compact) return null;

  const match = compact.match(/^([1-3]?[a-z]+)(\d+)(?::?(\d+))?$/);
  if (!match) return null;

  const [, rawBook, rawChapter, rawVerse] = match;
  const bookKey = normalise(rawBook);
  const book = books.find(item => bookTokens(item).some(token => token === bookKey || token.startsWith(bookKey)));
  if (!book) return null;

  return {
    book,
    chapter: Math.max(1, Number(rawChapter)),
    verse: rawVerse ? Math.max(1, Number(rawVerse)) : undefined,
  };
}

export function useScriptureEngine() {
  const [translations, setTranslations] = useState<Translation[]>([]);
  const [books, setBooks] = useState<Book[]>([]);
  const [translationId, setTranslationId] = useState('');
  const [bookId, setBookId] = useState<number | null>(null);
  const [chapter, setChapter] = useState(1);
  const [verse, setVerse] = useState<number | null>(null);
  const [chapterInfo, setChapterInfo] = useState<ChapterInfo | null>(null);
  const [selectedVerse, setSelectedVerse] = useState<Verse | null>(null);
  const [query, setQuery] = useState('');
  const [message, setMessage] = useState('');

  useEffect(() => {
    Promise.all([getTranslations(), getBooks()]).then(([loadedTranslations, loadedBooks]) => {
      setTranslations(loadedTranslations);
      setBooks(loadedBooks);
      if (loadedTranslations.length) setTranslationId(loadedTranslations[0].id);
      if (loadedBooks.length) setBookId(loadedBooks[0].id);
    });
  }, []);

  useEffect(() => {
    if (!translationId || !bookId) return;
    setChapterInfo(null);
    getChapter(translationId, bookId, chapter)
      .then(info => {
        setChapterInfo(info);
        if (!info) return;
        const nextVerse = verse ?? 1;
        const found = info.verses.find(item => item.verse === nextVerse) ?? info.verses[0] ?? null;
        setSelectedVerse(found);
        setVerse(found?.verse ?? null);
      })
      .catch(() => {
        setChapterInfo(null);
        setSelectedVerse(null);
      });
  }, [translationId, bookId, chapter, verse]);

  const selectedBook = useMemo(
    () => books.find(book => book.id === bookId) ?? null,
    [books, bookId],
  );

  const bookSuggestions = useMemo(() => {
    const q = normalise(query);
    if (!q) return books.slice(0, 10);
    return books.filter(book => bookTokens(book).some(token => token.startsWith(q))).slice(0, 10);
  }, [books, query]);

  const referenceSuggestion = useMemo(() => parseReference(query, books), [query, books]);

  const selectBook = useCallback((book: Book) => {
    setBookId(book.id);
    setChapter(1);
    setVerse(1);
    setQuery('');
    setMessage('');
  }, []);

  const selectChapter = useCallback((nextChapter: number) => {
    setChapter(nextChapter);
    setVerse(1);
    setMessage('');
  }, []);

  const selectVerse = useCallback((nextVerse: number | null) => {
    setVerse(nextVerse);
    setMessage('');
  }, []);

  const goToReference = useCallback(async (value = query) => {
    const parsed = parseReference(value, books);
    if (!parsed || !translationId) {
      setMessage('Reference not found');
      return null;
    }

    setBookId(parsed.book.id);
    setChapter(parsed.chapter);
    setVerse(parsed.verse ?? 1);
    setQuery('');
    setMessage('');

    if (parsed.verse) {
      const found = await getVerse(translationId, parsed.book.id, parsed.chapter, parsed.verse);
      setSelectedVerse(found);
      return found;
    }

    const info = await getChapter(translationId, parsed.book.id, parsed.chapter);
    const firstVerse = info?.verses[0] ?? null;
    setChapterInfo(info);
    setSelectedVerse(firstVerse);
    setVerse(firstVerse?.verse ?? null);
    return firstVerse;
  }, [books, query, translationId]);

  return {
    translations,
    books,
    translationId,
    setTranslationId,
    selectedBook,
    bookId,
    chapter,
    verse,
    chapterInfo,
    selectedVerse,
    query,
    setQuery,
    message,
    bookSuggestions,
    referenceSuggestion,
    selectBook,
    selectChapter,
    selectVerse,
    goToReference,
  };
}
