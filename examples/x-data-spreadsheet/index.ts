import Spreadsheet from "x-data-spreadsheet";
import zip from "jszip";

// If you need to override the default options, you can set the override
// const options = {};
// new Spreadsheet('#x-spreadsheet-demo', options);
const s = new Spreadsheet("#x-spreadsheet-demo")
  .loadData({}) // load data
  .change(data => {
    // save data to db
    console.log(data);
    zip.file("Hello.txt", JSON.stringify(data));

    zip.generateAsync({type:"blob"}).then(function(content) {
      content;
    });
  });