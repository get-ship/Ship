import { useNavigate } from '@tanstack/react-router'
import { Layers, Users, Zap, Briefcase, Settings, FolderTree } from 'lucide-react'
import {
  CommandDialog, CommandInput, CommandList, CommandEmpty,
  CommandGroup, CommandItem, CommandShortcut,
} from '@ship/primitives'
import { usePlatform } from './PlatformProvider'

export function CommandPalette() {
  const { commandPaletteOpen, toggleCommandPalette, toggleSidebar } = usePlatform()
  const navigate = useNavigate()

  function close() {
    if (commandPaletteOpen) toggleCommandPalette()
  }

  function go(to: string) {
    void navigate({ to })
    close()
  }

  function run(fn: () => void) {
    fn()
    close()
  }

  return (
    <CommandDialog open={commandPaletteOpen} onOpenChange={() => close()}>
      <CommandInput placeholder="Type a command..." />
      <CommandList>
        <CommandEmpty>No results found.</CommandEmpty>

        <CommandGroup heading="Navigate">
          <CommandItem onSelect={() => go('/studio/session')}>
            <Layers className="size-4" />
            <span>Artifacts</span>
          </CommandItem>
          <CommandItem onSelect={() => go('/studio/agents')}>
            <Users className="size-4" />
            <span>Agents</span>
          </CommandItem>
          <CommandItem onSelect={() => go('/studio/skills')}>
            <Zap className="size-4" />
            <span>Skills</span>
          </CommandItem>
          <CommandItem onSelect={() => go('/studio/workflows')}>
            <Briefcase className="size-4" />
            <span>Workflows</span>
          </CommandItem>
          <CommandItem onSelect={() => go('/studio/settings')}>
            <Settings className="size-4" />
            <span>Settings</span>
          </CommandItem>
        </CommandGroup>

        <CommandGroup heading="Actions">
          <CommandItem onSelect={() => run(toggleSidebar)}>
            <FolderTree className="size-4" />
            <span>Toggle file sidebar</span>
            <CommandShortcut>Cmd+B</CommandShortcut>
          </CommandItem>
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  )
}
