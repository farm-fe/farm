// Import required modules
import Koa from 'koa';
import compress from 'koa-compress';

// Create a Koa application
const app = new Koa();

// Use koa-compress middleware
app.use(compress());

// Define a route that sends a long text response
app.use(async (ctx) => {
  ctx.body = Array(10000).fill('This is a long text. ').join('');
});

// Start the server on port 3000
const PORT = 8000;
app.listen(PORT, () => {
  console.log(`Server running on http://localhost:${PORT}`);
});
