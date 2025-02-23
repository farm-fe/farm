import React, { useCallback, useMemo, useState } from 'react'
import { useScale, withScale } from '../../composables'
import { CheckboxProvider } from './context'

interface Props {
  value: string[]
  disabled?: boolean
  onChange?: (values: string[]) => void
}

export type CheckboxGroupProps = Props & Omit<React.HTMLAttributes<unknown>, keyof Props>

const defaultValue: string[] = []

function CheckboxGroupComponent(props: React.PropsWithChildren<CheckboxGroupProps>) {
  const { children, value = defaultValue, disabled = false, onChange, ...rest } = props
  const { SCALES } = useScale()
  const [selfValue, setSelfValue] = useState<string[]>(value)

  const updateState = useCallback((val: string, checked: boolean) => {
    const removed = selfValue.filter((v) => v !== val)
    const next = checked ? [...removed, val] : removed
    setSelfValue(next)
    onChange?.(next)
  }, [selfValue, onChange])

  const contextValue = useMemo(() => {
    return {
      disabledAll: disabled,
      values: value,
      inGroup: true,
      updateState
    }
  }, [disabled, value, updateState])

  return (
    <CheckboxProvider value={contextValue}>
      <div
        stylex={{
          width: SCALES.width(1, 'auto'),
          height: SCALES.height(1, 'auto'),
          padding: `${SCALES.pt(0)} ${SCALES.pr(0)} ${SCALES.pb(0)} ${SCALES.pl(0)}`,
          margin: `${SCALES.mt(0)} ${SCALES.mr(0)} ${SCALES.mb(0)} ${SCALES.ml(0)}`,
          ':not(#_) label': {
            marginRight: `calc(${SCALES.font(1)} * 2)`,
            '--checkbox-size': SCALES.font(1)
          },
          ':not(#_) label:last-of-type': {
            marginRight: 0
          }
        }}
        {...rest}
      >
        {children}
      </div>
    </CheckboxProvider>
  )
}

export const CheckboxGroup = withScale(CheckboxGroupComponent)
