import { useStore } from '@tanstack/react-store';

import { connectionStatusStore } from '@/stores/connectionStatus';

function ConnectionStatusIndicator() {
  const { isConnected, delay } = useStore(connectionStatusStore);

  return (
    <div className='flex items-center space-x-2'>
      <div
        className={`h-3 w-3 rounded-full ${
          isConnected ? 'bg-green-500' : 'bg-destructive'
        }`}
      />
      {isConnected && (
        <span className='text-xs drop-shadow-[0_1.2px_1.2px_rgba(0,0,0,0.8)]'>
          {delay} ms
        </span>
      )}
    </div>
  );
}

export { ConnectionStatusIndicator };
