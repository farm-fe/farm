import React from 'react'
import { useScale, withScale } from '../../composables'

type Props = React.HTMLAttributes<HTMLDivElement>

function DrawerContentComponent(props: React.PropsWithChildren<Props>) {
  const { SCALES } = useScale()
  const { children, ...rest } = props

  return (
    <div
      stylex={{
        position: 'relative',
        textAlign: 'left',
        fontSize: SCALES.font(1),
        width: SCALES.width(1, 'auto'),
        height: SCALES.height(1, 'auto'),
        padding: `${SCALES.pt(1.3125)} ${SCALES.pr(1.3125)} ${SCALES.pb(1.3125)} ${SCALES.pl(1.3125)}`
      }}
      {...rest}
    >
      {children}
    </div>
  )
}

export const DrawerContent = withScale(DrawerContentComponent)
