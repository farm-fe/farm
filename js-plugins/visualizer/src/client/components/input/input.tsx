import { inline } from '@stylex-extend/core'
import * as stylex from '@stylexjs/stylex'
import { clsx } from 'clsx'
import React, { useImperativeHandle, useRef, useState } from 'react'
import { useScale, withScale } from '../../composables'

interface Props {
  value?: string
  clearable?: boolean
}

type InputProps = Props & Omit<React.InputHTMLAttributes<HTMLInputElement>, keyof Props>

function simulateChangeEvent(el: HTMLInputElement, event: React.MouseEvent<HTMLDivElement>) {
  return {
    ...event,
    target: el,
    currentTarget: el
  }
}

const InputComponent = React.forwardRef<HTMLInputElement, InputProps>((props, ref) => {
  const {
    className: userClassName,
    style: userStyle,
    clearable,
    disabled,
    readOnly,
    type,
    value = '',
    onChange,
    ...rest
  } = props

  const inputRef = useRef<HTMLInputElement>(null)
  const [selfValue, setSelfValue] = useState<string>(value)
  const { SCALES } = useScale()

  if (value) {
    setSelfValue(value || '')
  }

  const { className, style } = stylex.props(inline({
    padding: 0,
    boxShadow: 'none',
    margin: '0.25em 0.625em',
    fontSize: SCALES.font(0.875),
    backgroundColor: 'transparent',
    border: 'none',
    color: '#000',
    outline: 'none',
    borderRadius: 0,
    width: '100%',
    minWidth: 0,
    WebkitAppearance: 'none',
    '::placeholder': {
      color: '#999'
    }
  }))

  useImperativeHandle(ref, () => inputRef.current!)

  const classes = clsx('input', className, userClassName)

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (disabled || readOnly) { return }
    setSelfValue(e.target.value)
    onChange?.(e)
  }

  const handleClear = (event: React.MouseEvent<HTMLDivElement>) => {
    setSelfValue('')
    if (!inputRef.current) { return }
    const changeEvent = simulateChangeEvent(inputRef.current, event)
    changeEvent.target.value = ''
    onChange?.(changeEvent)
    inputRef.current.focus()
  }

  return (
    <div
      stylex={{
        display: 'inline-block',
        boxSizing: 'border-box',
        fontSize: SCALES.font(0.875),
        width: SCALES.width(1, 'initial'),
        padding: `${SCALES.pt(0)} ${SCALES.pr(0)} ${SCALES.pb(0)} ${SCALES.pl(0)}`,
        margin: `${SCALES.mt(0)} ${SCALES.mr(0)} ${SCALES.mb(0)} ${SCALES.ml(0)}`,
        '--input-height': SCALES.height(2.25)
      }}
    >
      <div
        stylex={{
          display: 'inline-flex',
          alignItems: 'center',
          width: SCALES.width(1, 'initial'),
          height: 'var(--input-height)'
        }}
      >
        <div
          stylex={{
            display: 'inline-flex',
            verticalAlign: 'middle',
            alignItems: 'center',
            height: '100%',
            flex: 1,
            userSelect: 'none',
            borderRadius: '6px',
            border: '1px solid #666',
            transition: 'border 0.2s ease 0s, color 0.2s ease 0s'
          }}
        >
          <input
            type={type}
            ref={inputRef}
            value={selfValue}
            className={classes}
            disabled={disabled}
            readOnly={readOnly}
            onChange={handleChange}
            style={{ ...style, ...userStyle }}
            {...rest}
          />
          {clearable && (
            <div
              role="presentation"
              onClick={handleClear}
              stylex={{
                boxSizing: 'border-box',
                display: 'inline-flex',
                width: 'calc(var(--input-height) - 2px)',
                height: '100%',
                flexShrink: 0,
                alignItems: 'center',
                justifyContent: 'center',
                cursor: 'pointer',
                transition: 'color 150ms ease 0s',
                margin: 0,
                padding: 0,
                color: {
                  default: '#999',
                  ':hover': (disabled || readOnly) ? '#999' : '#000'
                },
                visibility: 'hidden',
                opacity: 0,
                ...(!!selfValue && { visibility: 'visible', opacity: 1 }),
                ...((disabled || readOnly) && { cursor: 'not-allowed' })
              }}
            >
              <svg
                stylex={{
                  color: 'currentColor',
                  width: 'calc(var(--input-height) - 2px)',
                  height: 'calc(var(--input-height) - 2px)',
                  transform: 'scale(0.4)'
                }}
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth="1.5"
                strokeLinecap="round"
                strokeLinejoin="round"
                fill="none"
                shapeRendering="geometricPrecision"
              >
                <path d="M18 6L6 18" />
                <path d="M6 6l12 12" />
              </svg>
            </div>
          )}
        </div>
      </div>
    </div>
  )
})

export const Input = withScale(InputComponent)
