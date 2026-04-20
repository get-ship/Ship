import { PermissionsEditor } from '@ship/set'
import type { Permissions } from '@ship/set'

interface Props {
  permissions: Permissions
  onChange: (p: Permissions) => void
}

export function PermissionsForm({ permissions, onChange }: Props) {
  return <PermissionsEditor permissions={permissions} onChange={onChange} />
}
