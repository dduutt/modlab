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
    <AlertDialogContent class="max-w-sm rounded-2xl p-6 select-none bg-white border border-gray-100 shadow-2xl">
      <AlertDialogHeader class="space-y-2">
        <AlertDialogTitle class="text-base font-semibold text-gray-900">
          {{ title }}
        </AlertDialogTitle>
        <AlertDialogDescription class="text-sm text-gray-600 leading-relaxed">
          {{ message }}
        </AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter class="mt-5 gap-2 sm:justify-end">
        <AlertDialogCancel
          @click="emit('close')"
          class="rounded-xl border border-gray-200 text-gray-600 hover:bg-gray-50 text-xs font-medium px-4 py-1.5 transition cursor-pointer"
        >
          {{ $t('common.cancel') }}
        </AlertDialogCancel>
        <AlertDialogAction
          @click="emit('confirm')"
          class="rounded-xl bg-red-600 hover:bg-red-700 text-white text-xs font-medium shadow-xs px-4 py-1.5 transition cursor-pointer border-transparent"
        >
          {{ $t('common.confirm') }}
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>
