import { useState, useEffect, useCallback } from 'react';
import Sidebar, { type Page } from './components/Sidebar';
import Read from './pages/Read';
import Present from './pages/Present';
import Outputs from './pages/Outputs';
import Songs from './pages/Songs';
import Library from './pages/Library';
import Settings from './pages/Settings';
import CommandPalette from './components/CommandPalette';

export default function App() {
  const [page, setPage] = useState<Page>('Read');
  const [paletteOpen, setPaletteOpen] = useState(false);

  // Global keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setPaletteOpen(o => !o);
      }
      if (e.key === 'Escape' && paletteOpen) {
        setPaletteOpen(false);
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [paletteOpen]);

  const navigate = useCallback((p: Page) => {
    setPage(p);
    setPaletteOpen(false);
  }, []);

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-surface-900">
      <Sidebar current={page} onNavigate={navigate} />

      <main className="flex-1 overflow-hidden min-w-0">
        {page === 'Read'     && <Read onPresent={() => navigate('Present')} />}
        {page === 'Present'  && <Present />}
        {page === 'Outputs'  && <Outputs />}
        {page === 'Songs'    && <Songs />}
        {page === 'Library'  && <Library />}
        {page === 'Settings' && <Settings />}
      </main>

      {paletteOpen && (
        <CommandPalette
          onNavigate={navigate}
          onClose={() => setPaletteOpen(false)}
        />
      )}
    </div>
  );
}
