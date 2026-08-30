import { onMounted, onUnmounted } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { usePlayerStore } from '@/stores/playerStore';
import { usePlaylistStore } from '@/stores/playlistStore';
import { useTranscriptStore } from '@/stores/transcriptStore';
import { useToast } from './useToast';
import { useI18n } from './useI18n';

export function useKeyboardShortcuts() {
  const player = usePlayerStore();
  const playlist = usePlaylistStore();
  const transcript = useTranscriptStore();
  const { showToast } = useToast();
  const { t } = useI18n();

  async function openFileDialog() {
    try {
      const selected = await open({
        multiple: true,
        filters: [
          {
            name: t('dialog.videoFiles'),
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
          showToast(t('toast.seekBackward', { seconds: 30 }));
        } else {
          player.seek(-5, true);
          showToast(t('toast.seekBackward', { seconds: 5 }));
        }
        break;

      case 'ArrowRight':
        e.preventDefault();
        if (e.shiftKey) {
          player.seek(30, true);
          showToast(t('toast.seekForward', { seconds: 30 }));
        } else {
          player.seek(5, true);
          showToast(t('toast.seekForward', { seconds: 5 }));
        }
        break;

      case 'ArrowUp':
        e.preventDefault();
        player.setVolume(Math.min(100, player.volume + 5));
        showToast(t('toast.volume', { percent: Math.round(player.volume) }));
        break;

      case 'ArrowDown':
        e.preventDefault();
        player.setVolume(Math.max(0, player.volume - 5));
        showToast(t('toast.volume', { percent: Math.round(player.volume) }));
        break;

      case 'KeyM':
        e.preventDefault();
        player.toggleMute();
        showToast(player.muted ? t('toast.muted') : t('toast.unmuted'));
        break;

      case 'KeyT':
        e.preventDefault();
        transcript.togglePanel();
        break;

      case 'KeyF':
        e.preventDefault();
        void toggleFullscreen();
        break;

      case 'BracketLeft': {
        e.preventDefault();
        const newSpeed = Math.max(0.25, Math.round((player.speed - 0.25) * 100) / 100);
        player.setSpeed(newSpeed);
        showToast(t('toast.speed', { speed: newSpeed }));
        break;
      }

      case 'BracketRight': {
        e.preventDefault();
        const newSpeed = Math.min(3.0, Math.round((player.speed + 0.25) * 100) / 100);
        player.setSpeed(newSpeed);
        showToast(t('toast.speed', { speed: newSpeed }));
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
