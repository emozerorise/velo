import { onMounted, onScopeDispose } from 'vue';
import { useWorkspaceStore } from '@/stores/workspaceStore';

export function useWorkspaceLayout(): void {
  const workspace = useWorkspaceStore();
  const measure = () => {
    workspace.narrow = window.innerWidth < 900;
    workspace.compact = window.innerHeight < 650 || workspace.narrow;
  };
  onMounted(() => {
    measure();
    window.addEventListener('resize', measure);
  });
  onScopeDispose(() => window.removeEventListener('resize', measure));
}
