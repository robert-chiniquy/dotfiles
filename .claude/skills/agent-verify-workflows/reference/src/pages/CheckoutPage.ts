import type { VerifyContext } from '../runner.js';

export class CheckoutPage {
  constructor(private ctx: VerifyContext) {}

  async navigateHome() {
    await this.ctx.page.goto(this.ctx.baseUrl);
  }

  async addItemToCart(sku: string) {
    await this.ctx.page
      .locator(`[data-test-sku="${sku}"] button.add-to-cart`)
      .click();
  }

  async openCart() {
    await this.ctx.page.locator('[data-test="cart-link"]').click();
  }

  async cartCount(): Promise<number> {
    const text = await this.ctx.page.locator('[data-test="cart-count"]').textContent();
    return Number(text ?? '0');
  }

  async proceedToCheckout() {
    await this.ctx.page.locator('[data-test="checkout-button"]').click();
  }

  async assertCheckoutVisible() {
    await this.ctx.page.waitForSelector('[data-test="checkout-form"]', { timeout: 5000 });
  }
}
