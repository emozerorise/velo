import { onMounted, onScopeDispose } from 'vue';
import { useWorkspaceStore } from '@/stores/workspaceStore';

export function useWorkspaceLayout(): void {
  const workspace = useWorkspaceStore();
  const measure = () => {
    workspace.compact = window.innerHeight < 650 || window.innerWidth < 900;
  };
  onMounted(() => {
    measure();
    window.addEventListener('resize', measure);
  });
  onScopeDispose(() => window.removeEventListener('resize', measure));
}
