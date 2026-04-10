export function DaemonRequired() {
  return (
    <div role="status">
      <h2>Ship Daemon Required</h2>
      <p>
        To connect, start the runtime with <code>ship start</code> or install
        Ship if you have not already.
      </p>
    </div>
  )
}
