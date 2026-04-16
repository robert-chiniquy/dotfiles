import type { VerifyContext } from '../runner.js';
import { CheckoutPage } from '../pages/CheckoutPage.js';

export default async function (ctx: VerifyContext) {
  const page = new CheckoutPage(ctx);

  await ctx.step('navigate-home', () => page.navigateHome());
  await ctx.step('add-to-cart', () => page.addItemToCart('sku-001'));
  await ctx.step('open-cart', () => page.openCart());

  await ctx.step('assert-cart-count-1', async () => {
    const count = await page.cartCount();
    if (count !== 1) throw new Error(`expected 1, got ${count}`);
  });

  await ctx.step('proceed-to-checkout', () => page.proceedToCheckout());
  await ctx.step('assert-checkout-visible', () => page.assertCheckoutVisible());
}
