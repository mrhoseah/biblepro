import { useState, useEffect, useRef } from 'react';
import { Search } from 'lucide-react';
import type { Page } from './Sidebar';

const NAV_ACTIONS: { label: string; page: Page; shortcut?: string }[] = [
  { label: 'Go to Present',  page: 'Present',  shortcut: 'P' },
  { label: 'Go to Songs',    page: 'Songs',    shortcut: 'S' },
  { label: 'Go to Media',    page: 'Media',    shortcut: 'M' },
  { label: 'Go to Countdowns', page: 'Countdowns', shortcut: 'C' },
  { label: 'Go to Outputs',  page: 'Outputs',  shortcut: 'O' },
  { label: 'Go to Library',  page: 'Library',  shortcut: 'L' },
  { label: 'Go to Settings', page: 'Settings', shortcut: ',' },
];

interface Props {
  onNavigate: (page: Page) => void;
  onClose: () => void;
}

export default function CommandPalette({ onNavigate, onClose }: Props) {
  const [query, setQuery] = useState('');
  const [idx, setIdx] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => { inputRef.current?.focus(); }, []);

  const filtered = NAV_ACTIONS.filter(a =>
    query === '' || a.label.toLowerCase().includes(query.toLowerCase())
  );

  useEffect(() => { setIdx(0); }, [query]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') { e.preventDefault(); setIdx(i => Math.min(i + 1, filtered.length - 1)); }
    if (e.key === 'ArrowUp')   { e.preventDefault(); setIdx(i => Math.max(i - 1, 0)); }
    if (e.key === 'Enter' && filtered[idx]) { onNavigate(filtered[idx].page); }
    if (e.key === 'Escape') { onClose(); }
  };

  return (
    <div
      className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-start justify-center pt-28"
      onClick={onClose}
    >
      <div
        className="w-full max-w-lg bg-surface-800 border border-bdr rounded-2xl shadow-panel overflow-hidden animate-in fade-in slide-in-from-top-4 duration-150"
        onClick={e => e.stopPropagation()}
      >
        <div className="flex items-center gap-3 px-4 py-3 border-b border-bdr">
          <Search size={16} className="text-ink-faint shrink-0" />
          <input
            ref={inputRef}
            value={query}
            onChange={e => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search commands, navigate…"
            className="flex-1 bg-transparent text-ink placeholder:text-ink-faint focus:outline-none text-sm"
          />
          <kbd className="px-1.5 py-0.5 text-2xs text-ink-faint bg-surface-700 border border-bdr rounded">Esc</kbd>
        </div>

        <div className="max-h-72 overflow-y-auto py-1">
          {filtered.length === 0 ? (
            <div className="px-4 py-6 text-sm text-ink-faint text-center">No results.</div>
          ) : filtered.map((action, i) => (
            <button
              key={action.page}
              onClick={() => onNavigate(action.page)}
              className={[
                'w-full flex items-center justify-between px-4 py-2.5 text-left transition-colors',
                i === idx ? 'bg-surface-700 text-ink' : 'text-ink-muted hover:bg-surface-700 hover:text-ink',
              ].join(' ')}
            >
              <span className="text-sm">{action.label}</span>
              {action.shortcut && (
                <kbd className="px-1.5 py-0.5 text-2xs text-ink-faint bg-surface-600 border border-bdr rounded">
                  {action.shortcut}
                </kbd>
              )}
            </button>
          ))}
        </div>

        <div className="px-4 py-2 border-t border-bdr flex items-center gap-4 text-2xs text-ink-faint">
          <span>↑↓ navigate</span>
          <span>↵ select</span>
          <span>Esc close</span>
          <span className="ml-auto">⌘K to open</span>
        </div>
      </div>
    </div>
  );
}
