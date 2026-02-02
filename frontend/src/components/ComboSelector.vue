<script setup lang="ts">
import { CheckIcon, ChevronsUpDownIcon } from 'lucide-vue-next'
import { computed, ref } from 'vue'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command'
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover'

const props = defineProps<{
    options: any[]
    value: string
    placeholder: string
    prefix?: string
}>()

const open = ref(false)
const value = ref('')

const selectedOption = computed(() =>
  (props.options || []).find(option => option.value === value.value),
)

function selectOption(selectedValue: string) {
  value.value = selectedValue === value.value ? '' : selectedValue
  open.value = false
}
</script>

<template>
  <Popover v-model:open="open">
    <PopoverTrigger as-child>
      <Button
        variant="secondary"
        role="combobox"
        :aria-expanded="open"
        class="min-w-50 justify-between"
      >
        <span>
          <span v-if="prefix" class="text-muted-foreground mr-1">{{ prefix }}:</span>
          {{ selectedOption?.label || placeholder }}
        </span>
        <ChevronsUpDownIcon class="opacity-50" />
      </Button>
    </PopoverTrigger>
    <PopoverContent class="min-w-50 p-0">
      <Command :highlight-on-hover="true">
        <CommandInput class="h-9" placeholder="Search framework..." />
        <CommandList>
          <CommandEmpty>No framework found.</CommandEmpty>
          <CommandGroup>
            <CommandItem
              v-for="option in options"
              class="combo-option"
              :key="option.value"
              :value="option.value"
              @select="(ev) => {
                selectOption(ev.detail.value as string)
              }"
            >
              {{ option.label }}
              <CheckIcon
                :class="cn(
                  'ml-auto',
                  value === option.value ? 'opacity-100' : 'opacity-0',
                )"
              />
            </CommandItem>
          </CommandGroup>
        </CommandList>
      </Command>
    </PopoverContent>
  </Popover>
</template>