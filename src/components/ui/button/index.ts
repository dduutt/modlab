import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export { default as Button } from './Button.vue'

export const buttonVariants = cva(
  'inline-flex shrink-0 items-center justify-center whitespace-nowrap rounded-lg text-xs font-medium transition-colors outline-none select-none disabled:pointer-events-none disabled:opacity-50 focus-visible:ring-2 focus-visible:ring-blue-400 cursor-pointer',
  {
    variants: {
      variant: {
        default: 'bg-blue-600 text-white hover:bg-blue-700 shadow-xs border border-transparent',
        outline: 'border border-gray-200 bg-white text-gray-700 hover:bg-gray-50 shadow-2xs',
        secondary: 'bg-gray-100 text-gray-700 hover:bg-gray-200 border border-transparent',
        ghost: 'text-gray-600 hover:bg-gray-100 hover:text-gray-900',
        destructive: 'bg-red-600 text-white hover:bg-red-700 shadow-xs border border-transparent',
        link: 'text-blue-600 underline-offset-4 hover:underline',
      },
      size: {
        'default': 'h-8 gap-1.5 px-3',
        'xs': 'h-6 gap-1 rounded-md px-2 text-[11px]',
        'sm': 'h-7 gap-1 rounded-md px-2.5 text-xs',
        'lg': 'h-9 gap-2 px-4 text-sm',
        'icon': 'size-8 p-1',
        'icon-xs': 'size-6 rounded-md p-0.5',
        'icon-sm': 'size-7 rounded-md p-1',
        'icon-lg': 'size-9 p-1.5',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  },
)
export type ButtonVariants = VariantProps<typeof buttonVariants>
