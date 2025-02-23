import React, { useCallback, useEffect, useImperativeHandle, useRef, useState } from 'react'
import { createPortal } from 'react-dom'
import { useClickAnyWhere, useDOMObserver, usePortal, useResize } from '../../composables'
import { CSSTransition } from '../css-transition'
import { useSelect } from './context'
import { getRefRect } from './layouts'

interface Props {
  visible: boolean
}

export type SelectDropdownProps = Omit<React.HTMLAttributes<unknown>, keyof Props> & Props

interface ReactiveDomReact {
  top: number
  left: number
  right: number
  width: number
}

const defaultRect: ReactiveDomReact = {
  top: -1000,
  left: -1000,
  right: -1000,
  width: 0
}

const SelectDropdown = React.forwardRef<
  HTMLDivElement,
  React.PropsWithChildren<SelectDropdownProps>
>(({ visible, children }, dropdownRef) => {
  const [rect, setRect] = useState<ReactiveDomReact>(defaultRect)
  const internalDropdownRef = useRef<HTMLDivElement | null>(null)
  const { ref, updateVisible } = useSelect()

  const el = usePortal('dropdown')

  useImperativeHandle<HTMLDivElement | null, HTMLDivElement | null>(
    dropdownRef,
    () => internalDropdownRef.current
  )

  const updateRect = useCallback(() => {
    const {
      top,
      left,
      right,
      width: nativeWidth
    } = getRefRect(ref)
    setRect({ top, left, right, width: nativeWidth })
  }, [ref])

  useResize(ref, updateRect)

  useClickAnyWhere(() => {
    setRect(defaultRect)
    updateVisible?.(false)
  })
  useDOMObserver(ref, () => {
    updateRect()
  })
  useEffect(() => {
    if (!ref || !ref.current) { return }
    const internalDropdownEl = ref.current
    internalDropdownEl.addEventListener('mouseenter', updateRect)
    /* istanbul ignore next */
    return () => {
      internalDropdownEl.removeEventListener('mouseenter', updateRect)
    }
  }, [ref, updateRect])

  const clickHandler = (event: React.MouseEvent<HTMLDivElement>) => {
    event.stopPropagation()
    event.nativeEvent.stopImmediatePropagation()
    event.preventDefault()
  }
  const mouseDownHandler = (event: React.MouseEvent<HTMLDivElement>) => {
    event.preventDefault()
  }

  if (!ref || !el) { return null }

  return createPortal(
    <CSSTransition visible={visible}>
      <div
        role="presentation"
        onClick={clickHandler}
        onMouseDown={mouseDownHandler}
        stylex={{
          position: 'absolute',
          zIndex: 1100,
          top: `${rect.top + 2}px`,
          left: `${rect.left}px`,
          width: `${rect.width}px`
        }}
      >
        <div
          ref={internalDropdownRef}
          stylex={{
            borderRadius: '6px',
            boxShadow: '0 30px 60px rgba(0, 0, 0, 0.12)',
            backgroundColor: '#fff',
            maxHeight: '17em',
            overflowY: 'auto',
            overflowAnchor: 'none',
            padding: '0.38em 0',
            scrollBehavior: 'smooth'
          }}
        >
          {children}
        </div>
      </div>
    </CSSTransition>,
    el
  )
})

export { SelectDropdown }
