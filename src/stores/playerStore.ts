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

  /// Playback commands are fire-and-forget: nothing awaits them, so they
  /// report their own failures instead of returning a promise that would
  /// reject into nowhere. A silently dropped `invoke` is indistinguishable
  /// from a dead button.
  function dispatch(action: string, run: () => Promise<void>): void {
    run().catch((e: unknown) => {
      console.error(`Player command "${action}" failed:`, e);
    });
  }

  function loadFile(path: string, startTime?: number): void {
    state.value = 'loading';
    playerService.loadFile(path, startTime).catch((e: unknown) => {
      state.value = 'error';
      console.error('Failed to load file:', e);
    });
  }

  function togglePlay(): void {
    dispatch('togglePlay', () => playerService.togglePause());
  }

  function play(): void {
    dispatch('play', () => playerService.play());
  }

  function pause(): void {
    dispatch('pause', () => playerService.pause());
  }

  function seek(seconds: number, exact = true): void {
    dispatch('seek', () => playerService.seek(seconds, exact));
  }

  function seekAbsolute(seconds: number): void {
    dispatch('seekAbsolute', () => playerService.seekAbsolute(seconds));
  }

  function setVolume(vol: number): void {
    volume.value = vol;
    dispatch('setVolume', () => playerService.setVolume(vol));
  }

  function toggleMute(): void {
    muted.value = !muted.value;
    dispatch('setMute', () => playerService.setMute(muted.value));
  }

  function setSpeed(newSpeed: number): void {
    speed.value = newSpeed;
    dispatch('setSpeed', () => playerService.setSpeed(newSpeed));
  }

  function selectAudioTrack(id: number): void {
    dispatch('selectAudioTrack', () => playerService.selectAudioTrack(id));
  }

  function selectSubtitleTrack(id: number): void {
    dispatch('selectSubtitleTrack', () => playerService.selectSubtitleTrack(id));
  }

  function addSubtitleFile(path: string): void {
    dispatch('addSubtitleFile', () => playerService.addSubtitleFile(path));
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
