import { useNavigate } from '@tanstack/react-router'
import { MonitorPlay, Users, Zap, Settings, FolderTree, GitBranch, PanelRight } from 'lucide-react'
import {
  CommandDialog, CommandInput, CommandList, CommandEmpty,
  CommandGroup, CommandItem, CommandShortcut,
} from '@ship/primitives'
import { useStudioShell } from './StudioShellContext'

export function CommandPalette() {
  const { commandPaletteOpen, setCommandPaletteOpen, togglePanel, toggleRightDrawer } = useStudioShell()
  const navigate = useNavigate()

  function runAndClose(fn: () => void) {
    fn()
    setCommandPaletteOpen(false)
  }

  return (
    <CommandDialog open={commandPaletteOpen} onOpenChange={setCommandPaletteOpen}>
      <CommandInput placeholder="Type a command..." />
      <CommandList>
        <CommandEmpty>No results found.</CommandEmpty>

        <CommandGroup heading="Views">
          <CommandItem onSelect={() => runAndClose(() => void navigate({ to: '/studio/session' }))}>
            <MonitorPlay className="size-4" />
            <span>Session</span>
          </CommandItem>
          <CommandItem onSelect={() => runAndClose(() => void navigate({ to: '/studio/agents' }))}>
            <Users className="size-4" />
            <span>Agents</span>
          </CommandItem>
          <CommandItem onSelect={() => runAndClose(() => void navigate({ to: '/studio/skills' }))}>
            <Zap className="size-4" />
            <span>Skills</span>
          </CommandItem>
          <CommandItem onSelect={() => runAndClose(() => void navigate({ to: '/studio/settings' }))}>
            <Settings className="size-4" />
            <span>Settings</span>
          </CommandItem>
        </CommandGroup>

        <CommandGroup heading="Actions">
          <CommandItem onSelect={() => runAndClose(() => togglePanel('files'))}>
            <FolderTree className="size-4" />
            <span>Toggle files panel</span>
            <CommandShortcut>Cmd+B</CommandShortcut>
          </CommandItem>
          <CommandItem onSelect={() => runAndClose(() => togglePanel('git'))}>
            <GitBranch className="size-4" />
            <span>Toggle git panel</span>
          </CommandItem>
          <CommandItem onSelect={() => runAndClose(() => toggleRightDrawer())}>
            <PanelRight className="size-4" />
            <span>Toggle right drawer</span>
            <CommandShortcut>Cmd+/</CommandShortcut>
          </CommandItem>
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  )
}
