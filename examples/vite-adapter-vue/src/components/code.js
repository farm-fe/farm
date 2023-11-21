import { ref } from 'vue';
import * as prettier from "prettier";
import bcrypt from "bcryptjs";
console.log(bcrypt);
const code = ref(`const a = 1;`);
const str = ref(`213`);
var salt = bcrypt.genSaltSync(10);
var hash = bcrypt.hashSync("B4c0/\/", salt);
const res = ref(hash);
const format = async () => {
  const formatted = await prettier.format(code.value, {
    parser: "babel",
    plugins: [require("prettier/plugins/babel"), require("prettier/plugins/estree")],
  });
  code.value = formatted;
};