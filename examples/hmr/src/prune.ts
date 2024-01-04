const id = 'comp-style';
const style = document.createElement('style');
style.id = id;
console.log('style', style);
style.innerHTML = `
  .box {
    margin-top: 20px;
    width: fit-content;
    height: 50px;
    background-color: purple;
    line-height: 50px;
    padding: 15px;
    color: white;
  }
`;

const existingStyle = document.getElementById(id);
if (existingStyle) {
  existingStyle.innerHTML = style.innerHTML;
} else {
  document.head.appendChild(style);
}

if (import.meta.hot) {
  import.meta.hot.accept();
  import.meta.hot.prune(() => {
    const style = document.getElementById(id)
    style?.remove()
  })
}