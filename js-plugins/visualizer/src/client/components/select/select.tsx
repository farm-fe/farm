import { noop } from 'foxact/noop';
import React, {
  useCallback,
  useImperativeHandle,
  useMemo,
  useRef,
  useState
} from 'react';
import { useScale, withScale } from '../../composables';
import { Provider } from './context';
import { SelectDropdown } from './dropdown';
import { Ellipsis } from './ellipsis';
import { SelectMultipleValue } from './select-multiple';
import { SelectOption } from './select-option';
import type { SelectOptionProps } from './select-option';
import { colors } from '../../themes/color.stylex';

interface Props {
  options: SelectOptionProps[];
  disabled?: boolean;
  value?: string | string[];
  placeholder?: string;
  multiple?: boolean;
  pure?: boolean;
  clearable?: boolean;
  onChange?: (value: string | string[]) => void;
}

export type SelectProps = Omit<React.HTMLAttributes<unknown>, keyof Props> &
  Props;

export type SelectInstance = {
  destory: () => void;
};

function getSelectValue(
  value: string | string[] | undefined,
  next: string,
  multiple: boolean
) {
  if (multiple) {
    if (!Array.isArray(value)) {
      return [next];
    }
    if (!value.includes(next)) {
      return [...value, next];
    }
    return value.filter((item) => item !== next);
  }
  return next;
}

const SelectComponent = React.forwardRef(
  (props: SelectProps, ref: React.Ref<SelectInstance>) => {
    const {
      disabled = false,
      value: userValue,
      placeholder,
      clearable = true,
      options,
      multiple = false,
      pure = false,
      onChange,
      ...rest
    } = props;

    const elementRef = useRef<HTMLDivElement>(null);

    const { SCALES } = useScale();
    const [visible, setVisible] = useState<boolean>(false);

    const [value, setValue] = useState<string | string[] | undefined>(() => {
      if (!multiple) {
        return userValue;
      }
      if (Array.isArray(userValue)) {
        return userValue;
      }
      return typeof userValue === 'undefined' ? [] : [userValue];
    });

    const isEmpty = useMemo(() => {
      if (!Array.isArray(value)) {
        return !value;
      }
      return value.length === 0;
    }, [value]);

    const updateVisible = (next: boolean) => {
      setVisible(next);
    };

    const updateValue = useCallback(
      (next: string) => {
        const nextValue = getSelectValue(value, next, multiple);
        setValue(nextValue);
        onChange?.(nextValue);
        if (!multiple) {
          updateVisible(false);
        }
      },
      [multiple, onChange, value]
    );

    const selectChild = useMemo(() => {
      const data = (Array.isArray(value) ? value : [value]).filter(Boolean);
      const getLabel = (val: string) => {
        const option = options.find((opt) => opt.value === val);
        return option?.label || val;
      };

      if (!multiple) {
        return (
          <SelectOption pure preventAllEvents>
            {getLabel(data[0])}
          </SelectOption>
        );
      }

      return data.map((item) => (
        <SelectMultipleValue
          key={item}
          onClear={clearable ? () => updateValue(item) : noop}
          disabled={disabled}
        >
          {getLabel(item)}
        </SelectMultipleValue>
      ));
    }, [value, multiple, clearable, disabled, updateValue, options]);

    const initialValue = useMemo(() => {
      return {
        value,
        visible,
        disableAll: disabled,
        ref: elementRef,
        updateValue,
        updateVisible
      };
    }, [visible, value, disabled, updateValue]);

    const handleClick = (event: React.MouseEvent<HTMLDivElement>) => {
      event.stopPropagation();
      event.nativeEvent.stopImmediatePropagation();
      event.preventDefault();
      if (disabled) {
        return;
      }
      updateVisible(!visible);
      event.preventDefault();
    };

    const handleMouseDown = (event: React.MouseEvent<HTMLDivElement>) => {
      if (visible) {
        event.preventDefault();
      }
    };

    useImperativeHandle(ref, () => {
      return {
        destory: () => updateVisible(false)
      };
    });

    return (
      <Provider value={initialValue}>
        <div
          ref={elementRef}
          role='presentation'
          onClick={handleClick}
          onMouseDown={handleMouseDown}
          stylex={{
            display: 'inline-flex',
            alignItems: 'center',
            userSelect: 'none',
            whiteSpace: 'nowrap',
            position: 'relative',
            maxWidth: '90vw',
            overflow: 'hidden',
            transition:
              ' border 150ms ease-in 0s, color 200ms ease-out 0s, box-shadow 200ms ease 0s',
            border: `1px solid ${colors.border}`,
            borderRadius: '6px',
            ':hover': {
              borderColor: colors.foreground
            },
            ...(disabled && { ':hover': { borderColor: '#eaeaea' } }),
            '--select-font-size': SCALES.font(0.875),
            '--select-height': SCALES.height(2.25),
            '--disabled-color': disabled ? '#888' : '#000',
            width: SCALES.width(1, 'initial'),
            height: multiple ? 'auto' : 'var(--select-height)',
            padding: `${SCALES.pt(0)} ${SCALES.pr(0.334)} ${SCALES.pb(0)} ${SCALES.pl(0.667)}`,
            margin: `${SCALES.mt(0)} ${SCALES.mr(0)} ${SCALES.mb(0)} ${SCALES.ml(0)}`,
            cursor: 'pointer',
            ...(disabled && { cursor: 'not-allowed' }),
            minHeight: 'var(--select-height)'
          }}
          {...rest}
        >
          <input
            aria-haspopup='listbox'
            readOnly
            stylex={{
              position: 'fixed',
              top: '-10000px',
              left: '-10000px',
              opacity: 0,
              zIndex: -1,
              width: 0,
              height: 0,
              padding: 0,
              fontSize: 0,
              border: 'none'
            }}
          />
          {isEmpty && (
            <span
              stylex={{
                display: 'inline-flex',
                flex: 1,
                height: 'var(--select-height)',
                alignItems: 'center',
                lineHeight: 1,
                padding: 0,
                marginRight: '1.25em',
                fontSize: 'var(--select-font-size)',
                color: '#999'
              }}
            >
              <Ellipsis height='var(--scale-height)'>{placeholder}</Ellipsis>
            </span>
          )}
          {value && (
            <div stylex={{ display: 'flex', flexWrap: 'wrap' }}>
              {selectChild}
            </div>
          )}
          <SelectDropdown visible={visible}>
            {options.map((item) => (
              <SelectOption key={item.value} {...item}>
                {item.label}
              </SelectOption>
            ))}
          </SelectDropdown>
          {!pure && (
            <div
              stylex={{
                position: 'absolute',
                right: '4pt',
                fontSize: 'var(--select-font-size)',
                top: '50%',
                bottom: 0,
                transform: 'translateY(-50%)',
                pointerEvents: 'none',
                transition: 'transform 200ms ease',
                display: 'flex',
                alignItems: 'center',
                color: '#666',
                ...(visible && { transform: 'translateY(-50%) rotate(180deg)' })
              }}
            >
              <svg
                viewBox='0 0 24 24'
                strokeWidth='1'
                strokeLinecap='round'
                strokeLinejoin='round'
                fill='none'
                shapeRendering='geometricPrecision'
                stylex={{
                  color: 'inherit',
                  stroke: 'currentColor',
                  transition: 'all 200ms ease',
                  width: '1.214em',
                  height: '1.214em'
                }}
              >
                <path d='M6 9l6 6 6-6' />
              </svg>
            </div>
          )}
        </div>
      </Provider>
    );
  }
);

export const Select = withScale(SelectComponent);
