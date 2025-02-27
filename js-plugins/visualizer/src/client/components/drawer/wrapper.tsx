import { inline } from '@stylex-extend/core'
import * as stylex from '@stylexjs/stylex'
import { clsx } from 'clsx'
import React from 'react'
import { useScale } from '../../composables'
import { CSSTransition } from '../css-transition'

interface Props {
  visible?: boolean
}

export type DrawerWrapperProps = Omit<React.HTMLAttributes<unknown>, keyof Props> & Props

function DrawerWrapper(props: React.PropsWithChildren<DrawerWrapperProps>) {
  const { visible, children } = props
  const { SCALES } = useScale()
  const { className, style } = stylex.props(inline({
    position: 'fixed',
    top: 0,
    left: 0,
    right: 'auto',
    bottom: 0,
    maxWidth: '100%',
    verticalAlign: 'middle',
    overflow: 'auto',
    display: 'flex',
    flexDirection: 'column',
    boxSizing: 'border-box',
    backgroundColor: '#fff',
    color: '#000',
    borderRadius: '6px',
    boxShadow: '0 30px 60px rgba(0, 0, 0, 0.12)',
    opacity: 0,
    outline: 'none',
    transform: 'translate3d(-100%, 0, 0)',
    transition: 'opacity, transform 400ms cubic-bezier(0.1, 0.6, 0.1, 1)',
    borderTopLeftRadius: 0,
    borderBottomLeftRadius: 0,
    ':not(#_).wrapper-enter': {
      opacity: 0,
      transform: 'translate3d(-100%, 0, 0)'
    },
    ':not(#_).wrapper-enter-active': {
      opacity: 1,
      transform: 'translate3d(0, 0, 0)'
    },
    ':not(#_).wrapper-leave': {
      opacity: 1,
      transition: 'opacity, transform 400ms cubic-bezier(0.1, 0.2, 0.1, 1)'
    },
    ':not(#_).wrapper-leave-active': {
      opacity: 0.4,
      transform: 'translate3d(-100%, 0, 0)'
    },
    fontSize: SCALES.font(1),
    '--wrapper-padding-left': SCALES.pl(1.3125),
    '--wrapper-padding-right': SCALES.pr(1.3125),
    padding: `${SCALES.pt(1.3125)} var(--wrapper-padding-right) ${SCALES.pb(1.3125)} var(--wrapper-padding-left)`,
    margin: `${SCALES.mt(0)} ${SCALES.mr(0)} ${SCALES.mb(0)} ${SCALES.ml(0)}`,
    width: SCALES.width(1, 'auto'),
    height: SCALES.height(1, '100%')
  }))
  const classes = clsx(className, 'wrapper')

  return (
    <CSSTransition name="wrapper" visible={visible} clearTime={300}>
      <div role="dialog" tabIndex={-1} className={classes} style={style}>
        {children}
      </div>
    </CSSTransition>
  )
}

export { DrawerWrapper }
