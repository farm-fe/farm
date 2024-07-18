
declare module '~virtual/svg-component' {
  const MySvgIcon: (props: {
    name: "icon-vue",
    className?:string
    style?: React.CSSProperties
  })=> JSX.Element;
  export const svgNames: ["icon-vue"];
  export type SvgName = "icon-vue";
  export default MySvgIcon;
}
