import { businessQuote } from './business-quote.mjs';
export function showQuote(product) {
  const detail = document.querySelector('#detail');
  detail.innerHTML = '<h2></h2><label>Quantity <input id="quantity" type="number" value="2"></label><button id="quote">Quote</button><output id="total"></output>';
  detail.querySelector('h2').textContent = product.name;
  document.querySelector('#quote').onclick = () => {
    try {
      const quote = businessQuote({ price: product.price, stock: product.stock, quantity: Number(document.querySelector('#quantity').value), discount: 5, destination: 'local', rules: { reserveStock: 2, maxDiscount: 20 }, tiers: [{ minQuantity: 3, discount: 5 }], shipping: [{ destination: 'local', freeAbove: 10000, cents: 500 }] });
      document.querySelector('#total').textContent = String(quote.total);
    } catch (error) { document.querySelector('#total').textContent = error.message; }
  };
}
