import { useState } from 'react'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/Dialog'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/Select'
import { WasteType } from '@/api/types'
import { useRecycleWaste } from '@/hooks/useRecycleWaste'

const WASTE_TYPES = [
  { value: WasteType.Paper, label: 'Paper' },
  { value: WasteType.PetPlastic, label: 'PET Plastic' },
  { value: WasteType.Plastic, label: 'Plastic' },
  { value: WasteType.Metal, label: 'Metal' },
  { value: WasteType.Glass, label: 'Glass' },
]

interface Props {
  open: boolean
  address: string
  onClose: () => void
  onSuccess?: (wasteId: bigint) => void
}

export function RegisterWasteModal({ open, address, onClose, onSuccess }: Props) {
  const [wasteType, setWasteType] = useState<WasteType>(WasteType.Paper)
  const [weight, setWeight] = useState('')
  const [latitude, setLatitude] = useState('')
  const [longitude, setLongitude] = useState('')

  const { mutate: recycleWaste, isPending } = useRecycleWaste()

  function handleClose() {
    if (isPending) return
    setWeight('')
    setLatitude('')
    setLongitude('')
    setWasteType(WasteType.Paper)
    onClose()
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    const w = parseFloat(weight)
    if (!w || w <= 0) return

    // Convert kg → grams for the contract
    const weightGrams = BigInt(Math.round(w * 1000))
    // Convert decimal degrees → microdegrees (contract expects i128 microdegrees)
    const lat = BigInt(Math.round(parseFloat(latitude || '0') * 1_000_000))
    const lng = BigInt(Math.round(parseFloat(longitude || '0') * 1_000_000))

    recycleWaste(
      { recycler: address, wasteType, weightGrams, latitude: lat, longitude: lng },
      {
        onSuccess: (wasteId) => {
          onSuccess?.(wasteId)
          handleClose()
        },
      }
    )
  }

  return (
    <Dialog open={open} onOpenChange={(o) => !o && handleClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Register Waste</DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-1">
            <label className="text-sm font-medium">Waste type</label>
            <Select
              value={String(wasteType)}
              onValueChange={(v) => setWasteType(Number(v) as WasteType)}
            >
              <SelectTrigger className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {WASTE_TYPES.map((t) => (
                  <SelectItem key={t.value} value={String(t.value)}>
                    {t.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-1">
            <label className="text-sm font-medium">Weight (kg)</label>
            <Input
              type="number"
              min="0.001"
              step="0.001"
              placeholder="e.g. 2.5"
              value={weight}
              onChange={(e) => setWeight(e.target.value)}
              required
            />
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1">
              <label className="text-sm font-medium">Latitude</label>
              <Input
                type="number"
                min="-90"
                max="90"
                step="0.000001"
                placeholder="e.g. 40.714"
                value={latitude}
                onChange={(e) => setLatitude(e.target.value)}
              />
            </div>
            <div className="space-y-1">
              <label className="text-sm font-medium">Longitude</label>
              <Input
                type="number"
                min="-180"
                max="180"
                step="0.000001"
                placeholder="e.g. -74.006"
                value={longitude}
                onChange={(e) => setLongitude(e.target.value)}
              />
            </div>
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={handleClose} disabled={isPending}>
              Cancel
            </Button>
            <Button type="submit" disabled={isPending || !weight}>
              {isPending ? 'Submitting…' : 'Submit'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
