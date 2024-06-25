const DEFAULT_DURATION = 3;

export const Holder = /*#__PURE__*/React.forwardRef((props, ref) => {
  const {
    duration = DEFAULT_DURATION,
  } = props;

  console.log(duration);
  return null;
});