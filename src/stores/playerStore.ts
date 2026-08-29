import { defineStore } from 'pinia';
import { ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { playerService } from '@/services/playerService';
import type { PlaybackState, Track, MediaInfo, TimeUpdate } from '@/types/player';

export const usePlayerStore = defineStore('player', () => {
  const state = ref<PlaybackState>('idle');
  const currentTime = ref(0);
  const duration = ref(0);
  const percent = ref(0);
  const volume = ref(80);
  const muted = ref(false);
  const speed = ref(1.0);
  const mediaInfo = ref<MediaInfo | null>(null);
  const audioTracks = ref<Track[]>([]);
  const subtitleTracks = ref<Track[]>([]);
  const isFullscreen = ref(false);
  const isDraggingTimeline = ref(false);
  // Time under the scrubber while dragging, so the readout can follow the
  // cursor instead of the (frozen) playback position.
  const seekPreview = ref<number | null>(null);

  let unlistenFunctions: (() => void)[] = [];

  async function initListeners() {
    // Cleanup any existing listeners
    cleanupListeners();

    const u1 = await listen<PlaybackState>('velo://player-state', (event) => {
      state.value = event.payload;
    });

    const u2 = await listen<TimeUpdate>('velo://time-update', (event) => {
      if (!isDraggingTimeline.value) {
        currentTime.value = event.payload.current_time;
        duration.value = event.payload.duration;
        percent.value = event.payload.percent;
      }
    });

    const u3 = await listen<MediaInfo>('velo://media-loaded', (event) => {
      mediaInfo.value = event.payload;
      duration.value = event.payload.duration;
    });

    const u4 = await listen<Track[]>('velo://tracks-changed', (event) => {
      audioTracks.value = event.payload.filter((t) => t.type === 'audio');
      subtitleTracks.value = event.payload.filter((t) => t.type === 'sub');
    });

    const u5 = await listen<number>('velo://volume-changed', (event) => {
      volume.value = event.payload;
    });

    const u6 = await listen<boolean>('velo://mute-changed', (event) => {
      muted.value = event.payload;
    });

    const u7 = await listen<number>('velo://speed-changed', (event) => {
      speed.value = event.payload;
    });

    unlistenFunctions = [u1, u2, u3, u4, u5, u6, u7];
  }

  function cleanupListeners() {
    unlistenFunctions.forEach((unlisten) => unlisten());
    unlistenFunctions = [];
  }

  async function loadFile(path: string, startTime?: number) {
    try {
      state.value = 'loading';
      await playerService.loadFile(path, startTime);
    } catch (e) {
      state.value = 'error';
      console.error('Failed to load file:', e);
    }
  }

  async function togglePlay() {
    await playerService.togglePause();
  }

  async function play() {
    await playerService.play();
  }

  async function pause() {
    await playerService.pause();
  }

  async function seek(seconds: number, exact = true) {
    await playerService.seek(seconds, exact);
  }

  async function seekAbsolute(seconds: number) {
    await playerService.seekAbsolute(seconds);
  }

  async function setVolume(vol: number) {
    volume.value = vol;
    await playerService.setVolume(vol);
  }

  async function toggleMute() {
    muted.value = !muted.value;
    await playerService.setMute(muted.value);
  }

  async function setSpeed(newSpeed: number) {
    speed.value = newSpeed;
    await playerService.setSpeed(newSpeed);
  }

  async function selectAudioTrack(id: number) {
    await playerService.selectAudioTrack(id);
  }

  async function selectSubtitleTrack(id: number) {
    await playerService.selectSubtitleTrack(id);
  }

  async function addSubtitleFile(path: string) {
    await playerService.addSubtitleFile(path);
  }

  return {
    state,
    currentTime,
    duration,
    percent,
    volume,
    muted,
    speed,
    mediaInfo,
    audioTracks,
    subtitleTracks,
    isFullscreen,
    isDraggingTimeline,
    seekPreview,
    initListeners,
    cleanupListeners,
    loadFile,
    togglePlay,
    play,
    pause,
    seek,
    seekAbsolute,
    setVolume,
    toggleMute,
    setSpeed,
    selectAudioTrack,
    selectSubtitleTrack,
    addSubtitleFile,
  };
});
