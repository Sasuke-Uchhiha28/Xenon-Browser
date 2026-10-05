import { Card } from './Card'

interface MetricCardProps {
  /** Small mono label, for example "Trackers blocked today". */
  label: string
  /** The measured value; when null the card renders nothing at all. */
  value: string | null
}

/**
 * A number-with-label card (Design.md 9). UI-08 / Design.md 4.3: when
 * there is no real value yet, the element stays hidden — never a
 * placeholder number.
 *
 * @returns The card, or `null` when `value` is null.
 */
export function MetricCard({ label, value }: MetricCardProps) {
  if (value === null) return null
  return (
    <Card className="px-4 py-3">
      <div className="font-mono text-xs text-outline">{label}</div>
      <div className="font-headline text-2xl text-on-surface">{value}</div>
    </Card>
  )
}
