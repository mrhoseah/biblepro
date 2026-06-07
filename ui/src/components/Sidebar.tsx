import { Clock3, Images, Music, MonitorPlay, Monitor, Library, Settings, ChevronLeft, ChevronRight } from 'lucide-react';
import { useState } from 'react';

export type Page = 'Library' | 'Songs' | 'Media' | 'Countdowns' | 'Present' | 'Outputs' | 'Settings';

const NAV: { id: Page; Icon: React.ElementType; label: string }[] = [
  { id: 'Library',  Icon: Library,      label: 'Library' },
  { id: 'Songs',    Icon: Music,        label: 'Songs' },
  { id: 'Media',    Icon: Images,       label: 'Media' },
  { id: 'Countdowns', Icon: Clock3,     label: 'Countdowns' },
  { id: 'Present',  Icon: MonitorPlay,  label: 'Present' },
  { id: 'Outputs',  Icon: Monitor,      label: 'Outputs' },
];

interface Props {
  current: Page;
  onNavigate: (page: Page) => void;
}

export default function Sidebar({ current, onNavigate }: Props) {
  const [expanded, setExpanded] = useState(true);

  return (
    <aside
      className="flex flex-col h-screen bg-surface-950 border-r border-bdr transition-all duration-200 shrink-0"
      style={{ width: expanded ? 192 : 52 }}
    >
      {/* Logo */}
      <div className="flex items-center gap-2.5 px-3 h-12 border-b border-bdr shrink-0">
        <div className="w-6 h-6 rounded-md bg-accent flex items-center justify-center shrink-0">
          <span className="text-surface-950 font-black text-xs">B</span>
        </div>
        {expanded && (
          <span className="font-bold text-sm tracking-tight text-ink">BiblePro</span>
        )}
      </div>

      {/* Nav items */}
      <nav className="flex-1 py-2 overflow-y-auto overflow-x-hidden">
        {NAV.map(({ id, Icon, label }) => {
          const active = current === id;
          return (
            <button
              key={id}
              onClick={() => onNavigate(id)}
              className={[
                'w-full flex items-center gap-3 px-3 py-2.5 text-left transition-colors duration-100 relative group',
                active
                  ? 'text-accent bg-accent-glow'
                  : 'text-ink-faint hover:text-ink hover:bg-surface-800',
              ].join(' ')}
            >
              {active && (
                <span className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-5 bg-accent rounded-r-full" />
              )}
              <Icon size={16} className="shrink-0" />
              {expanded && (
                <span className="text-sm font-medium whitespace-nowrap overflow-hidden">{label}</span>
              )}
              {/* Tooltip when collapsed */}
              {!expanded && (
                <span className="absolute left-full ml-2 px-2 py-1 bg-surface-700 text-ink text-xs rounded-md opacity-0 group-hover:opacity-100 pointer-events-none whitespace-nowrap z-50 border border-bdr">
                  {label}
                </span>
              )}
            </button>
          );
        })}
      </nav>

      {/* Bottom */}
      <div className="border-t border-bdr py-2">
        <button
          onClick={() => onNavigate('Settings')}
          className={[
            'w-full flex items-center gap-3 px-3 py-2.5 transition-colors duration-100',
            current === 'Settings'
              ? 'text-accent'
              : 'text-ink-faint hover:text-ink hover:bg-surface-800',
          ].join(' ')}
        >
          <Settings size={16} className="shrink-0" />
          {expanded && <span className="text-sm font-medium">Settings</span>}
        </button>
        <button
          onClick={() => setExpanded(e => !e)}
          className="w-full flex items-center justify-center py-2 text-ink-faint hover:text-ink transition-colors"
        >
          {expanded ? <ChevronLeft size={14} /> : <ChevronRight size={14} />}
        </button>
      </div>
    </aside>
  );
}
