import React, { useMemo, useRef, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  BatteryCharging,
  Camera,
  Check,
  ClipboardCopy,
  Copy,
  ExternalLink,
  FileArchive,
  FileCode2,
  FileImage,
  FileText,
  FolderUp,
  Pin,
  Search,
  Smartphone,
  Video,
  Wifi,
  X,
} from 'lucide-react';
import type { ClipboardBadge, ClipboardItem, ScrcpyState, ScreenshotRecord, ShelfFile } from '../../types';
import { formatTime } from '../../services/lyrics';

interface ShelfTabProps {
  files: ShelfFile[];
  scrcpy: ScrcpyState;
  clipboardItems: ClipboardItem[];
  screenshots?: ScreenshotRecord[];
  onAddFiles: (files: FileList | ShelfFile[]) => void;
  onRemoveFile: (id: string) => void;
  onClearFiles: () => void;
  onToggleMirror: () => void;
  onToggleRecord: () => void;
  onToggleWirelessAdb: () => void;
  onCopyClipboard: () => void;
  onToggleClipboardPin: (id: string) => void;
  onCopyClipboardItem: (item: ClipboardItem) => void;
  onOpenScreenshot?: (record: ScreenshotRecord) => void;
  onCopyScreenshotPath?: (record: ScreenshotRecord) => void;
  onScrollTrapChange?: (trapped: boolean) => void;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function inferKind(name: string): ShelfFile['kind'] {
  const lower = name.toLowerCase();
  if (/\.(png|jpe?g|webp|gif|svg)$/.test(lower)) return 'image';
  if (/\.(ts|tsx|js|jsx|rs|py|json|css|html)$/.test(lower)) return 'code';
  if (/\.(zip|tar|gz|7z|rar)$/.test(lower)) return 'archive';
  if (/\.(pdf|md|txt|docx?)$/.test(lower)) return 'doc';
  return 'other';
}

function FileKindIcon({ kind }: { kind: ShelfFile['kind'] }) {
  switch (kind) {
    case 'image':
      return <FileImage className="w-3.5 h-3.5 text-sky-400 shrink-0" />;
    case 'code':
      return <FileCode2 className="w-3.5 h-3.5 text-violet-400 shrink-0" />;
    case 'archive':
      return <FileArchive className="w-3.5 h-3.5 text-amber-400 shrink-0" />;
    default:
      return <FileText className="w-3.5 h-3.5 text-emerald-400 shrink-0" />;
  }
}

const BADGE_STYLES: Record<ClipboardBadge, string> = {
  CODE: 'bg-violet-400/20 text-violet-300 border-violet-400/30',
  URL: 'bg-sky-400/20 text-sky-300 border-sky-400/30',
  HEX: 'bg-pink-400/20 text-pink-300 border-pink-400/30',
  TEXT: 'bg-emerald-400/20 text-emerald-300 border-emerald-400/30',
};

export const ShelfTab: React.FC<ShelfTabProps> = ({
  files,
  scrcpy,
  clipboardItems,
  screenshots = [],
  onAddFiles,
  onRemoveFile,
  onClearFiles,
  onToggleMirror,
  onToggleRecord,
  onToggleWirelessAdb,
  onCopyClipboard,
  onToggleClipboardPin,
  onCopyClipboardItem,
  onOpenScreenshot,
  onCopyScreenshotPath,
  onScrollTrapChange,
}) => {
  // Segmented toggle: Shelf (Staged Files & Screenshots) vs Clipboard History
  const [segment, setSegment] = useState<'shelf' | 'clipboard'>('shelf');
  const [isDraggingOver, setIsDraggingOver] = useState(false);
  const [clipboardQuery, setClipboardQuery] = useState('');
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDraggingOver(false);
    if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
      onAddFiles(e.dataTransfer.files);
    }
  };

  const handleAddSample = (e: React.MouseEvent) => {
    e.stopPropagation();
    const samples: ShelfFile[] = [
      {
        id: `sample-${Date.now()}`,
        name: 'isle-spec-final.fig',
        sizeBytes: 2_480_000,
        kind: 'image',
        addedAt: Date.now(),
      },
    ];
    onAddFiles(samples);
  };

  const filteredClipboard = useMemo(() => {
    const q = clipboardQuery.trim().toLowerCase();
    const sorted = [...clipboardItems].sort((a, b) => Number(b.pinned) - Number(a.pinned));
    if (!q) return sorted.slice(0, 10);
    return sorted
      .filter(
        (item) =>
          item.content.toLowerCase().includes(q) ||
          item.badge.toLowerCase().includes(q)
      )
      .slice(0, 10);
  }, [clipboardItems, clipboardQuery]);

  const handleCopySnippet = (item: ClipboardItem) => {
    onCopyClipboardItem(item);
    setCopiedId(item.id);
    window.setTimeout(() => {
      setCopiedId((cur) => (cur === item.id ? null : cur));
    }, 1400);
  };

  return (
    <div className="w-full h-full px-4 pb-2.5 flex flex-col justify-between select-none">
      {/* Top Segmented Control: [ Staged Files & Snips | Clipboard History ] */}
      <div
        className="flex items-center justify-between pb-1"
        onPointerDown={(e) => e.stopPropagation()}
      >
        <div className="inline-flex p-0.5 rounded-xl bg-white/[0.06] border border-white/10">
          <button
            type="button"
            onClick={() => setSegment('shelf')}
            className={`px-3 py-1 rounded-lg text-[10.5px] font-semibold transition-all cursor-pointer ${
              segment === 'shelf'
                ? 'bg-white/20 text-white shadow-sm'
                : 'text-white/50 hover:text-white/80'
            }`}
          >
            Staged Files ({files.length + screenshots.length})
          </button>
          <button
            type="button"
            onClick={() => setSegment('clipboard')}
            className={`px-3 py-1 rounded-lg text-[10.5px] font-semibold transition-all cursor-pointer ${
              segment === 'clipboard'
                ? 'bg-white/20 text-white shadow-sm'
                : 'text-white/50 hover:text-white/80'
            }`}
          >
            Clipboard History ({clipboardItems.length})
          </button>
        </div>

        {segment === 'shelf' && files.length > 0 && (
          <button
            type="button"
            onClick={onClearFiles}
            className="text-[10px] text-white/45 hover:text-rose-300 transition-colors cursor-pointer"
          >
            Clear Shelf
          </button>
        )}
      </div>

      <AnimatePresence mode="wait">
        {segment === 'shelf' ? (
          /* Segment 1: Shelf & Phone & Screenshot Hub */
          <motion.div
            key="segment-shelf"
            initial={{ opacity: 0, y: 4 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -4 }}
            transition={{ duration: 0.14 }}
            className="flex-1 flex items-stretch gap-3 min-h-0"
          >
            {/* Left Box: Drag & Drop Dropzone + File Cards */}
            <div
              onPointerDown={(e) => e.stopPropagation()}
              onDragOver={(e) => {
                e.preventDefault();
                setIsDraggingOver(true);
              }}
              onDragLeave={() => setIsDraggingOver(false)}
              onDrop={handleDrop}
              onClick={() => fileInputRef.current?.click()}
              onMouseEnter={() => onScrollTrapChange?.(true)}
              onMouseLeave={() => onScrollTrapChange?.(false)}
              onWheel={(e) => e.stopPropagation()}
              className={`w-[172px] shrink-0 rounded-2xl border border-dashed p-2 flex flex-col justify-between transition-all cursor-pointer overflow-hidden ${
                isDraggingOver
                  ? 'border-violet-400 bg-violet-400/15 scale-[1.01]'
                  : 'border-white/15 bg-white/[0.04] hover:bg-white/[0.07] hover:border-white/25'
              }`}
            >
              <input
                ref={fileInputRef}
                type="file"
                multiple
                className="hidden"
                onChange={(e) => {
                  if (e.target.files && e.target.files.length > 0) {
                    onAddFiles(e.target.files);
                    e.target.value = '';
                  }
                }}
              />

              {files.length === 0 && screenshots.length === 0 ? (
                <div className="flex-1 flex flex-col items-center justify-center text-center px-1">
                  <FolderUp
                    className={`w-5 h-5 mb-1 transition-colors ${
                      isDraggingOver ? 'text-violet-300' : 'text-white/40'
                    }`}
                  />
                  <p className="text-[10.5px] text-white/70 leading-tight">Drop files here</p>
                  <p className="text-[9px] text-white/40 mt-0.5">or press PrtScn to snip</p>
                  <button
                    type="button"
                    onClick={handleAddSample}
                    className="mt-2 px-2 py-0.5 rounded-full bg-white/10 hover:bg-white/20 text-[9px] text-violet-300 cursor-pointer"
                  >
                    + Sample File
                  </button>
                </div>
              ) : (
                <div className="flex-1 space-y-1 overflow-y-auto isle-scroll pr-0.5">
                  {/* Screenshots section */}
                  {screenshots.map((snip) => (
                    <div
                      key={snip.id}
                      onClick={(e) => e.stopPropagation()}
                      className="group flex items-center gap-1.5 px-2 py-1 rounded-lg bg-sky-500/10 border border-sky-400/25"
                    >
                      <Camera className="w-3.5 h-3.5 text-sky-300 shrink-0" />
                      <div className="min-w-0 flex-1">
                        <p className="text-[10px] font-semibold text-sky-100 truncate leading-tight">
                          {snip.fileName}
                        </p>
                        <p className="text-[8.5px] text-sky-200/50 truncate font-mono">
                          JSON metadata
                        </p>
                      </div>
                      <button
                        type="button"
                        onClick={() => onCopyScreenshotPath?.(snip)}
                        className="text-sky-300 hover:text-white cursor-pointer"
                        title="Copy file path"
                      >
                        <Copy className="w-2.5 h-2.5" />
                      </button>
                      <button
                        type="button"
                        onClick={() => onOpenScreenshot?.(snip)}
                        className="text-sky-300 hover:text-white cursor-pointer"
                        title="Open screenshot"
                      >
                        <ExternalLink className="w-2.5 h-2.5" />
                      </button>
                    </div>
                  ))}

                  {/* Regular files */}
                  {files.map((file) => (
                    <div
                      key={file.id}
                      onClick={(e) => e.stopPropagation()}
                      className="flex items-center gap-1.5 px-2 py-1 rounded-lg bg-white/[0.07] border border-white/[0.08]"
                    >
                      <FileKindIcon kind={file.kind || inferKind(file.name)} />
                      <div className="min-w-0 flex-1">
                        <p className="text-[10px] font-medium text-white/90 truncate leading-tight">
                          {file.name}
                        </p>
                        <p className="text-[8.5px] text-white/40 tabular-nums">
                          {formatBytes(file.sizeBytes)}
                        </p>
                      </div>
                      <button
                        type="button"
                        onClick={() => onRemoveFile(file.id)}
                        className="text-white/35 hover:text-rose-300 cursor-pointer"
                        aria-label={`Remove ${file.name}`}
                      >
                        <X className="w-2.5 h-2.5" />
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Right Box: Phone Companion & Scrcpy Launchers */}
            <div
              className="flex-1 flex flex-col justify-between min-w-0"
              onPointerDown={(e) => e.stopPropagation()}
            >
              {/* Phone Status Header */}
              <div className="flex items-center justify-between px-2.5 py-1.5 rounded-xl bg-white/[0.05] border border-white/[0.08]">
                <div className="flex items-center gap-1.5 min-w-0">
                  <span
                    className={`w-2 h-2 rounded-full shrink-0 ${
                      scrcpy.connected
                        ? 'bg-emerald-400'
                        : 'bg-white/30'
                    }`}
                  />
                  <span className="text-[11px] font-semibold text-white truncate">
                    {scrcpy.deviceName}
                  </span>
                </div>

                <div className="flex items-center gap-1.5 shrink-0">
                  {scrcpy.toast ? (
                    <span className="px-1.5 py-0.5 rounded-md bg-emerald-400/20 text-emerald-300 text-[9px] font-semibold">
                      {scrcpy.toast}
                    </span>
                  ) : scrcpy.recording ? (
                    <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-md bg-rose-500/25 text-rose-300 text-[9px] font-semibold tabular-nums">
                      <span className="w-1.5 h-1.5 rounded-full bg-rose-400 animate-pulse" />
                      REC {formatTime(scrcpy.recordingSec)}
                    </span>
                  ) : (
                    <span className="inline-flex items-center gap-1 text-[10px] font-medium text-emerald-300 tabular-nums">
                      <BatteryCharging className="w-3 h-3" />
                      {scrcpy.batteryPct}%
                    </span>
                  )}
                </div>
              </div>

              {/* 2x2 Quick Launch Grid */}
              <div className="grid grid-cols-2 gap-1.5 mt-1.5 flex-1">
                <motion.button
                  type="button"
                  whileTap={{ scale: 0.94 }}
                  onClick={onToggleMirror}
                  className={`rounded-xl px-2.5 py-1.5 border text-left flex flex-col justify-between transition-colors cursor-pointer ${
                    scrcpy.mirroring
                      ? 'bg-violet-500/25 border-violet-400/50 text-white'
                      : 'bg-white/[0.05] hover:bg-white/[0.09] border-white/[0.08] text-white/80'
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <Smartphone className="w-3.5 h-3.5 text-violet-300" />
                    <span
                      className={`text-[8.5px] font-semibold px-1.5 py-0.2 rounded-full ${
                        scrcpy.mirroring
                          ? 'bg-violet-400/30 text-violet-200'
                          : 'bg-white/10 text-white/50'
                      }`}
                    >
                      {scrcpy.mirroring ? 'LIVE' : 'SCRCPY'}
                    </span>
                  </div>
                  <span className="text-[10px] font-medium leading-tight mt-1">
                    Mirror Phone
                  </span>
                </motion.button>

                <motion.button
                  type="button"
                  whileTap={{ scale: 0.94 }}
                  onClick={onToggleRecord}
                  className={`rounded-xl px-2.5 py-1.5 border text-left flex flex-col justify-between transition-colors cursor-pointer ${
                    scrcpy.recording
                      ? 'bg-rose-500/25 border-rose-400/50 text-white'
                      : 'bg-white/[0.05] hover:bg-white/[0.09] border-white/[0.08] text-white/80'
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <Video className="w-3.5 h-3.5 text-rose-300" />
                    <span
                      className={`text-[8.5px] font-semibold px-1.5 py-0.2 rounded-full ${
                        scrcpy.recording
                          ? 'bg-rose-400/30 text-rose-200'
                          : 'bg-white/10 text-white/50'
                      }`}
                    >
                      {scrcpy.recording ? 'STOP' : 'MP4'}
                    </span>
                  </div>
                  <span className="text-[10px] font-medium leading-tight mt-1">
                    Record Screen
                  </span>
                </motion.button>

                <motion.button
                  type="button"
                  whileTap={{ scale: 0.94 }}
                  onClick={onToggleWirelessAdb}
                  className={`rounded-xl px-2.5 py-1.5 border text-left flex flex-col justify-between transition-colors cursor-pointer ${
                    scrcpy.wirelessAdb
                      ? 'bg-sky-500/25 border-sky-400/50 text-white'
                      : 'bg-white/[0.05] hover:bg-white/[0.09] border-white/[0.08] text-white/80'
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <Wifi className="w-3.5 h-3.5 text-sky-300" />
                    <span
                      className={`text-[8.5px] font-semibold px-1.5 py-0.2 rounded-full ${
                        scrcpy.wirelessAdb
                          ? 'bg-sky-400/30 text-sky-200'
                          : 'bg-white/10 text-white/50'
                      }`}
                    >
                      {scrcpy.wirelessAdb ? '5555' : 'USB'}
                    </span>
                  </div>
                  <span className="text-[10px] font-medium leading-tight mt-1">
                    Wireless ADB
                  </span>
                </motion.button>

                <motion.button
                  type="button"
                  whileTap={{ scale: 0.94 }}
                  onClick={onCopyClipboard}
                  className="rounded-xl px-2.5 py-1.5 border bg-white/[0.05] hover:bg-white/[0.09] border-white/[0.08] text-white/80 text-left flex flex-col justify-between transition-colors cursor-pointer"
                >
                  <div className="flex items-center justify-between">
                    <ClipboardCopy className="w-3.5 h-3.5 text-emerald-300" />
                    <span className="text-[8.5px] font-semibold px-1.5 py-0.2 rounded-full bg-white/10 text-white/50">
                      SYNC
                    </span>
                  </div>
                  <span className="text-[10px] font-medium leading-tight mt-1">
                    Copy Phone Clip
                  </span>
                </motion.button>
              </div>
            </div>
          </motion.div>
        ) : (
          /* Segment 2: Clipboard History Segment */
          <motion.div
            key="segment-clipboard"
            initial={{ opacity: 0, y: 4 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -4 }}
            transition={{ duration: 0.14 }}
            className="flex-1 flex flex-col min-h-0"
            onPointerDown={(e) => e.stopPropagation()}
          >
            {/* Search Filter */}
            <div className="flex items-center gap-2 mb-1.5">
              <div className="flex-1 flex items-center gap-1.5 bg-white/[0.06] border border-white/10 focus-within:border-pink-400/45 rounded-xl px-2.5 h-7 transition-colors">
                <Search className="w-3 h-3 text-white/40 shrink-0" />
                <input
                  type="text"
                  value={clipboardQuery}
                  onChange={(e) => setClipboardQuery(e.target.value)}
                  placeholder="Search clipboard snippets (CODE, URL, HEX)..."
                  className="w-full bg-transparent text-[10.5px] text-white placeholder:text-white/35 focus:outline-none"
                />
              </div>
              <span className="text-[9.5px] font-medium text-white/45 tabular-nums shrink-0">
                {filteredClipboard.length} items
              </span>
            </div>

            {/* Scroll-Wheel Isolated Clipboard List */}
            <div
              className="flex-1 overflow-y-auto isle-scroll space-y-1 pr-0.5"
              onMouseEnter={() => onScrollTrapChange?.(true)}
              onMouseLeave={() => onScrollTrapChange?.(false)}
              onWheel={(e) => e.stopPropagation()}
            >
              {filteredClipboard.length === 0 ? (
                <div className="h-full flex items-center justify-center text-[11px] text-white/40">
                  No matching snippets
                </div>
              ) : (
                filteredClipboard.map((item) => {
                  const isCopied = copiedId === item.id;
                  return (
                    <div
                      key={item.id}
                      className="group flex items-center gap-2 px-2.5 py-1 rounded-xl bg-white/[0.04] hover:bg-white/[0.08] border border-white/[0.06] transition-colors"
                    >
                      <span
                        className={`px-1.5 py-0.5 rounded-md border text-[8.5px] font-bold tracking-wider shrink-0 ${BADGE_STYLES[item.badge]}`}
                      >
                        {item.badge}
                      </span>

                      {item.badge === 'HEX' && (
                        <span
                          className="w-3 h-3 rounded-full border border-white/25 shrink-0"
                          style={{ backgroundColor: item.content }}
                        />
                      )}

                      <span className="flex-1 min-w-0 text-[10.5px] text-white/85 font-mono truncate">
                        {item.content}
                      </span>

                      <span className="text-[9px] text-white/35 shrink-0 hidden sm:inline">
                        {item.copiedAt}
                      </span>

                      <button
                        type="button"
                        onClick={() => onToggleClipboardPin(item.id)}
                        className={`p-1 rounded-lg transition-colors cursor-pointer ${
                          item.pinned
                            ? 'text-amber-300 bg-amber-400/15'
                            : 'text-white/30 hover:text-white/75 opacity-0 group-hover:opacity-100'
                        }`}
                        title={item.pinned ? 'Unpin' : 'Pin to top'}
                      >
                        <Pin className="w-3 h-3" />
                      </button>

                      <motion.button
                        type="button"
                        whileTap={{ scale: 0.88 }}
                        onClick={() => handleCopySnippet(item)}
                        className={`p-1 rounded-lg transition-colors cursor-pointer ${
                          isCopied
                            ? 'text-emerald-300 bg-emerald-400/20'
                            : 'text-white/55 hover:text-white hover:bg-white/10'
                        }`}
                        title="Copy to clipboard"
                      >
                        {isCopied ? (
                          <Check className="w-3 h-3" />
                        ) : (
                          <Copy className="w-3 h-3" />
                        )}
                      </motion.button>
                    </div>
                  );
                })
              )}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};
