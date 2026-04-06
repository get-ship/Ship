import { useNavigate } from '@tanstack/react-router'
import { Layers, Users, Zap, Briefcase, Settings, FolderTree, MessageSquare } from 'lucide-react'
import {
  CommandDialog, CommandInput, CommandList, CommandEmpty,
  CommandGroup, CommandItem, CommandShortcut,
} from '@ship/primitives'
import { useStudioShell } from './StudioShellContext'

export function CommandPalette() {
  const { commandPaletteOpen, setCommandPaletteOpen, toggleFileSidebar, toggleAgentPanel } = useStudioShell()
  const navigate = useNavigate()

  function go(to: string) {
    void navigate({ to })
    setCommandPaletteOpen(false)
  }

  function run(fn: () => void) {
    fn()
    setCommandPaletteOpen(false)
  }

  return (
    <CommandDialog open={commandPaletteOpen} onOpenChange={setCommandPaletteOpen}>
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
          <CommandItem onSelect={() => go('/studio/jobs')}>
            <Briefcase className="size-4" />
            <span>Jobs</span>
          </CommandItem>
          <CommandItem onSelect={() => go('/studio/settings')}>
            <Settings className="size-4" />
            <span>Settings</span>
          </CommandItem>
        </CommandGroup>

        <CommandGroup heading="Actions">
          <CommandItem onSelect={() => run(toggleFileSidebar)}>
            <FolderTree className="size-4" />
            <span>Toggle file sidebar</span>
            <CommandShortcut>Cmd+B</CommandShortcut>
          </CommandItem>
          <CommandItem onSelect={() => run(toggleAgentPanel)}>
            <MessageSquare className="size-4" />
            <span>Toggle agent chat</span>
            <CommandShortcut>Cmd+/</CommandShortcut>
          </CommandItem>
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  )
}
