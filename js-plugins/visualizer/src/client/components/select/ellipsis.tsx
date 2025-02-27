export function Ellipsis({ children, height }: { children: React.ReactNode, height: string }) {
  return (
    <span
      stylex={{
        overflow: 'hidden',
        textOverflow: 'ellipsis',
        whiteSpace: 'nowrap',
        minWidth: 0,
        lineHeight: height
      }}
    >
      {children}
    </span>
  )
}
