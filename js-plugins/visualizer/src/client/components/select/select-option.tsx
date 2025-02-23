import React, { useMemo } from 'react'
import { useScale, withScale } from '../../composables'
import { useSelect } from './context'
import { Ellipsis } from './ellipsis'

interface Props {
  label?: string
  value?: string
  disabled?: boolean
  preventAllEvents?: boolean
}

export type SelectOptionProps = Omit<React.HTMLAttributes<unknown>, keyof Props> & Props

function SelectOptionComponent(props: React.PropsWithChildren<SelectOptionProps>) {
  const { children, value: initialValue, disabled = false, preventAllEvents, ...rest } = props
  const { SCALES } = useScale()
  const { disableAll, value, updateValue } = useSelect()
  const isDisabled = useMemo(() => disabled || disableAll, [disabled, disableAll])

  const selected = useMemo(() => {
    if (!value) { return false }
    if (typeof value === 'string') { return initialValue === value }
    return value.includes(initialValue + '')
  }, [value, initialValue])

  const color = useMemo(() => {
    if (isDisabled) { return '#888' }
    return selected ? '#000' : '#666'
  }, [selected, isDisabled])

  const bgColor = useMemo(() => {
    if (isDisabled) { return '#fafafa' }
    return selected ? '#eaeaea' : '#fff'
  }, [selected, isDisabled])

  const hoverBgColor = useMemo(() => {
    if (isDisabled || selected) { return bgColor }
    return '#fafafa'
  }, [isDisabled, bgColor, selected])

  const handleClick = (event: React.MouseEvent<HTMLDivElement>) => {
    if (preventAllEvents) { return }
    event.stopPropagation()
    event.nativeEvent.stopImmediatePropagation()
    event.preventDefault()
    if (isDisabled) { return }
    updateValue?.(initialValue!)
  }

  return (
    <div
      role="presentation"
      stylex={{
        boxSizing: 'border-box',
        maxWidth: '100%',
        display: 'flex',
        justifyContent: 'flex-start',
        alignItems: 'center',
        fontWeight: 'normal',
        userSelect: 'none',
        border: 0,
        transition: 'background 0.2s ease 0s, border-color 0.2s ease 0s',
        backgroundColor: {
          default: bgColor,
          ':hover': hoverBgColor
        },
        color: {
          default: color,
          ':hover': '#333'
        },
        cursor: 'pointer',
        ...(isDisabled && { cursor: 'not-allowed' }),
        '--select-font-size': SCALES.font(0.75),
        fontSize: 'var(--select-font-size)',
        width: SCALES.width(1, '100%'),
        height: SCALES.height(2.25),
        padding: `${SCALES.pt(0)} ${SCALES.pr(0.667)} ${SCALES.pb(0)} ${SCALES.pl(0.667)}`,
        margin: `${SCALES.mt(0)} ${SCALES.mr(0)} ${SCALES.mb(0)} ${SCALES.ml(0)}`
      }}
      onClick={handleClick}
      {...rest}
    >
      <Ellipsis height={SCALES.height(2.25)}>{children}</Ellipsis>
    </div>
  )
}

export const SelectOption = withScale(SelectOptionComponent)
