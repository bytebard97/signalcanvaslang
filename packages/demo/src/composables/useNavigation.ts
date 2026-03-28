// packages/demo/src/composables/useNavigation.ts
import { ref, computed } from 'vue'

export interface NavEntry {
  instanceName: string
  templateName: string
}

export function useNavigation() {
  // stack[0] is always null (root); subsequent entries are drill targets
  const stack = ref<Array<NavEntry | null>>([null])

  const currentEntry = computed(() => stack.value[stack.value.length - 1])
  const currentTemplateName = computed((): string | null => currentEntry.value?.templateName ?? null)

  const breadcrumbs = computed(() =>
    stack.value.map((entry, index) => ({
      label: entry?.instanceName ?? 'Root',
      index,
    })),
  )

  function drillInto(entry: NavEntry): void {
    stack.value = [...stack.value, entry]
  }

  /** Truncate stack to the given index (breadcrumb click). */
  function navigateTo(index: number): void {
    stack.value = stack.value.slice(0, index + 1)
  }

  function reset(): void {
    stack.value = [null]
  }

  return { stack, currentEntry, currentTemplateName, breadcrumbs, drillInto, navigateTo, reset }
}
