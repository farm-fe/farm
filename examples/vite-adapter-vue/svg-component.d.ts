
declare module '~virtual/svg-component' {
  const MySvgIcon: import("vue").DefineComponent<{
      name: {
          type: import("vue").PropType<"icon-vue">;
          default: string;
          required: true;
      };
  }, {}, unknown, {}, {}, import("vue").ComponentOptionsMixin, import("vue").ComponentOptionsMixin, {}, string, import("vue").VNodeProps & import("vue").AllowedComponentProps & import("vue").ComponentCustomProps, Readonly<import("vue").ExtractPropTypes<{
      name: {
          type: import("vue").PropType<"icon-vue">;
          default: string;
          required: true;
      };
  }>>, {
      name: "icon-vue";
  }>;
  export const svgNames: ["icon-vue"];
  export type SvgName = "icon-vue";
  export default MySvgIcon;
}
