
/*
Use the rspress tabs component to rewrite tabs
https://rspress.dev/
*/
import React from 'react';
import {
  Children,
  type ReactNode,
  type ReactElement,
  useMemo,
  useState,
  useEffect,
  useContext,
  forwardRef,
  type ForwardedRef,
  isValidElement,
  type ComponentPropsWithRef,
  type ForwardRefExoticComponent,
} from 'react';
import { TabDataContext } from '../logic/TabDataContext';
import { useStorageValue } from '../logic/useStorageValue';
import styles from './index.module.css';

type TabItem = {
  value?: string;
  label?: string | ReactNode;
  disabled?: boolean;
};

interface TabsProps {
  values?: ReactNode[] | ReadonlyArray<ReactNode> | TabItem[];
  defaultValue?: string;
  onChange?: (index: number) => void;
  children: ReactNode;
  groupId?: string;
  tabContainerClassName?: string;
  tabPosition?: 'left' | 'center';
}

function isTabItem(item: unknown): item is TabItem {
  if (item && typeof item === 'object' && 'label' in item) {
    return true;
  }
  return false;
}

const renderTab = (item: ReactNode | TabItem) => {
  if (isTabItem(item)) {
    return item.label || item.value;
  }
  return item;
};

export const groupIdPrefix = 'tabs.';

export const Tabs: ForwardRefExoticComponent<TabsProps> = forwardRef(
  (props: TabsProps, ref: ForwardedRef<any>): ReactElement => {
    const {
      values,
      defaultValue,
      onChange,
      children: rawChildren,
      groupId,
      tabPosition = 'left',
      tabContainerClassName,
    } = props;
    // remove "\n" character when write JSX element in multiple lines, use Children.toArray for Tabs with no Tab element
    const children = Children.toArray(rawChildren).filter(
      child => !(typeof child === 'string' && child.trim() === ''),
    );


    let tabValues = values || [];

    if (tabValues.length === 0) {
      tabValues = Children.map(children, child => {
        if (isValidElement(child)) {
          return {
            label: child.props?.label,
            value: child.props?.value || child.props?.label,
          };
        }

        return {
          label: undefined,
          value: undefined,
        };
      });
    }

    const { tabData, setTabData } = useContext(TabDataContext);
    const [activeIndex, setActiveIndex] = useState(() => {
      if (defaultValue === undefined) {
        return 0;
      }

      return tabValues.findIndex(item => {
        if (typeof item === 'string') {
          return item === defaultValue;
        }
        if (item && typeof item === 'object' && 'value' in item) {
          return item.value === defaultValue;
        }
        return false;
      });
    });

    const [storageIndex, setStorageIndex] = useStorageValue(
      `${groupIdPrefix}${groupId}`,
      activeIndex,
    );


    const syncIndex = useMemo(() => {
      if (groupId) {
        if (tabData[groupId] !== undefined) {
          return tabData[groupId];
        }

        return Number.parseInt(storageIndex);
      }

      return activeIndex;
    }, [tabData[groupId]]);

    // sync when other browser page trigger update
    useEffect(() => {
      if (groupId) {
        const correctIndex = Number.parseInt(storageIndex);

        if (syncIndex !== correctIndex) {
          setTabData({ ...tabData, [groupId]: correctIndex });
        }
      }
    }, [storageIndex]);

    // const currentIndex = groupId ? syncIndex : activeIndex;
    useEffect(() => {
      if (groupId) {
        setCurrentIndex(syncIndex);
      } else {
        setCurrentIndex(activeIndex);
      }
    }, [groupId, syncIndex, activeIndex]);

    const [currentIndex, setCurrentIndex] = useState(groupId ? syncIndex : activeIndex);

    return (
      <div className={styles['container']} ref={ref}>
        <div className={tabContainerClassName}>
          {tabValues.length ? (
            <div
              className={`${styles['tab-list']} ${styles['no-scrollbar']}`}
              style={{
                display: 'flex',
                justifyContent:
                  tabPosition === 'center' ? 'center' : 'flex-start',
              }}
            >
              {tabValues.map((item, index) => {
                return (
                  <div
                    key={index}
                    className={`${styles['tab']} ${currentIndex === index
                      ? styles['selected']
                      : styles['not-selected']
                      }`}
                    onClick={() => {
                      const newIndex = index;
                      onChange?.(newIndex);
                      setCurrentIndex(newIndex);

                      if (groupId) {
                        setTabData({ ...tabData, [groupId]: newIndex });
                        setStorageIndex(newIndex);
                      } else {
                        setActiveIndex(newIndex);
                      }
                    }}
                  >
                    {renderTab(item)}
                  </div>
                );
              })}
            </div>
          ) : null}
        </div>
        <div>{Children.toArray(children)[currentIndex]}</div>
      </div>
    );
  },
);

export function Tab({
  children,
  ...props
}: ComponentPropsWithRef<'div'> &
  Pick<TabItem, 'label' | 'value'>): ReactElement {
  return (
    <div {...props} className="rounded-md">
      {children}
    </div>
  );
}
