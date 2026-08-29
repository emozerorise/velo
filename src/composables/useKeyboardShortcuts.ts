import { onMounted, onUnmounted } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { usePlayerStore } from '@/stores/playerStore';
import { usePlaylistStore } from '@/stores/playlistStore';
import { useToast } from './useToast';

export function useKeyboardShortcuts() {
  const player = usePlayerStore();
  const playlist = usePlaylistStore();
  const { showToast } = useToast();

  async function openFileDialog() {
    try {
      const selected = await open({
        multiple: true,
        filters: [
          {
            name: 'Video Files',
            extensions: ['mp4', 'mkv', 'mov', 'avi', 'webm', 'flv', 'm4v', 'ts', 'wmv'],
          },
        ],
      });

      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        playlist.addFiles(paths);
      }
    } catch (e) {
      console.error('Failed to open file dialog:', e);
    }
  }

  async function openDirectoryDialog() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });

      if (selected && typeof selected === 'string') {
        await playlist.addDirectory(selected);
      }
    } catch (e) {
      console.error('Failed to open directory dialog:', e);
    }
  }

  async function toggleFullscreen() {
    try {
      const appWindow = getCurrentWebviewWindow();
      const isFull = await appWindow.isFullscreen();
      await appWindow.setFullscreen(!isFull);
      player.isFullscreen = !isFull;
    } catch (e) {
      console.error('Failed to toggle fullscreen:', e);
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    // Ignore keystrokes inside text inputs
    if (
      e.target instanceof HTMLInputElement ||
      e.target instanceof HTMLTextAreaElement
    ) {
      return;
    }

    const isCmdOrCtrl = e.metaKey || e.ctrlKey;

    if (isCmdOrCtrl && e.key.toLowerCase() === 'o') {
      e.preventDefault();
      if (e.shiftKey) {
        void openDirectoryDialog();
      } else {
        void openFileDialog();
      }
      return;
    }

    switch (e.code) {
      case 'Space':
        e.preventDefault();
        player.togglePlay();
        break;

      case 'ArrowLeft':
        e.preventDefault();
        if (e.shiftKey) {
          player.seek(-30, true);
          showToast('Seek -30s');
        } else {
          player.seek(-5, true);
          showToast('Seek -5s');
        }
        break;

      case 'ArrowRight':
        e.preventDefault();
        if (e.shiftKey) {
          player.seek(30, true);
          showToast('Seek +30s');
        } else {
          player.seek(5, true);
          showToast('Seek +5s');
        }
        break;

      case 'ArrowUp':
        e.preventDefault();
        player.setVolume(Math.min(100, player.volume + 5));
        showToast(`Volume: ${Math.round(player.volume)}%`);
        break;

      case 'ArrowDown':
        e.preventDefault();
        player.setVolume(Math.max(0, player.volume - 5));
        showToast(`Volume: ${Math.round(player.volume)}%`);
        break;

      case 'KeyM':
        e.preventDefault();
        player.toggleMute();
        showToast(player.muted ? 'Muted' : 'Unmuted');
        break;

      case 'KeyF':
        e.preventDefault();
        void toggleFullscreen();
        break;

      case 'BracketLeft': {
        e.preventDefault();
        const newSpeed = Math.max(0.25, Math.round((player.speed - 0.25) * 100) / 100);
        player.setSpeed(newSpeed);
        showToast(`Speed: ${newSpeed}x`);
        break;
      }

      case 'BracketRight': {
        e.preventDefault();
        const newSpeed = Math.min(3.0, Math.round((player.speed + 0.25) * 100) / 100);
        player.setSpeed(newSpeed);
        showToast(`Speed: ${newSpeed}x`);
        break;
      }
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', handleKeyDown);
  });

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeyDown);
  });

  return {
    openFileDialog,
    openDirectoryDialog,
    toggleFullscreen,
  };
}
