// Deprecated per Anti-Bloatware Overhaul (Section 4)
import React from 'react';

export const Omnibar: React.FC<{
  open?: boolean;
  onClose?: () => void;
  onSelectTab?: (tab: any) => void;
  onOpenSettings?: () => void;
  onToggleDnd?: () => void;
  onToggleMicMute?: () => void;
  onLoadLrclibSong?: (title: string, artist: string) => void;
}> = () => null;

export default Omnibar;
