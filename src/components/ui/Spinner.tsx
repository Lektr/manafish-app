import { Loader2Icon, type LucideProps } from 'lucide-react';

import { type VariantProps, cva } from '@/lib/utils';

const spinnerVariants = cva({
  base: 'text-muted-foreground animate-spin',
  variants: {
    size: {
      default: 'h-6 w-6',
      sm: 'h-4 w-4',
      lg: 'h-8 w-8',
      xl: 'h-10 w-10',
      none: '',
    },
  },
  defaultVariants: {
    size: 'default',
  },
});

function Spinner({
  className,
  size,
  ...props
}: LucideProps & VariantProps<typeof spinnerVariants>) {
  return (
    <Loader2Icon className={spinnerVariants({ size, className })} {...props} />
  );
}

export { Spinner };
