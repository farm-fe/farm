/* eslint-disable @eslint-react/no-clone-element */
import clsx from 'clsx';
import React, { useEffect, useState } from 'react';

interface Props {
  visible?: boolean;
  enterTime?: number;
  leaveTime?: number;
  clearTime?: number;
  className?: string;
  name?: string;
}

export type CssTransitionProps = Props;

function CSSTransition({
  children,
  className,
  visible = false,
  enterTime = 60,
  leaveTime = 60,
  clearTime = 60,
  name = 'transition',
  ...props
}: React.PropsWithChildren<CssTransitionProps>) {
  const statusClassName = visible ? 'enter' : 'leave';
  const time = visible ? enterTime : leaveTime;
  const [classes, setClasses] = useState<string>(
    () => `${name}-${statusClassName}`
  );
  const [renderable, setRenderable] = useState<boolean>(visible);

  if (visible && !renderable) {
    setRenderable(true);
  }

  useEffect(() => {
    const timer = setTimeout(() => {
      setClasses(
        `${name}-${statusClassName} ${name}-${statusClassName}-active`
      );
      clearTimeout(timer);
    }, time);
    const clearClassesTimer = setTimeout(() => {
      if (!visible) {
        setClasses('');
        setRenderable(false);
      }
      clearTimeout(clearClassesTimer);
    }, time + clearTime);
    return () => {
      clearTimeout(timer);
      clearTimeout(clearClassesTimer);
    };
  }, [clearTime, name, statusClassName, time, visible]);

  if (!React.isValidElement(children) || !renderable) {
    return null;
  }

  return React.cloneElement(children, {
    ...props,
    // @ts-expect-error safe
    // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-argument
    className: clsx(children.props.className, className, classes)
  });
}

export { CSSTransition };
