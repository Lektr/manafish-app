import { useStore } from '@tanstack/react-store';
import { Gamepad2Icon } from 'lucide-react';
import { useEffect, useState } from 'react';

import { GamepadBindInput } from '@/components/composites/GamepadBindInput';
import { Button } from '@/components/ui/Button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/Dialog';
import { toast } from '@/components/ui/Toaster';

import {
  type GamepadBindings,
  configStore,
  updateGamepadBindings,
} from '@/stores/configStore';

const DEFAULT_GAMEPAD_BINDINGS: GamepadBindings = {
  moveHorizontal: 'LeftStick',
  moveUp: '7',
  moveDown: '6',
  pitchYaw: 'RightStick',
  rollLeft: '4',
  rollRight: '5',
};

function GamepadBindingsDialog() {
  const config = useStore(configStore, (state) => state);
  const [localBindings, setLocalBindings] = useState<GamepadBindings | null>(
    null,
  );
  const [isControllerConnected, setIsControllerConnected] = useState(() => {
    const gamepads = navigator.getGamepads();
    return Array.from(gamepads).some((gamepad) => gamepad !== null);
  });

  useEffect(() => {
    const handleGamepadConnected = () => setIsControllerConnected(true);
    const handleGamepadDisconnected = () => setIsControllerConnected(false);

    window.addEventListener('gamepadconnected', handleGamepadConnected);
    window.addEventListener('gamepaddisconnected', handleGamepadDisconnected);

    return () => {
      window.removeEventListener('gamepadconnected', handleGamepadConnected);
      window.removeEventListener(
        'gamepaddisconnected',
        handleGamepadDisconnected,
      );
    };
  }, []);

  if (!config) return null;

  const currentBindings = localBindings ?? config.gamepad;

  async function handleBindingChange(
    key: keyof GamepadBindings,
    value: string,
  ) {
    const newBindings = {
      ...currentBindings,
      [key]: value,
    };

    setLocalBindings(newBindings);

    try {
      await updateGamepadBindings(newBindings);
      toast.success('Gamepad binding updated');
    } catch (error) {
      console.error('Failed to update gamepad binding:', error);
      toast.error('Failed to update gamepad binding');
    }
  }

  return (
    <Dialog>
      <DialogTrigger asChild disabled={!isControllerConnected}>
        <Button variant='outline' size='icon'>
          <Gamepad2Icon className='h-[1.2rem] w-[1.2rem]' />
          <span className='sr-only'>Show gamepad layout</span>
        </Button>
      </DialogTrigger>
      <DialogContent className='max-h-[90vh] overflow-y-auto focus-visible:outline-none'>
        <DialogHeader>
          <DialogTitle>Gamepad Layout</DialogTitle>
        </DialogHeader>
        <DialogDescription>
          Configure your gamepad bindings for controlling the drone.
        </DialogDescription>
        <div className='mx-auto grid max-w-2xl grid-cols-1 sm:grid-cols-2'>
          <div className='mx-4'>
            <h3 className='text-md mt-4 mb-2 font-semibold'>Movement</h3>
            <div className='space-y-2'>
              <GamepadBindInput
                label='Move Horizontal'
                bind={currentBindings.moveHorizontal}
                defaultBind={DEFAULT_GAMEPAD_BINDINGS.moveHorizontal}
                onBindChange={(newBind) =>
                  handleBindingChange('moveHorizontal', newBind)
                }
                isJoystick
              />
              <GamepadBindInput
                label='Move Up'
                bind={currentBindings.moveUp}
                defaultBind={DEFAULT_GAMEPAD_BINDINGS.moveUp}
                onBindChange={(newBind) =>
                  handleBindingChange('moveUp', newBind)
                }
              />
              <GamepadBindInput
                label='Move Down'
                bind={currentBindings.moveDown}
                defaultBind={DEFAULT_GAMEPAD_BINDINGS.moveDown}
                onBindChange={(newBind) =>
                  handleBindingChange('moveDown', newBind)
                }
              />
            </div>
          </div>
          <div className='mx-4'>
            <h3 className='text-md mt-4 mb-2 font-semibold'>Pitch & Yaw</h3>
            <div className='space-y-2'>
              <GamepadBindInput
                label='Pitch/Yaw'
                bind={currentBindings.pitchYaw}
                defaultBind={DEFAULT_GAMEPAD_BINDINGS.pitchYaw}
                onBindChange={(newBind) =>
                  handleBindingChange('pitchYaw', newBind)
                }
                isJoystick
              />
            </div>
            <h3 className='text-md mt-4 mb-2 font-semibold'>Roll</h3>
            <div className='space-y-2'>
              <GamepadBindInput
                label='Roll Left'
                bind={currentBindings.rollLeft}
                defaultBind={DEFAULT_GAMEPAD_BINDINGS.rollLeft}
                onBindChange={(newBind) =>
                  handleBindingChange('rollLeft', newBind)
                }
              />
              <GamepadBindInput
                label='Roll Right'
                bind={currentBindings.rollRight}
                defaultBind={DEFAULT_GAMEPAD_BINDINGS.rollRight}
                onBindChange={(newBind) =>
                  handleBindingChange('rollRight', newBind)
                }
              />
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

export { GamepadBindingsDialog };
