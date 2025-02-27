import { injectGlobalStyle } from '@stylex-extend/core'
import * as stylex from '@stylexjs/stylex'
import { clsx } from 'clsx'
import React, { useRef } from 'react'
import type { MouseEvent } from 'react'
import { CSSTransition } from '../css-transition'

interface Props {
  onClick?: (event: MouseEvent<HTMLElement>) => void
  visible?: boolean
  width?: string
  onContentClick?: (event: MouseEvent<HTMLElement>) => void
  backdropClassName?: string
  positionClassName?: string
  layerClassName?: string
}

type NativeAttrs = Omit<React.HTMLAttributes<unknown>, keyof Props>
export type BackdropProps = Props & NativeAttrs

const styles = stylex.create({
  backdrop: {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    overflow: 'auto',
    zIndex: 1000,
    WebkitOverflowScrolling: 'touch',
    boxSizing: 'border-box',
    textAlign: 'center',
    '::before': {
      display: 'inline-block',
      width: 0,
      height: '100%',
      verticalAlign: 'middle',
      content: ''
    }
  },
  position: (width: string) => ({
    position: 'relative',
    zIndex: 1001,
    outline: 'none',
    maxWidth: '90%',
    width,
    margin: '20px auto',
    verticalAlign: 'middle',
    display: 'inline-block'
  }),
  layer: {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    width: '100%',
    height: '100%',
    opacity: 0.25,
    backgroundColor: 'black',
    transition: 'opacity 0.35s cubic-bezier(0.4, 0, 0.2, 1)',
    pointerEvents: 'none',
    zIndex: 1000
  }
})

injectGlobalStyle({
  '.backdrop': {
    '&.backdrop-wrapper-enter .layer': {
      opacity: 0
    },
    '&.backdrop-wrapper-enter-active .layer': {
      opacity: 0.25
    },
    '&.backdrop-wrapper-leave .layer': {
      opacity: 0.25
    },
    '&.backdrop-wrapper-leave-active .layer': {
      opacity: 0
    }
  }
})

const Backdrop: React.FC<React.PropsWithChildren<BackdropProps>> = React.memo(
  ({
    children,
    onClick,
    visible = false,
    width,
    onContentClick,
    backdropClassName,
    layerClassName,
    ...props
  }) => {
    const isContentMouseDown = useRef(false)
    const handleClick = (event: MouseEvent<HTMLElement>) => {
      if (isContentMouseDown.current) { return }
      onClick?.(event)
    }
    const handleMouseUp = () => {
      if (!isContentMouseDown.current) { return }
      const timer = setTimeout(() => {
        isContentMouseDown.current = false
        clearTimeout(timer)
      }, 0)
    }

    const handleMouseDown = () => {
      isContentMouseDown.current = true
    }

    return (
      <CSSTransition name="backdrop-wrapper" visible={visible} clearTime={300}>
        <div
          role="presentation"
          className={clsx(stylex.props(styles.backdrop).className, backdropClassName, 'backdrop')}
          onClick={handleClick}
          onMouseUp={handleMouseUp}
          {...props}
        >
          <div className={clsx(stylex.props(styles.layer).className, layerClassName, 'layer')} />
          <div
            role="presentation"
            onClick={onContentClick}
            onMouseDown={handleMouseDown}
            {...stylex.props(styles.position(width!))}
          >
            {children}
          </div>
        </div>
      </CSSTransition>
    )
  }
)

export { Backdrop }
