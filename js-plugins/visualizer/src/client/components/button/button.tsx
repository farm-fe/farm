import { inline } from '@stylex-extend/core'
import * as stylex from '@stylexjs/stylex'
import { clsx } from 'clsx'
import React from 'react'
import { useScale, withScale } from '../../composables'

interface Props {
  icon?: React.ReactNode
  auto?: boolean
  type?: 'default' | 'secondary'
}

type ButtonProps =
  & Props
  & Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, keyof Props>

const styles = stylex.create({
  text: {
    position: 'relative',
    zIndex: 1,
    display: 'inline-flex',
    justifyContent: 'center',
    textAlign: 'center',
    lineHeight: 'inherit',
    top: '-1px'
  },
  icon: {
    position: 'absolute',
    right: 'auto',
    top: '50%',
    transform: 'translateY(-50%)',
    display: 'flex',
    justifyContent: 'center',
    color: '#666',
    alignItems: 'center',
    zIndex: 1,
    ':not(#_)  svg': {
      background: 'transparent',
      height: 'calc(var(--button-height) / 2.35)',
      width: 'calc(var(--button-height) / 2.35)'
    }
  }
})

function getButtonChildrenWithIcon(
  auto: boolean,
  icon: React.ReactNode,
  children: React.ReactNode
) {
  if (!icon) { return <div {...stylex.props(styles.text)}>{children}</div> }
  if (icon && !children) {
    return (
      <span
        {...stylex.props(
          styles.icon,
          inline({ position: 'static', transform: 'none' })
        )}
      >
        {icon}
      </span>
    )
  }
  return (
    <>
      <span {...stylex.props(styles.icon)}>{icon}</span>
      <div {...stylex.props(styles.text)}>{children}</div>
    </>
  )
}

const ButtonComponent = React.forwardRef<HTMLButtonElement, ButtonProps>(
  (props, ref) => {
    const {
      type = 'default',
      className: userClassName,
      style: userStyle,
      auto = false,
      icon,
      children,
      ...rest
    } = props

    const { SCALES } = useScale()
    const { className, style } = stylex.props(
      inline({
        boxSizing: 'border-box',
        borderRadius: '6px',
        fontWeight: 400,
        userSelect: 'none',
        outline: 'none',
        textTransform: 'capitalize',
        justifyContent: 'center',
        textAlign: 'center',
        whiteSpace: 'nowrap',
        transition: 'background-color 200ms ease 0s, box-shadow 200ms ease 0ms, border 200ms ease 0ms, color 200ms ease 0ms',
        position: 'relative',
        overflow: 'hidden',
        color: {
          default: '#666',
          ':hover': '#000'
        },
        backgroundColor: '#fff',
        border: '1px solid #eaeaea',
        cursor: 'pointer',
        width: 'initial',
        ':hover': {
          borderColor: '#000'
        },
        minWidth: auto ? 'min-content' : SCALES.width(10.5),
        lineHeight: SCALES.height(2.5),
        fontSize: SCALES.font(0.875),
        height: SCALES.height(2.5),
        padding: `${SCALES.pt(0)} ${auto ? SCALES.pr(1.15) : SCALES.pr(1.375)} ${SCALES.pt(0)} ${
          auto ? SCALES.pl(1.15) : SCALES.pl(1.375)
        }`,
        margin: `${SCALES.mt(0)} ${SCALES.mr(0)} ${SCALES.mb(0)} ${
          SCALES.ml(
            0
          )
        }`,
        '--button-height': SCALES.height(2.5),
        '--button-icon-padding': SCALES.pl(0.727),
        ...(auto && { width: 'auto' }),
        ...(type === 'secondary' && {
          backgroundColor: '#000',
          borderColor: '#000',
          color: '#fff'
        })
      })
    )

    const classes = clsx('button', className, userClassName)

    return (
      <button
        ref={ref}
        className={classes}
        style={{ ...style, ...userStyle }}
        {...rest}
        type="button"
      >
        {getButtonChildrenWithIcon(auto, icon, children)}
      </button>
    )
  }
)

export const Button = withScale(ButtonComponent)
