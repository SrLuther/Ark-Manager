import { cn } from '../../utils/helpers'
import { type HTMLAttributes } from 'react'

export interface CardProps extends HTMLAttributes<HTMLDivElement> {
  noPad?: boolean
}

export function Card({ className, noPad, children, ...props }: CardProps) {
  return (
    <div
      className={cn(
        'bg-surface-800 border border-surface-700 rounded-xl',
        !noPad && 'p-4',
        className
      )}
      {...props}
    >
      {children}
    </div>
  )
}

export function CardHeader({
  className,
  children,
  ...props
}: HTMLAttributes<HTMLDivElement>) {
  return (
    <div className={cn('flex items-center justify-between mb-4', className)} {...props}>
      {children}
    </div>
  )
}

export function CardTitle({
  className,
  children,
  ...props
}: HTMLAttributes<HTMLHeadingElement>) {
  return (
    <h3 className={cn('text-sm font-semibold text-slate-100', className)} {...props}>
      {children}
    </h3>
  )
}
