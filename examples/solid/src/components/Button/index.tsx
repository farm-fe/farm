import { type ParentComponent } from 'solid-js';
import './index.css';

export const Button: ParentComponent<{ to: string }> = (props) => {
  return (
    <a class="farm-button" href={props.to} target="_blank">
      {props.children}
    </a>
  );
};
