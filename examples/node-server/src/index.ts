import express from 'express';
const app = express();
app.listen(12306, () => {
  console.log('server up');
  process.exit(0);
});
