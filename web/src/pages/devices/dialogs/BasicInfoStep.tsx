/**
 * BasicInfoStep - Step 1 of the device type creation wizard
 * Handles basic device type information: name, type ID, description, categories
 */

import { useState, useEffect } from "react"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { Badge } from "@/components/ui/badge"
import { AlertCircle } from "lucide-react"
import { cn } from "@/lib/utils"
import type { DeviceType } from "@/types"

interface FormErrors {
  device_type?: string
  name?: string
  metrics?: Record<number, string>
  commands?: Record<number, string>
  [key: string]: string | Record<number, string> | undefined
}

interface BasicInfoStepProps {
  data: Partial<DeviceType>
  onChange: <K extends keyof DeviceType>(field: K, value: DeviceType[K]) => void
  errors: FormErrors
}

export function BasicInfoStep({ data, onChange, errors }: BasicInfoStepProps) {
  const [categoryInput, setCategoryInput] = useState("")
  const [nameInput, setNameInput] = useState(data.name || "")

  // Sync nameInput with data.name when it changes (e.g., when switching to edit mode)
  useEffect(() => {
    setNameInput(data.name || "")
  }, [data.name])

  const addCategory = () => {
    const cat = categoryInput.trim()
    if (cat && !data.categories?.includes(cat)) {
      onChange('categories', [...(data.categories || []), cat])
      setCategoryInput("")
    }
  }

  const removeCategory = (cat: string) => {
    onChange('categories', (data.categories || []).filter(c => c !== cat))
  }

  // Generate type ID from name
  const generateTypeId = (name: string): string => {
    return name.toLowerCase()
      .replace(/\s+/g, "_")
      .replace(/[^a-z0-9_]/g, "")
      .replace(/_+/g, "_")
      .replace(/^_|_$/g, "")
  }

  // Only auto-generate on blur (when user finishes typing)
  const handleNameBlur = () => {
    if (!data.device_type && nameInput.trim()) {
      onChange('device_type', generateTypeId(nameInput))
    }
  }

  const handleNameChange = (value: string) => {
    setNameInput(value)
    onChange('name', value)
  }

  return (
    <div className="space-y-6 max-w-2xl mx-auto py-4">
      <div className="text-center mb-6">
        <h3 className="text-lg font-semibold">Basic Information</h3>
        <p className="text-sm text-muted-foreground">Enter the basic information for your device type</p>
      </div>

      {/* Device Type (name) */}
      <div className="space-y-2">
        <Label htmlFor="device-type-name" className="text-sm font-medium">
          Device Type <span className="text-error">*</span>
        </Label>
        <Input
          id="device-type-name"
          value={nameInput}
          onChange={(e) => handleNameChange(e.target.value)}
          onBlur={handleNameBlur}
          placeholder="e.g., Smart Temperature Sensor"
          className={cn(errors.name && "border-error")}
        />
        {errors.name && (
          <p className="text-xs text-error flex items-center gap-1">
            <AlertCircle className="h-4 w-4" />
            {errors.name}
          </p>
        )}
      </div>

      {/* Type ID (auto-generated from Device Type) */}
      <div className="space-y-2">
        <Label htmlFor="type-id" className="text-sm font-medium">
          Type ID <span className="text-error">*</span>
        </Label>
        <Input
          id="type-id"
          value={data.device_type || ""}
          onChange={(e) => onChange('device_type', e.target.value)}
          placeholder="smart_temp_sensor"
          className={cn("font-mono", errors.device_type && "border-error")}
        />
        <p className="text-xs text-muted-foreground">
          Auto-generated from Device Type after you finish typing
        </p>
        {errors.device_type && (
          <p className="text-xs text-error flex items-center gap-1">
            <AlertCircle className="h-4 w-4" />
            {errors.device_type}
          </p>
        )}
      </div>

      {/* Description */}
      <div className="space-y-2">
        <Label htmlFor="description" className="text-sm font-medium">Description</Label>
        <Textarea
          id="description"
          value={data.description || ""}
          onChange={(e) => onChange('description', e.target.value)}
          placeholder="Describe what this device type does..."
          rows={3}
          className="resize-none"
        />
      </div>

      {/* Categories */}
      <div className="space-y-2">
        <Label className="text-sm font-medium">Categories</Label>
        <div className="flex gap-2 flex-wrap">
          {data.categories?.map((cat, i) => (
            <Badge key={i} variant="secondary" className="pl-2 pr-1 h-7">
              {cat}
              <button
                onClick={() => removeCategory(cat)}
                className="ml-1 hover:text-error"
              >
                ×
              </button>
            </Badge>
          ))}
          <div className="flex gap-1">
            <Input
              placeholder="+ Add category"
              value={categoryInput}
              onChange={(e) => setCategoryInput(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && (e.preventDefault(), addCategory())}
              className="h-7 w-32 text-xs"
            />
          </div>
        </div>
      </div>
    </div>
  )
}
