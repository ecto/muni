// Stub for place-autocomplete - not using Google Places API
import { Input } from "@/components/ui/input"
import type { ComponentProps } from "react"

export interface PlaceAutocompleteProps extends ComponentProps<typeof Input> {
  onPlaceSelect?: (place: { lat: number; lng: number; address: string }) => void
}

export function PlaceAutocomplete({ onPlaceSelect, ...props }: PlaceAutocompleteProps) {
  return <Input placeholder="Search location..." {...props} />
}
