/**
 * Install Component Dialog
 *
 * Dialog for manually importing a community component via ZIP package upload.
 * The backend handles ZIP extraction — frontend just sends the file.
 */

import { useState, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { PackagePlus, FileArchive, Upload } from 'lucide-react'
import { UnifiedFormDialog } from '@/components/dialog/UnifiedFormDialog'
import { useStore } from '@/store'
import { notifySuccess, notifyFromError } from '@/lib/notify'

interface InstallComponentDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function InstallComponentDialog({ open, onOpenChange }: InstallComponentDialogProps) {
  const { t } = useTranslation('dashboardComponents')
  const inputRef = useRef<HTMLInputElement>(null)
  const { installManualZip } = useStore()

  const [zipFile, setZipFile] = useState<File | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)

  const handleClick = () => inputRef.current?.click()

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (file) {
      setZipFile(file)
      setSubmitError(null)
    }
  }

  const handleInstall = async () => {
    if (!zipFile) return

    setIsSubmitting(true)
    setSubmitError(null)

    try {
      await installManualZip(zipFile)
      notifySuccess(t('installSuccess'))
      resetState()
      onOpenChange(false)
    } catch (error) {
      setSubmitError(error instanceof Error ? error.message : t('installError'))
      notifyFromError(error, t('installError'))
    } finally {
      setIsSubmitting(false)
    }
  }

  const resetState = () => {
    setZipFile(null)
    setSubmitError(null)
  }

  const handleOpenChange = (newOpen: boolean) => {
    if (!newOpen) resetState()
    onOpenChange(newOpen)
  }

  const canInstall = zipFile && !isSubmitting

  return (
    <UnifiedFormDialog
      open={open}
      onOpenChange={handleOpenChange}
      title={t('componentLibrary.importTitle')}
      icon={<PackagePlus className="w-full h-full" />}
      width="lg"
      className="z-[110]"
      isSubmitting={isSubmitting}
      submitError={submitError}
      onSubmit={handleInstall}
      submitLabel={t('installConfirm')}
      submitDisabled={!canInstall}
    >
      <div className="space-y-6">
        {/* ZIP upload zone */}
        <div
          onClick={handleClick}
          className={`
            border-2 border-dashed rounded-lg p-8 text-center cursor-pointer
            transition-colors
            ${zipFile
              ? 'border-success bg-success-light'
              : 'border-border hover:border-primary-foreground hover:bg-muted-30'
            }
          `}
        >
          <input
            ref={inputRef}
            type="file"
            accept=".zip"
            onChange={handleInputChange}
            className="hidden"
          />
          {zipFile ? (
            <div className="space-y-1">
              <FileArchive className="w-10 h-10 text-success mx-auto" />
              <p className="text-sm font-medium text-foreground">{zipFile.name}</p>
              <p className="text-xs text-muted-foreground">
                {(zipFile.size / 1024).toFixed(1)} KB
              </p>
            </div>
          ) : (
            <div className="space-y-2">
              <Upload className="w-10 h-10 text-muted-foreground mx-auto" />
              <p className="text-sm text-muted-foreground">{t('selectFile')} (.zip)</p>
            </div>
          )}
        </div>

        {/* Instructions */}
        <div className="p-4 bg-muted-30 rounded-lg border border-border">
          <ul className="text-xs text-muted-foreground space-y-1 list-disc list-inside">
            <li>{t('componentLibrary.zipPackageDesc')}</li>
          </ul>
        </div>
      </div>
    </UnifiedFormDialog>
  )
}
