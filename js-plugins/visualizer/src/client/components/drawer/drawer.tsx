import React, { useState } from 'react'
import { createPortal } from 'react-dom'
import { useBodyScroll, usePortal, withScale } from '../../composables'
import { Backdrop } from './backdrop'
import { DrawerWrapper } from './wrapper'

interface Props {
  visible?: boolean
  onClose?: () => void
  onContentClick?: (event: React.MouseEvent<HTMLElement>) => void
}

export type DrawerProps = Omit<React.HTMLAttributes<unknown>, keyof Props> & Props

function DrawerComponent(props: DrawerProps) {
  const { visible: userVisible = false, children, onClose, ...rest } = props
  const portal = usePortal('drawer')
  const [visible, setVisible] = useState<boolean>(userVisible)
  const [, setBodyHidden] = useBodyScroll({ delayReset: 300 })

  if (typeof userVisible !== 'undefined' && userVisible !== visible) {
    setVisible(userVisible)
    setBodyHidden(userVisible)
  }

  const closeDrawer = () => {
    onClose?.()
    setVisible(false)
    setBodyHidden(false)
  }

  const closeFromBackdrop = () => {
    closeDrawer()
  }
  if (!portal) { return null }
  return createPortal(
    <Backdrop onClick={closeFromBackdrop} visible={visible} width="100%">
      <DrawerWrapper visible={visible} {...rest}>{children}</DrawerWrapper>
    </Backdrop>,
    portal
  )
}

export const Drawer = withScale(DrawerComponent)
