import type { QueryClient } from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';
import { Outlet, createRootRouteWithContext } from '@tanstack/react-router';
import { TanStackRouterDevtools } from '@tanstack/react-router-devtools';

import { ThemeProvider } from '@/components/providers/ThemeProvider';
import { Toaster } from '@/components/ui/Toaster';

import { loadConfig } from '@/stores/configStore';

export const Route = createRootRouteWithContext<{
  queryClient: QueryClient;
}>()({
  component: RootComponent,
  loader: loadConfig,
});

function RootComponent() {
  return (
    <>
      <ThemeProvider>
        <Outlet />
        <Toaster />
      </ThemeProvider>
      <ReactQueryDevtools buttonPosition='bottom-right' />
      <TanStackRouterDevtools position='bottom-right' />
    </>
  );
}
