import { BookOpen, Music, Image, Star, Clock, Download } from 'lucide-react';

const CATEGORIES = [
  { icon: BookOpen, label: 'Scriptures',    count: 31_102 },
  { icon: Music,    label: 'Songs',         count: 4 },
  { icon: Image,    label: 'Backgrounds',   count: 0 },
  { icon: Star,     label: 'Favorites',     count: 0 },
  { icon: Clock,    label: 'Recent',        count: 12 },
  { icon: Download, label: 'Downloads',     count: 0 },
];

export default function Library() {
  return (
    <div className="h-full overflow-y-auto bg-surface-900 p-6">
      <div className="max-w-4xl mx-auto">
        <div className="mb-6">
          <h1 className="text-xl font-bold text-ink">Library</h1>
          <p className="text-sm text-ink-faint mt-1">All your content in one place</p>
        </div>

        <div className="grid grid-cols-3 gap-3">
          {CATEGORIES.map(({ icon: Icon, label, count }) => (
            <button
              key={label}
              className="flex flex-col gap-3 p-5 bg-surface-800 border border-bdr rounded-xl hover:border-bdr-strong hover:bg-surface-700 transition-all text-left group"
            >
              <div className="w-10 h-10 rounded-lg bg-surface-700 border border-bdr flex items-center justify-center group-hover:border-accent/40 transition-colors">
                <Icon size={18} className="text-ink-muted group-hover:text-accent transition-colors" />
              </div>
              <div>
                <div className="text-sm font-semibold text-ink">{label}</div>
                <div className="text-2xs text-ink-faint mt-0.5">
                  {count > 0 ? count.toLocaleString() + ' items' : 'Empty'}
                </div>
              </div>
            </button>
          ))}
        </div>

        <div className="mt-8 p-5 bg-surface-800 border border-bdr rounded-xl">
          <h2 className="text-sm font-bold text-ink mb-1">Bible Translations</h2>
          <p className="text-xs text-ink-faint mb-3">Download additional Bible translations for offline use.</p>
          <button className="flex items-center gap-2 px-4 py-2 bg-accent text-surface-950 rounded-lg text-xs font-bold hover:brightness-110 transition-all">
            <Download size={13} /> Browse Translations
          </button>
        </div>
      </div>
    </div>
  );
}
