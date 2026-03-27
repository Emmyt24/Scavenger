import { useState } from 'react'
import { Copy, Check, ExternalLink } from 'lucide-react'
import { cn } from '@/lib/utils'
import { config } from '@/config'

const EXPLORER_ROOTS: Record<string, string> = {
  TESTNET: 'https://stellar.expert/explorer/testnet/account',
  MAINNET: 'https://stellar.expert/explorer/public/account',
  FUTURENET: 'https://stellar.expert/explorer/futurenet/account',
  STANDALONE: '',
}

interface AddressDisplayProps {
  address: string
  /** Number of chars to show at each end (default 4) */
  chars?: number
  /** Show link to Stellar Expert explorer */
  showExplorer?: boolean
  className?: string
}

export function AddressDisplay({ address, chars = 4, showExplorer = false, className }: AddressDisplayProps) {
  const [copied, setCopied] = useState(false)

  const truncated = `${address.slice(0, chars)}…${address.slice(-chars)}`
  const explorerUrl = `${EXPLORER_ROOTS[config.network] ?? EXPLORER_ROOTS.TESTNET}/${address}`

  const copy = () => {
    navigator.clipboard.writeText(address).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    })
  }

  return (
    <span className={cn('inline-flex items-center gap-1', className)}>
      <span
        className="font-mono text-xs"
        title={address}
        aria-label={address}
      >
        {truncated}
      </span>

      <button
        type="button"
        onClick={copy}
        title="Copy address"
        aria-label="Copy address"
        className="text-muted-foreground hover:text-foreground transition-colors"
      >
        {copied
          ? <Check className="h-3 w-3 text-green-500" />
          : <Copy className="h-3 w-3" />}
      </button>

      {showExplorer && EXPLORER_ROOTS[config.network] && (
        <a
          href={explorerUrl}
          target="_blank"
          rel="noreferrer"
          title="View on Stellar Expert"
          aria-label="View on Stellar Expert"
          className="text-muted-foreground hover:text-foreground transition-colors"
        >
          <ExternalLink className="h-3 w-3" />
        </a>
      )}
    </span>
  )
}
