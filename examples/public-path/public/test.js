const say = "this is a test file"
console.log(say)

const div = document.createElement('div');
div.textContent = 'create from public script';
div.className = 'public-script';
document.body.appendChild(div)