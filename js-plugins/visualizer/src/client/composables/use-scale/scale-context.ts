import React from 'react';

export const ScalePropKeys = [
  'width',
  'height',
  'padding',
  'margin',
  'w',
  'h',
  'paddingLeft',
  'paddingRight',
  'paddingTop',
  'paddingBottom',
  'pl',
  'pr',
  'pt',
  'pb',
  'marginLeft',
  'marginRight',
  'marginTop',
  'marginBottom',
  'ml',
  'mr',
  'mt',
  'mb',
  'px',
  'py',
  'mx',
  'my',
  'font',
  'unit',
  'scale'
];

export type ScaleProps = {
  width?: string | number;
  height?: string | number;
  padding?: string | number;
  margin?: string | number;
  w?: string | number;
  h?: string | number;
  paddingLeft?: string | number;
  paddingRight?: string | number;
  paddingTop?: string | number;
  paddingBottom?: string | number;
  pl?: string | number;
  pr?: string | number;
  pt?: string | number;
  pb?: string | number;
  marginLeft?: string | number;
  marginRight?: string | number;
  marginTop?: string | number;
  marginBottom?: string | number;
  ml?: string | number;
  mr?: string | number;
  mt?: string | number;
  mb?: string | number;
  px?: string | number;
  py?: string | number;
  mx?: string | number;
  my?: string | number;
  font?: string | number;
  unit?: string;
  scale?: number;
};

export type DynamicLayoutPipe = (
  scale1x: number,
  defaultValue?: string | number
) => string;

export type ScaleInputKeys =
  | 'pl'
  | 'pr'
  | 'pt'
  | 'pb'
  | 'px'
  | 'py'
  | 'ml'
  | 'mr'
  | 'mt'
  | 'mb'
  | 'mx'
  | 'my'
  | 'width'
  | 'height'
  | 'font';

export type DynamicScales = {
  [key in ScaleInputKeys]: DynamicLayoutPipe;
};

export type GetScalePropsFunction = (
  key: keyof ScaleProps | Array<keyof ScaleProps>
) => ScaleProps[keyof ScaleProps];

export type GetAllScalePropsFunction = () => ScaleProps;

export interface ScaleConfig {
  SCALES: DynamicScales;
  getScaleProps: GetScalePropsFunction;
  getAllScaleProps: GetAllScalePropsFunction;
  unit: string;
}

export type SCALES = DynamicScales;

const defaultDynamicLayoutPipe: DynamicLayoutPipe = (scale1x) => {
  return `${scale1x}`;
};

const defaultContext: ScaleConfig = {
  getScaleProps: () => undefined,
  getAllScaleProps: () => ({}),
  SCALES: {
    pl: defaultDynamicLayoutPipe,
    pr: defaultDynamicLayoutPipe,
    pb: defaultDynamicLayoutPipe,
    pt: defaultDynamicLayoutPipe,
    px: defaultDynamicLayoutPipe,
    py: defaultDynamicLayoutPipe,
    mb: defaultDynamicLayoutPipe,
    ml: defaultDynamicLayoutPipe,
    mr: defaultDynamicLayoutPipe,
    mt: defaultDynamicLayoutPipe,
    mx: defaultDynamicLayoutPipe,
    my: defaultDynamicLayoutPipe,
    width: defaultDynamicLayoutPipe,
    height: defaultDynamicLayoutPipe,
    font: defaultDynamicLayoutPipe
  },
  unit: '16px'
};

const ScaleContext = React.createContext(defaultContext);
export const useScale = () => React.useContext(ScaleContext);
export const ScaleProvider = ScaleContext.Provider;
