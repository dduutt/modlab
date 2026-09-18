<script setup lang="ts">
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';

defineProps<{
  show: boolean;
  title: string;
  message: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'confirm'): void;
}>();
</script>

<template>
  <AlertDialog :open="show" @update:open="(val: boolean) => { if (!val) emit('close'); }">
    <AlertDialogContent class="max-w-sm rounded-xl p-5 select-none bg-white">
      <AlertDialogHeader class="space-y-2">
        <AlertDialogTitle class="text-base font-semibold text-gray-900">
          {{ title }}
        </AlertDialogTitle>
        <AlertDialogDescription class="text-sm text-gray-600">
          {{ message }}
        </AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter class="mt-4 gap-2">
        <AlertDialogCancel @click="emit('close')" class="rounded-lg text-xs font-medium cursor-pointer">
          {{ $t('common.cancel') }}
        </AlertDialogCancel>
        <AlertDialogAction
          @click="emit('confirm')"
          class="rounded-lg bg-red-600 hover:bg-red-700 text-white text-xs font-medium shadow-xs cursor-pointer border-transparent"
        >
          {{ $t('common.confirm') }}
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>
