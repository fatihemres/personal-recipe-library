// Local shadcn/ui-style Button: Radix Slot + CVA, customized to the culinary theme.
import { Slot } from '@radix-ui/react-slot';
import { cva, type VariantProps } from 'class-variance-authority';
import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';
import type { ComponentProps } from 'react';
const buttonVariants = cva('inline-flex items-center justify-center gap-2 rounded-xl text-sm font-semibold transition-colors disabled:pointer-events-none disabled:opacity-50', {
  variants: { variant: { default: 'bg-primary text-primary-foreground hover:opacity-90', outline: 'border border-border bg-card hover:bg-muted', ghost: 'hover:bg-muted' }, size: { default: 'px-4 py-3', icon: 'size-10' } },
  defaultVariants: { variant: 'default', size: 'default' },
});
export function Button({ className, variant, size, asChild = false, ...props }: ComponentProps<'button'> & VariantProps<typeof buttonVariants> & { asChild?: boolean }) {
  const Comp = asChild ? Slot : 'button';
  return <Comp data-slot="button" className={twMerge(clsx(buttonVariants({ variant, size }), className))} {...props} />;
}
