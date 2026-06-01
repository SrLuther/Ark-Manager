import { cn } from '../../utils/helpers'
import { type HTMLAttributes } from 'react'

export type BadgeVariant = 'default' | 'success' | 'warning' | 'error' | 'info' | 'purple'

export interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  variant?: BadgeVariant
}

const variantClasses: Record<BadgeVariant, string> = {
  default: 'bg-surface-700 text-slate-300',
  success: 'bg-emerald-900/50 text-emerald-400 border border-emerald-800',
  warning: 'bg-yellow-900/50 text-yellow-400 border border-yellow-800',
  error:   'bg-red-900/50   text-red-400   border border-red-800',
  info:    'bg-ark-900/50   text-ark-400   border border-ark-800',
  purple:  'bg-purple-900/50 text-purple-400 border border-purple-800',
}

export function Badge({ variant = 'default', className, children, ...props }: BadgeProps) {
  return (
    <span
      className={cn(
        'inline-flex items-center px-2 py-0.5 rounded text-xs font-medium',
        variantClasses[variant],
        className
      )}
      {...props}
    >
      {children}
    </span>
  )
}
