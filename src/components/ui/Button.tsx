import { cn } from '../../utils/helpers'
import { type ButtonHTMLAttributes, forwardRef } from 'react'
import { Loader2 } from 'lucide-react'

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost'
  size?: 'sm' | 'md' | 'lg'
  loading?: boolean
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      className,
      variant = 'primary',
      size = 'md',
      loading,
      disabled,
      children,
      ...props
    },
    ref
  ) => {
    const base =
      'inline-flex items-center justify-center gap-2 rounded-lg font-medium transition-colors ' +
      'focus:outline-none focus:ring-2 focus:ring-ark-500 focus:ring-offset-2 focus:ring-offset-surface-900 ' +
      'disabled:opacity-50 disabled:cursor-not-allowed'

    const variants: Record<NonNullable<ButtonProps['variant']>, string> = {
      primary:   'bg-ark-600 text-white hover:bg-ark-500 active:bg-ark-700',
      secondary: 'bg-surface-700 text-slate-100 hover:bg-surface-600 border border-surface-600',
      danger:    'bg-red-600 text-white hover:bg-red-500 active:bg-red-700',
      ghost:     'text-slate-400 hover:text-slate-100 hover:bg-surface-800',
    }

    const sizes: Record<NonNullable<ButtonProps['size']>, string> = {
      sm: 'px-3 py-1.5 text-xs',
      md: 'px-4 py-2 text-sm',
      lg: 'px-5 py-2.5 text-base',
    }

    return (
      <button
        ref={ref}
        className={cn(base, variants[variant ?? 'primary'], sizes[size ?? 'md'], className)}
        disabled={disabled || loading}
        {...props}
      >
        {loading && <Loader2 size={14} className="animate-spin" />}
        {children}
      </button>
    )
  }
)
Button.displayName = 'Button'
