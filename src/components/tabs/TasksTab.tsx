import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Plus, Trash2 } from 'lucide-react';
import { TASK_COLOR_MAP } from '../../constants';
import type { TaskItem, TaskPriorityColor } from '../../types';

interface TasksTabProps {
  tasks: TaskItem[];
  onToggleTask: (id: string) => void;
  onAddTask: (title: string, color: TaskPriorityColor) => void;
  onDeleteTask: (id: string) => void;
  onClearCompleted?: () => void;
  onScrollTrapChange?: (trapped: boolean) => void;
}

const COLOR_ORDER: TaskPriorityColor[] = ['cyan', 'emerald', 'amber', 'violet', 'rose'];

export const TasksTab: React.FC<TasksTabProps> = ({
  tasks,
  onToggleTask,
  onAddTask,
  onDeleteTask,
  onClearCompleted,
  onScrollTrapChange,
}) => {
  const [draft, setDraft] = useState('');
  const [colorIdx, setColorIdx] = useState(0);

  const remainingCount = tasks.filter((t) => !t.completed).length;
  const completedCount = tasks.length - remainingCount;
  const selectedColor = COLOR_ORDER[colorIdx];

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = draft.trim();
    if (!trimmed) return;
    onAddTask(trimmed, selectedColor);
    setDraft('');
  };

  const cycleColor = () => {
    setColorIdx((i) => (i + 1) % COLOR_ORDER.length);
  };

  return (
    <div className="w-full h-full px-4 pb-3 flex flex-col justify-between select-none">
      {/* Header + Quick Add Input */}
      <div
        className="flex items-center gap-2"
        onPointerDown={(e) => e.stopPropagation()}
      >
        <div className="flex items-center gap-1.5 shrink-0">
          <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-white/[0.08] border border-white/10 text-[10.5px] font-semibold text-white/85 tabular-nums">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
            {remainingCount} left
          </span>
          {completedCount > 0 && onClearCompleted && (
            <button
              type="button"
              onClick={onClearCompleted}
              className="px-2 py-0.5 rounded-full bg-white/[0.06] hover:bg-rose-500/20 border border-white/10 hover:border-rose-400/30 text-[9.5px] font-medium text-white/60 hover:text-rose-200 cursor-pointer transition-colors"
              title="Clear completed tasks"
            >
              Clear Done
            </button>
          )}
        </div>

        <form onSubmit={handleSubmit} className="flex-1 flex items-center gap-1.5 min-w-0">
          <div className="flex-1 flex items-center gap-1.5 bg-white/[0.06] border border-white/10 focus-within:border-white/25 rounded-xl px-2.5 h-7 transition-colors">
            <button
              type="button"
              onClick={cycleColor}
              title={`Priority color: ${TASK_COLOR_MAP[selectedColor].label} (click to cycle)`}
              className="w-2.5 h-2.5 rounded-full shrink-0 cursor-pointer transition-transform hover:scale-125"
              style={{
                backgroundColor: TASK_COLOR_MAP[selectedColor].dot,
                boxShadow: `0 0 8px ${TASK_COLOR_MAP[selectedColor].ring}`,
              }}
              aria-label="Cycle reminder priority color"
            />
            <input
              type="text"
              value={draft}
              onChange={(e) => setDraft(e.target.value)}
              placeholder="Add reminder..."
              className="w-full bg-transparent text-[11.5px] text-white placeholder:text-white/35 focus:outline-none"
            />
          </div>

          <motion.button
            type="submit"
            whileHover={{ scale: 1.06 }}
            whileTap={{ scale: 0.9 }}
            disabled={!draft.trim()}
            className="w-7 h-7 rounded-xl bg-white/12 hover:bg-white/20 disabled:opacity-35 border border-white/10 flex items-center justify-center text-white cursor-pointer transition-colors shrink-0"
            aria-label="Add reminder"
          >
            <Plus className="w-3.5 h-3.5" />
          </motion.button>
        </form>
      </div>

      {/* Scrollable Checklist with Scroll Trap Isolation (Section 3) */}
      <div
        className="mt-2 flex-1 overflow-y-auto isle-scroll space-y-1 pr-0.5"
        onPointerDown={(e) => e.stopPropagation()}
        onMouseEnter={() => onScrollTrapChange?.(true)}
        onMouseLeave={() => onScrollTrapChange?.(false)}
        onWheel={(e) => e.stopPropagation()}
      >
        <AnimatePresence initial={false}>
          {tasks.length === 0 ? (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className="h-full flex items-center justify-center text-[11.5px] text-white/40"
            >
              All clear — add a quick reminder above
            </motion.div>
          ) : (
            tasks.map((task) => {
              const palette = TASK_COLOR_MAP[task.color] ?? TASK_COLOR_MAP.cyan;
              return (
                <motion.div
                  key={task.id}
                  layout
                  initial={{ opacity: 0, y: -6 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, x: -16, transition: { duration: 0.16 } }}
                  className="group flex items-center gap-2.5 px-2.5 py-1.5 rounded-xl bg-white/[0.04] hover:bg-white/[0.08] border border-white/[0.05] transition-colors"
                >
                  <button
                    type="button"
                    onClick={() => onToggleTask(task.id)}
                    className="flex items-center gap-2.5 flex-1 min-w-0 text-left cursor-pointer"
                  >
                    {/* Animated Apple Reminders Checkbox */}
                    <span
                      className="relative w-4 h-4 rounded-full flex items-center justify-center shrink-0 transition-colors"
                      style={{
                        border: `1.5px solid ${palette.dot}`,
                        backgroundColor: task.completed ? palette.dot : 'transparent',
                        boxShadow: task.completed ? `0 0 8px ${palette.ring}` : 'none',
                      }}
                    >
                      <AnimatePresence>
                        {task.completed && (
                          <motion.svg
                            viewBox="0 0 12 12"
                            className="w-2.5 h-2.5 text-slate-950"
                            initial={{ scale: 0.5, opacity: 0 }}
                            animate={{ scale: 1, opacity: 1 }}
                            exit={{ scale: 0.5, opacity: 0 }}
                            transition={{ duration: 0.16 }}
                          >
                            <motion.path
                              d="M2.5 6.2 L5 8.6 L9.5 3.5"
                              fill="none"
                              stroke="currentColor"
                              strokeWidth="2"
                              strokeLinecap="round"
                              strokeLinejoin="round"
                              initial={{ pathLength: 0 }}
                              animate={{ pathLength: 1 }}
                              transition={{ duration: 0.2 }}
                            />
                          </motion.svg>
                        )}
                      </AnimatePresence>
                    </span>

                    {/* Priority Dot + Title */}
                    <span
                      className={`text-[12px] truncate transition-all duration-200 ${
                        task.completed
                          ? 'line-through text-white/35'
                          : 'text-white/90 font-medium'
                      }`}
                    >
                      {task.title}
                    </span>
                  </button>

                  <button
                    type="button"
                    onClick={() => onDeleteTask(task.id)}
                    className="opacity-0 group-hover:opacity-100 text-white/35 hover:text-rose-400 transition-all cursor-pointer p-0.5"
                    aria-label={`Delete ${task.title}`}
                  >
                    <Trash2 className="w-3 h-3" />
                  </button>
                </motion.div>
              );
            })
          )}
        </AnimatePresence>
      </div>
    </div>
  );
};
